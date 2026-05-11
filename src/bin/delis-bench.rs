use clap::Parser;
use delis::process::{cleanup_children, register_pid, unregister_pid};
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Run Delis benchmarks from a TOML manifest or explicit file list"
)]
struct Cli {
    #[arg(value_name = "TOML", conflicts_with = "path")]
    manifest: Option<PathBuf>,

    #[arg(long = "path", value_name = "PATH", num_args = 1.., conflicts_with = "manifest")]
    path: Vec<PathBuf>,

    #[arg(long)]
    name: Option<String>,

    #[arg(long)]
    delis: Option<PathBuf>,

    #[arg(long, default_value = "native")]
    mode: BenchMode,

    #[arg(long = "delis-arg")]
    delis_args: Vec<String>,

    #[arg(long, default_value_t = false)]
    recursive: bool,

    #[arg(long)]
    sygus_solver: Option<String>,

    #[arg(long = "sygus-solver-arg")]
    sygus_solver_args: Vec<String>,

    #[arg(long, default_value_t = default_timeout_secs())]
    sygus_timeout_secs: u64,

    #[arg(long, default_value_t = default_timeout_secs())]
    solve_timeout_secs: u64,

    #[arg(long)]
    results_dir: Option<PathBuf>,

    #[arg(long)]
    results: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    rerun: bool,

    #[arg(long, default_value_t = false)]
    restart: bool,
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
    path_regex: Option<String>,
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
    #[serde(default)]
    restart: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, clap::ValueEnum)]
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
    restart: bool,
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

struct SpawnedCommand {
    child: Child,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

fn default_timeout_secs() -> u64 {
    600
}

fn load_config(path: &Path) -> Result<BenchmarkConfig, io::Error> {
    let text = fs::read_to_string(path)?;
    let config: BenchmarkConfig = toml::from_str(&text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid manifest: {e}"),
        )
    })?;
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

fn resolve_relative_from(base_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

fn resolve_program_path(base_file: &Path, program: &Path) -> PathBuf {
    if program.is_absolute() || program.components().count() == 1 {
        program.to_path_buf()
    } else {
        resolve_relative(base_file, program)
    }
}

fn resolve_program_path_from(base_dir: &Path, program: &Path) -> PathBuf {
    if program.is_absolute() || program.components().count() == 1 {
        program.to_path_buf()
    } else {
        resolve_relative_from(base_dir, program)
    }
}

fn resolve_delis_path(base_dir: &Path, configured: Option<&PathBuf>) -> Result<PathBuf, io::Error> {
    if let Some(path) = configured {
        return Ok(resolve_program_path_from(base_dir, path));
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
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(cmd);
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

fn filter_files_by_regex(
    root: &Path,
    files: Vec<PathBuf>,
    path_regex: Option<&str>,
) -> Result<Vec<PathBuf>, io::Error> {
    let Some(path_regex) = path_regex else {
        return Ok(files);
    };
    let regex = Regex::new(path_regex).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid path_regex {path_regex:?}: {e}"),
        )
    })?;
    let filtered = files
        .into_iter()
        .filter(|path| {
            let rel = path.strip_prefix(root).unwrap_or(path);
            regex.is_match(&rel.to_string_lossy())
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "path_regex {:?} matched no benchmark files under {}",
                path_regex,
                root.display()
            ),
        ));
    }
    Ok(filtered)
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

fn resolve_folders(
    config: &BenchmarkConfig,
    manifest: &Path,
) -> Result<Vec<ResolvedFolder>, io::Error> {
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
        let files = filter_files_by_regex(
            &path,
            list_benchmark_files(&path, folder.recursive)?,
            folder.path_regex.as_deref(),
        )?;
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
            restart: folder.restart,
        });
    }
    Ok(resolved)
}

fn common_parent(paths: &[PathBuf]) -> PathBuf {
    if paths.is_empty() {
        return PathBuf::from(".");
    }
    let mut current = paths[0]
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    for path in &paths[1..] {
        let candidate = path.parent().unwrap_or_else(|| Path::new("."));
        while !candidate.starts_with(&current) && !current.as_os_str().is_empty() {
            current.pop();
        }
        if current.as_os_str().is_empty() {
            return PathBuf::from(".");
        }
    }
    current
}

fn resolve_cli_folder(cli: &Cli, cwd: &Path) -> Result<ResolvedFolder, io::Error> {
    if cli.path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "either provide a manifest path or pass --path",
        ));
    }
    let mut files = Vec::with_capacity(cli.path.len());
    for path in &cli.path {
        let resolved = resolve_relative_from(cwd, path);
        if resolved.is_dir() {
            files.extend(list_benchmark_files(&resolved, cli.recursive)?);
            continue;
        }
        if resolved.extension().and_then(|ext| ext.to_str()) != Some("sl") || !resolved.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "path {} is neither an .sl file nor a benchmark folder",
                    resolved.display()
                ),
            ));
        }
        files.push(resolved);
    }
    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no benchmark files found from --path",
        ));
    }
    files.sort();
    files.dedup();

    let name = cli.name.clone().unwrap_or_else(|| "ad-hoc".to_string());
    let results_root = cli
        .results_dir
        .as_deref()
        .map(|path| resolve_relative_from(cwd, path))
        .unwrap_or_else(|| cwd.join("bench-results"));
    let results_path = cli
        .results
        .as_deref()
        .map(|path| resolve_relative_from(cwd, path))
        .unwrap_or_else(|| results_root.join(format!("{}.tsv", slugify(&name))));
    let artifacts_dir = results_root.join("artifacts").join(slugify(&name));

    let (solver_cmd, solver_args) = if cli.mode == BenchMode::ExternalSygus {
        let solver_cmd = cli
            .sygus_solver
            .clone()
            .or_else(|| env::var("DELIS_BENCH_SYGUS_SOLVER").ok())
            .unwrap_or_else(|| "cvc5".to_string());
        let solver_cmd = resolve_program_path_from(cwd, Path::new(&solver_cmd))
            .display()
            .to_string();
        let solver_args = if !cli.sygus_solver_args.is_empty() {
            cli.sygus_solver_args.clone()
        } else {
            default_solver_args(&solver_cmd)
        };
        (solver_cmd, solver_args)
    } else {
        (String::new(), Vec::new())
    };

    Ok(ResolvedFolder {
        name,
        path: common_parent(&files),
        mode: cli.mode,
        delis_args: cli.delis_args.clone(),
        sygus_timeout: Duration::from_secs(cli.sygus_timeout_secs),
        solve_timeout: Duration::from_secs(cli.solve_timeout_secs),
        solver_cmd,
        solver_args,
        files,
        results_path,
        artifacts_dir,
        rerun: cli.rerun,
        restart: cli.restart,
    })
}

fn spawn_command(
    program: &Path,
    args: &[String],
    stdin_bytes: Option<&[u8]>,
) -> Result<SpawnedCommand, io::Error> {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let nonce = format!(
        "delis-bench-{}-{}",
        std::process::id(),
        timestamp_nanos
    );
    let stdout_path = env::temp_dir().join(format!("{nonce}.stdout"));
    let stderr_path = env::temp_dir().join(format!("{nonce}.stderr"));
    let stdout_file = File::create(&stdout_path)?;
    let stderr_file = File::create(&stderr_path)?;

    let mut child = std::process::Command::new(program)
        .args(args)
        .stdin(if stdin_bytes.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
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
    Ok(SpawnedCommand {
        child,
        stdout_path,
        stderr_path,
    })
}

fn read_command_output(path: &Path) -> Vec<u8> {
    let bytes = fs::read(path).unwrap_or_default();
    let _ = fs::remove_file(path);
    bytes
}

fn run_command(
    program: &Path,
    args: &[String],
    stdin_bytes: Option<&[u8]>,
    timeout: Duration,
) -> Result<CommandRun, io::Error> {
    let SpawnedCommand {
        mut child,
        stdout_path,
        stderr_path,
    } = spawn_command(program, args, stdin_bytes)?;
    let pid = child.id();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            unregister_pid(pid);
            let _ = child.wait()?;
            return Ok(CommandRun::Completed(CommandResult {
                status,
                duration: start.elapsed(),
                stdout: read_command_output(&stdout_path),
                stderr: read_command_output(&stderr_path),
            }));
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            unregister_pid(pid);
            let _ = child.wait();
            return Ok(CommandRun::TimedOut {
                duration: start.elapsed(),
                stdout: read_command_output(&stdout_path),
                stderr: read_command_output(&stderr_path),
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

fn row_path(row: &str) -> Option<&str> {
    let mut cols = row.split('\t');
    let _group = cols.next();
    cols.next()
}

#[derive(Debug, Default)]
struct ResultsTable {
    order: Vec<String>,
    rows: HashMap<String, String>,
}

fn load_results_table(results_path: &Path) -> Result<ResultsTable, io::Error> {
    let mut table = ResultsTable::default();
    if !results_path.exists() {
        return Ok(table);
    }
    let text = fs::read_to_string(results_path)?;
    for line in text.lines() {
        if line.is_empty() || line.starts_with("group\tpath\t") {
            continue;
        }
        if let Some(path) = row_path(line) {
            let path = path.to_string();
            if !table.rows.contains_key(&path) {
                table.order.push(path.clone());
            }
            table.rows.insert(path, line.to_string());
        }
    }
    Ok(table)
}

fn progress_path(results_path: &Path) -> PathBuf {
    results_path.with_extension("progress")
}

fn load_progress_marker(progress_path: &Path) -> Result<Option<String>, io::Error> {
    if !progress_path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(progress_path)?;
    let path = text.lines().next().map(str::trim).unwrap_or_default();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path.to_string()))
    }
}

fn write_progress_marker(progress_path: &Path, path: &str) -> Result<(), io::Error> {
    if let Some(parent) = progress_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(progress_path, format!("{path}\n"))
}

fn clear_progress_marker(progress_path: &Path) -> Result<(), io::Error> {
    match fs::remove_file(progress_path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn write_results_table(results_path: &Path, table: &ResultsTable) -> Result<(), io::Error> {
    if let Some(parent) = results_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = results_path.with_extension("tsv.tmp");
    let mut writer = BufWriter::new(File::create(&tmp_path)?);
    write_header(&mut writer)?;
    for path in &table.order {
        if let Some(row) = table.rows.get(path) {
            writeln!(writer, "{row}")?;
        }
    }
    writer.flush()?;
    fs::rename(tmp_path, results_path)?;
    Ok(())
}

fn open_results_writer(results_path: &Path) -> Result<BufWriter<File>, io::Error> {
    if let Some(parent) = results_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existed = results_path.exists();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_path)?;
    let mut writer = BufWriter::new(file);
    if !existed {
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
    let make_outcome = |status: Status,
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
        CommandRun::TimedOut {
            duration, stderr, ..
        } => Ok(make_outcome(
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
                format!(
                    "SyGuS generation timed out after {:.3}s",
                    duration.as_secs_f64()
                )
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
    let make_outcome = |status: Status,
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
        CommandRun::TimedOut {
            duration, stderr, ..
        } => Ok(make_outcome(
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

fn run_folders(
    delis: &Path,
    manifest_dir: &Path,
    folders: Vec<ResolvedFolder>,
) -> Result<(), io::Error> {
    let mut stdout = BufWriter::new(io::stdout().lock());
    write_header(&mut stdout)?;
    stdout.flush()?;

    for folder in folders {
        let folder_paths = folder
            .files
            .iter()
            .map(|file| relative_display(file, manifest_dir))
            .collect::<HashSet<_>>();
        let manage_results_table = folder.rerun || folder.restart;
        let mut results_table = if manage_results_table {
            Some(load_results_table(&folder.results_path)?)
        } else {
            None
        };
        let completed = if folder.restart {
            HashSet::new()
        } else if let Some(table) = &results_table {
            table.rows
                .keys()
                .filter(|path| folder_paths.contains(path.as_str()))
                .cloned()
                .collect::<HashSet<_>>()
        } else {
            load_completed_paths(&folder.results_path)?
                .into_iter()
                .filter(|path| folder_paths.contains(path.as_str()))
                .collect()
        };
        let progress = progress_path(&folder.results_path);
        if let Some(path) = load_progress_marker(&progress)? {
            if folder_paths.contains(path.as_str()) && !completed.contains(&path) {
                eprintln!(
                    "resuming folder {} after interruption; last in-progress file was {}",
                    folder.path.display(),
                    path
                );
            } else {
                clear_progress_marker(&progress)?;
            }
        }
        let pending = folder
            .files
            .iter()
            .filter(|file| !completed.contains(&relative_display(file, manifest_dir)))
            .count();
        eprintln!(
            "running folder {} -> {} ({} total, {} pending, rerun={}, restart={}, mode={})",
            folder.path.display(),
            folder.results_path.display(),
            folder.files.len(),
            pending,
            folder.rerun,
            folder.restart,
            mode_name(folder.mode),
        );

        let mut writer = if manage_results_table {
            None
        } else {
            Some(open_results_writer(&folder.results_path)?)
        };
        for file in &folder.files {
            let relative = relative_display(file, manifest_dir);
            if completed.contains(&relative) {
                continue;
            }
            eprintln!("  running {}", relative);
            write_progress_marker(&progress, &relative)?;
            let outcome = run_file(delis, manifest_dir, &folder, file)?;
            let row = outcome_row(&outcome);
            if let Some(table) = results_table.as_mut() {
                if !table.rows.contains_key(&relative) {
                    table.order.push(relative.clone());
                }
                table.rows.insert(relative.clone(), row.clone());
                write_results_table(&folder.results_path, table)?;
            } else if let Some(writer) = writer.as_mut() {
                writeln!(writer, "{row}")?;
                writer.flush()?;
            }
            writeln!(stdout, "{row}")?;
            stdout.flush()?;
            clear_progress_marker(&progress)?;
        }
        clear_progress_marker(&progress)?;
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    ctrlc::set_handler(move || {
        cleanup_children();
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();
    if let Some(manifest) = cli.manifest.as_ref() {
        let config = load_config(manifest)?;
        let manifest_dir = manifest
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let delis = resolve_delis_path(&manifest_dir, config.delis.as_ref())?;
        let folders = resolve_folders(&config, manifest)?;
        run_folders(&delis, &manifest_dir, folders)?;
    } else if !cli.path.is_empty() {
        let cwd = env::current_dir()?;
        let folder = resolve_cli_folder(&cli, &cwd)?;
        let delis = resolve_delis_path(&cwd, cli.delis.as_ref())?;
        run_folders(&delis, &cwd, vec![folder])?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "either provide a manifest path or pass --path",
        ));
    }

    Ok(())
}
