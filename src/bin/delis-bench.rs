use clap::Parser;
use delis::process::{cleanup_children, register_pid, unregister_pid};
use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Run Delis benchmarks from a TOML manifest")]
struct Cli {
    #[arg(value_name = "TOML")]
    manifest: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BenchmarkConfig {
    delis: Option<PathBuf>,
    mode: Option<BenchMode>,
    sygus_solver: Option<String>,
    #[serde(default)]
    sygus_solver_args: Vec<String>,
    #[serde(default = "default_timeout_secs")]
    sygus_timeout_secs: u64,
    #[serde(default = "default_timeout_secs")]
    solve_timeout_secs: u64,
    results_dir: Option<PathBuf>,
    #[serde(default)]
    folders: Vec<FolderConfig>,
}

#[derive(Clone, Debug, Deserialize)]
struct FolderConfig {
    name: Option<String>,
    path: PathBuf,
    mode: Option<BenchMode>,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    delis_args: Vec<String>,
    sygus_timeout_secs: Option<u64>,
    solve_timeout_secs: Option<u64>,
    sygus_solver: Option<String>,
    #[serde(default)]
    sygus_solver_args: Vec<String>,
    results: Option<PathBuf>,
    #[serde(default)]
    rerun: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BenchMode {
    Native,
    ExternalSygus,
}

#[derive(Clone, Debug)]
struct ResolvedFolder {
    name: String,
    path: PathBuf,
    mode: BenchMode,
    delis_args: Vec<String>,
    sygus_timeout: Duration,
    solve_timeout: Duration,
    solver_cmd: String,
    solver_args: Vec<String>,
    files: Vec<PathBuf>,
    results_path: PathBuf,
    artifacts_dir: PathBuf,
    rerun: bool,
}

#[derive(Debug)]
enum Status {
    Solved,
    Infeasible,
    SygusTimeout,
    SolveTimeout,
    Error,
}

#[derive(Debug)]
struct Outcome {
    group: String,
    path: String,
    status: Status,
    total_duration: Duration,
    sygus_duration: Duration,
    solve_duration: Duration,
    sygus_bytes: usize,
    sygus_lines: usize,
    solution_bytes: usize,
    delis_exit_code: Option<i32>,
    solver_exit_code: Option<i32>,
    delis_args: Vec<String>,
    solver_cmd: String,
    solver_args: Vec<String>,
    solution_path: String,
    detail: String,
}

#[derive(Debug)]
struct CommandResult {
    status: ExitStatus,
    duration: Duration,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug)]
enum CommandRun {
    Completed(CommandResult),
    TimedOut {
        duration: Duration,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

fn default_timeout_secs() -> u64 {
    5
}

fn load_config(path: &Path) -> Result<BenchmarkConfig, io::Error> {
    let text = fs::read_to_string(path)?;
    let config: BenchmarkConfig = toml::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid manifest: {e}")))?;
    if config.folders.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "manifest has no benchmark folders",
        ));
    }
    Ok(config)
}

fn resolve_relative(base_file: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    }
}

fn resolve_program_path(base_file: &Path, program: &Path) -> PathBuf {
    if program.is_absolute() || program.components().count() == 1 {
        program.to_path_buf()
    } else {
        resolve_relative(base_file, program)
    }
}

fn resolve_delis_path(manifest: &Path, configured: Option<&PathBuf>) -> Result<PathBuf, io::Error> {
    if let Some(path) = configured {
        return Ok(resolve_program_path(manifest, path));
    }
    if let Ok(path) = env::var("DELIS_BENCH_SOLVER") {
        return Ok(PathBuf::from(path));
    }
    let exe = env::current_exe()?;
    exe.parent()
        .map(|dir| dir.join("delis"))
        .ok_or_else(|| io::Error::other("cannot infer delis path"))
}

fn default_solver_args(cmd: &str) -> Vec<String> {
    let path = Path::new(cmd);
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or(cmd);
    match name {
        "dryadsynth-bv" => vec!["-in".to_string()],
        "cvc5" => vec!["--lang=sygus2".to_string()],
        _ => Vec::new(),
    }
}

fn list_benchmark_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if recursive {
                files.extend(list_benchmark_files(&path, true)?);
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("sl") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "results".to_string()
    } else {
        slug.to_string()
    }
}

fn resolve_folders(config: &BenchmarkConfig, manifest: &Path) -> Result<Vec<ResolvedFolder>, io::Error> {
    let results_root = resolve_relative(
        manifest,
        config
            .results_dir
            .as_deref()
            .unwrap_or_else(|| Path::new("bench-results")),
    );
    let mut resolved = Vec::with_capacity(config.folders.len());
    for folder in &config.folders {
        let path = resolve_relative(manifest, &folder.path);
        let files = list_benchmark_files(&path, folder.recursive)?;
        if files.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("benchmark folder {} has no .sl files", path.display()),
            ));
        }
        let name = folder
            .name
            .clone()
            .unwrap_or_else(|| folder.path.display().to_string());
        let mode = folder.mode.or(config.mode).unwrap_or(BenchMode::Native);
        let (solver_cmd, solver_args) = if mode == BenchMode::ExternalSygus {
            let solver_cmd = folder
                .sygus_solver
                .clone()
                .or_else(|| config.sygus_solver.clone())
                .or_else(|| env::var("DELIS_BENCH_SYGUS_SOLVER").ok())
                .unwrap_or_else(|| "cvc5".to_string());
            let solver_cmd = resolve_program_path(manifest, Path::new(&solver_cmd))
                .display()
                .to_string();
            let solver_args = if !folder.sygus_solver_args.is_empty() {
                folder.sygus_solver_args.clone()
            } else if !config.sygus_solver_args.is_empty() {
                config.sygus_solver_args.clone()
            } else {
                default_solver_args(&solver_cmd)
            };
            (solver_cmd, solver_args)
        } else {
            (String::new(), Vec::new())
        };
        let results_path = match &folder.results {
            Some(path) => resolve_relative(manifest, path),
            None => results_root.join(format!("{}.tsv", slugify(&name))),
        };
        let artifacts_dir = results_root.join("artifacts").join(slugify(&name));
        resolved.push(ResolvedFolder {
            name,
            path,
            mode,
            delis_args: folder.delis_args.clone(),
            sygus_timeout: Duration::from_secs(
                folder
                    .sygus_timeout_secs
                    .unwrap_or(config.sygus_timeout_secs),
            ),
            solve_timeout: Duration::from_secs(
                folder
                    .solve_timeout_secs
                    .unwrap_or(config.solve_timeout_secs),
            ),
            solver_cmd,
            solver_args,
            files,
            results_path,
            artifacts_dir,
            rerun: folder.rerun,
        });
    }
    Ok(resolved)
}

fn spawn_command(program: &Path, args: &[String], stdin_bytes: Option<&[u8]>) -> Result<Child, io::Error> {
    let mut child = std::process::Command::new(program)
        .args(args)
        .stdin(if stdin_bytes.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    register_pid(child.id());
    if let Some(input) = stdin_bytes {
        if let Some(mut stdin) = child.stdin.take() {
            let bytes = input.to_vec();
            thread::Builder::new()
                .name("delis-bench-stdin-writer".to_string())
                .spawn(move || {
                    let _ = stdin.write_all(&bytes);
                    let _ = stdin.flush();
                })?;
        }
    }
    Ok(child)
}

fn run_command(
    program: &Path,
    args: &[String],
    stdin_bytes: Option<&[u8]>,
    timeout: Duration,
) -> Result<CommandRun, io::Error> {
    let mut child = spawn_command(program, args, stdin_bytes)?;
    let pid = child.id();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            unregister_pid(pid);
            let output = child.wait_with_output()?;
            return Ok(CommandRun::Completed(CommandResult {
                status,
                duration: start.elapsed(),
                stdout: output.stdout,
                stderr: output.stderr,
            }));
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            unregister_pid(pid);
            let output = child.wait_with_output()?;
            return Ok(CommandRun::TimedOut {
                duration: start.elapsed(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn summarize_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let normalized = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    let normalized = normalized.replace('\t', " ");
    if normalized.len() > 240 {
        format!("{}...", &normalized[..240])
    } else {
        normalized
    }
}

fn count_lines(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        0
    } else {
        String::from_utf8_lossy(bytes).lines().count()
    }
}

fn status_name(status: &Status) -> &'static str {
    match status {
        Status::Solved => "solved",
        Status::Infeasible => "infeasible",
        Status::SygusTimeout => "sygus_timeout",
        Status::SolveTimeout => "solve_timeout",
        Status::Error => "error",
    }
}

fn mode_name(mode: BenchMode) -> &'static str {
    match mode {
        BenchMode::Native => "native",
        BenchMode::ExternalSygus => "external_sygus",
    }
}

fn join_args(args: &[String]) -> String {
    args.join(" ")
}

fn sanitize_field(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ")
}

fn relative_display(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn artifact_path(folder: &ResolvedFolder, file: &Path) -> PathBuf {
    let relative = file.strip_prefix(&folder.path).unwrap_or(file);
    folder.artifacts_dir.join(relative)
}

fn write_solution_artifact(path: &Path, bytes: &[u8]) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

fn write_native_solution_artifact(
    folder: &ResolvedFolder,
    file: &Path,
    bytes: &[u8],
    manifest_dir: &Path,
) -> Result<String, io::Error> {
    if bytes.is_empty() {
        return Ok(String::new());
    }
    let artifact = artifact_path(folder, file);
    write_solution_artifact(&artifact, bytes)?;
    Ok(relative_display(&artifact, manifest_dir))
}

fn outcome_row(outcome: &Outcome) -> String {
    let delis_exit_code = outcome
        .delis_exit_code
        .map(|code| code.to_string())
        .unwrap_or_default();
    let solver_exit_code = outcome
        .solver_exit_code
        .map(|code| code.to_string())
        .unwrap_or_default();
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        sanitize_field(&outcome.group),
        sanitize_field(&outcome.path),
        status_name(&outcome.status),
        outcome.total_duration.as_millis(),
        outcome.sygus_duration.as_millis(),
        outcome.solve_duration.as_millis(),
        outcome.sygus_bytes,
        outcome.sygus_lines,
        outcome.solution_bytes,
        delis_exit_code,
        solver_exit_code,
        sanitize_field(&join_args(&outcome.delis_args)),
        sanitize_field(&outcome.solver_cmd),
        sanitize_field(&join_args(&outcome.solver_args)),
        sanitize_field(&outcome.solution_path),
        sanitize_field(&outcome.detail),
    )
}

fn write_header<W: Write>(writer: &mut W) -> Result<(), io::Error> {
    writeln!(
        writer,
        "group\tpath\tstatus\ttotal_ms\tsygus_ms\tsolve_ms\tsygus_bytes\tsygus_lines\tsolution_bytes\tdelis_exit_code\tsolver_exit_code\tdelis_args\tsolver\tsolver_args\tsolution_path\tdetail"
    )
}

fn load_completed_paths(results_path: &Path) -> Result<HashSet<String>, io::Error> {
    let mut completed = HashSet::new();
    if !results_path.exists() {
        return Ok(completed);
    }
    let text = fs::read_to_string(results_path)?;
    for line in text.lines() {
        if line.is_empty() || line.starts_with("group\tpath\t") {
            continue;
        }
        let mut cols = line.split('\t');
        let _group = cols.next();
        if let Some(path) = cols.next() {
            completed.insert(path.to_string());
        }
    }
    Ok(completed)
}

fn open_results_writer(results_path: &Path, rerun: bool) -> Result<BufWriter<File>, io::Error> {
    if let Some(parent) = results_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existed = results_path.exists();
    let file = if rerun {
        File::create(results_path)?
    } else {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(results_path)?
    };
    let mut writer = BufWriter::new(file);
    if rerun || !existed {
        write_header(&mut writer)?;
        writer.flush()?;
    }
    Ok(writer)
}

fn run_file_external_sygus(
    delis: &Path,
    manifest_dir: &Path,
    folder: &ResolvedFolder,
    file: &Path,
) -> Result<Outcome, io::Error> {
    let mut command_args = folder.delis_args.clone();
    command_args.push("--dump-sygus".to_string());
    command_args.push(file.display().to_string());

    let relative_path = relative_display(file, manifest_dir);
    let generation = run_command(delis, &command_args, None, folder.sygus_timeout)?;
    let make_outcome =
        |status: Status,
         sygus_duration: Duration,
         solve_duration: Duration,
         sygus_bytes: usize,
         sygus_lines: usize,
         solution_bytes: usize,
         delis_exit_code: Option<i32>,
         solver_exit_code: Option<i32>,
         solution_path: String,
         detail: String| Outcome {
            group: folder.name.clone(),
            path: relative_path.clone(),
            status,
            total_duration: sygus_duration + solve_duration,
            sygus_duration,
            solve_duration,
            sygus_bytes,
            sygus_lines,
            solution_bytes,
            delis_exit_code,
            solver_exit_code,
            delis_args: folder.delis_args.clone(),
            solver_cmd: folder.solver_cmd.clone(),
            solver_args: folder.solver_args.clone(),
            solution_path,
            detail,
        };

    match generation {
        CommandRun::TimedOut { duration, stderr, .. } => Ok(make_outcome(
            Status::SygusTimeout,
            duration,
            Duration::ZERO,
            0,
            0,
            0,
            None,
            None,
            String::new(),
            if stderr.is_empty() {
                format!("SyGuS generation timed out after {:.3}s", duration.as_secs_f64())
            } else {
                format!(
                    "SyGuS generation timed out after {:.3}s; {}",
                    duration.as_secs_f64(),
                    summarize_text(&stderr)
                )
            },
        )),
        CommandRun::Completed(generated) => {
            let sygus_bytes = generated.stdout.len();
            let sygus_lines = count_lines(&generated.stdout);
            if !generated.status.success() {
                return Ok(make_outcome(
                    Status::Error,
                    generated.duration,
                    Duration::ZERO,
                    sygus_bytes,
                    sygus_lines,
                    0,
                    generated.status.code(),
                    None,
                    String::new(),
                    if generated.stderr.is_empty() {
                        format!("delis exited with status {:?}", generated.status.code())
                    } else {
                        summarize_text(&generated.stderr)
                    },
                ));
            }

            let solver = PathBuf::from(&folder.solver_cmd);
            let solve = run_command(
                &solver,
                &folder.solver_args,
                Some(&generated.stdout),
                folder.solve_timeout,
            )?;
            match solve {
                CommandRun::TimedOut {
                    duration,
                    stdout,
                    stderr,
                } => Ok(make_outcome(
                    Status::SolveTimeout,
                    generated.duration,
                    duration,
                    sygus_bytes,
                    sygus_lines,
                    stdout.len(),
                    generated.status.code(),
                    None,
                    String::new(),
                    if stderr.is_empty() {
                        format!("solver timed out after {:.3}s", duration.as_secs_f64())
                    } else {
                        format!(
                            "solver timed out after {:.3}s; {}",
                            duration.as_secs_f64(),
                            summarize_text(&stderr)
                        )
                    },
                )),
                CommandRun::Completed(solved) => {
                    let stdout_text = String::from_utf8_lossy(&solved.stdout).trim().to_string();
                    let status = if solved.status.success() {
                        if stdout_text == "infeasible" {
                            Status::Infeasible
                        } else {
                            Status::Solved
                        }
                    } else {
                        Status::Error
                    };
                    let detail = if matches!(status, Status::Error) {
                        let stderr = summarize_text(&solved.stderr);
                        if stderr.is_empty() {
                            format!("solver exited with status {:?}", solved.status.code())
                        } else {
                            stderr
                        }
                    } else {
                        summarize_text(&solved.stderr)
                    };
                    let solution_path = if solved.status.success() && !solved.stdout.is_empty() {
                        let artifact = artifact_path(folder, file);
                        write_solution_artifact(&artifact, &solved.stdout)?;
                        relative_display(&artifact, manifest_dir)
                    } else {
                        String::new()
                    };
                    Ok(make_outcome(
                        status,
                        generated.duration,
                        solved.duration,
                        sygus_bytes,
                        sygus_lines,
                        solved.stdout.len(),
                        generated.status.code(),
                        solved.status.code(),
                        solution_path,
                        detail,
                    ))
                }
            }
        }
    }
}

fn run_file_native(
    delis: &Path,
    manifest_dir: &Path,
    folder: &ResolvedFolder,
    file: &Path,
) -> Result<Outcome, io::Error> {
    let mut command_args = folder.delis_args.clone();
    command_args.push(file.display().to_string());

    let relative_path = relative_display(file, manifest_dir);
    let run = run_command(delis, &command_args, None, folder.solve_timeout)?;
    let make_outcome =
        |status: Status,
         total_duration: Duration,
         solution_bytes: usize,
         delis_exit_code: Option<i32>,
         solution_path: String,
         detail: String| Outcome {
            group: folder.name.clone(),
            path: relative_path.clone(),
            status,
            total_duration,
            sygus_duration: Duration::ZERO,
            solve_duration: total_duration,
            sygus_bytes: 0,
            sygus_lines: 0,
            solution_bytes,
            delis_exit_code,
            solver_exit_code: None,
            delis_args: folder.delis_args.clone(),
            solver_cmd: String::new(),
            solver_args: Vec::new(),
            solution_path,
            detail,
        };

    match run {
        CommandRun::TimedOut { duration, stderr, .. } => Ok(make_outcome(
            Status::SolveTimeout,
            duration,
            0,
            None,
            String::new(),
            if stderr.is_empty() {
                format!("delis timed out after {:.3}s", duration.as_secs_f64())
            } else {
                format!(
                    "delis timed out after {:.3}s; {}",
                    duration.as_secs_f64(),
                    summarize_text(&stderr)
                )
            },
        )),
        CommandRun::Completed(result) => {
            let stdout_text = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let status = if result.status.success() {
                if stdout_text == "infeasible" {
                    Status::Infeasible
                } else {
                    Status::Solved
                }
            } else {
                Status::Error
            };
            let detail = if matches!(status, Status::Error) {
                let stderr = summarize_text(&result.stderr);
                if stderr.is_empty() {
                    format!("delis exited with status {:?}", result.status.code())
                } else {
                    stderr
                }
            } else {
                summarize_text(&result.stderr)
            };
            let solution_path = if matches!(status, Status::Solved | Status::Infeasible) {
                write_native_solution_artifact(folder, file, &result.stdout, manifest_dir)?
            } else {
                String::new()
            };
            Ok(make_outcome(
                status,
                result.duration,
                result.stdout.len(),
                result.status.code(),
                solution_path,
                detail,
            ))
        }
    }
}

fn run_file(
    delis: &Path,
    manifest_dir: &Path,
    folder: &ResolvedFolder,
    file: &Path,
) -> Result<Outcome, io::Error> {
    match folder.mode {
        BenchMode::Native => run_file_native(delis, manifest_dir, folder, file),
        BenchMode::ExternalSygus => run_file_external_sygus(delis, manifest_dir, folder, file),
    }
}

fn main() -> Result<(), io::Error> {
    ctrlc::set_handler(move || {
        cleanup_children();
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();
    let config = load_config(&cli.manifest)?;
    let manifest_dir = cli
        .manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let delis = resolve_delis_path(&cli.manifest, config.delis.as_ref())?;
    let folders = resolve_folders(&config, &cli.manifest)?;

    let mut stdout = BufWriter::new(io::stdout().lock());
    write_header(&mut stdout)?;
    stdout.flush()?;

    for folder in folders {
        let completed = if folder.rerun {
            HashSet::new()
        } else {
            load_completed_paths(&folder.results_path)?
        };
        let pending = folder
            .files
            .iter()
            .filter(|file| !completed.contains(&relative_display(file, &manifest_dir)))
            .count();
        eprintln!(
            "running folder {} -> {} ({} total, {} pending, rerun={}, mode={})",
            folder.path.display(),
            folder.results_path.display(),
            folder.files.len(),
            pending,
            folder.rerun,
            mode_name(folder.mode),
        );

        let mut writer = open_results_writer(&folder.results_path, folder.rerun)?;
        for file in &folder.files {
            let relative = relative_display(file, &manifest_dir);
            if completed.contains(&relative) {
                continue;
            }
            eprintln!("  running {}", relative);
            let outcome = run_file(&delis, &manifest_dir, &folder, file)?;
            let row = outcome_row(&outcome);
            writeln!(writer, "{row}")?;
            writer.flush()?;
            writeln!(stdout, "{row}")?;
            stdout.flush()?;
        }
    }

    Ok(())
}
