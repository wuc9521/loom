use clap::Parser;
use delis::process::{cleanup_children, register_pid, unregister_pid};
use serde::Deserialize;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
    solver: Option<PathBuf>,
    #[serde(default = "default_timeout_secs")]
    timeout_secs: u64,
    #[serde(default)]
    cases: Vec<BenchmarkCase>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkCase {
    name: Option<String>,
    path: PathBuf,
    #[serde(default)]
    args: Vec<String>,
    timeout_secs: Option<u64>,
}

#[derive(Debug)]
enum Status {
    Solved,
    Infeasible,
    Timeout,
    Error,
}

#[derive(Debug)]
struct Outcome {
    name: String,
    path: PathBuf,
    status: Status,
    duration: Duration,
    detail: String,
}

fn default_timeout_secs() -> u64 {
    5
}

fn load_config(path: &Path) -> Result<BenchmarkConfig, io::Error> {
    let text = fs::read_to_string(path)?;
    let config: BenchmarkConfig = toml::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid manifest: {e}")))?;
    if config.cases.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "manifest has no benchmark cases",
        ));
    }
    Ok(config)
}

fn resolve_solver_path(manifest: &Path, solver: Option<&PathBuf>) -> Result<PathBuf, io::Error> {
    if let Some(path) = solver {
        return Ok(resolve_relative(manifest, path));
    }
    if let Ok(path) = env::var("DELIS_BENCH_SOLVER") {
        return Ok(PathBuf::from(path));
    }
    let exe = env::current_exe()?;
    let sibling = exe
        .parent()
        .map(|dir| dir.join("delis"))
        .ok_or_else(|| io::Error::other("cannot infer solver path"))?;
    Ok(sibling)
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

fn run_case(solver: &Path, manifest: &Path, case: &BenchmarkCase, timeout: Duration) -> Result<Outcome, io::Error> {
    let path = resolve_relative(manifest, &case.path);
    let name = case.name.clone().unwrap_or_else(|| path.display().to_string());
    let mut child = Command::new(solver)
        .arg(&path)
        .args(&case.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let pid = child.id();
    register_pid(pid);
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            unregister_pid(pid);
            let output = child.wait_with_output()?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let detail = if !stdout.is_empty() { stdout } else { stderr.clone() };
            let status = if status.success() {
                if detail == "infeasible" {
                    Status::Infeasible
                } else {
                    Status::Solved
                }
            } else {
                Status::Error
            };
            let detail = if detail.is_empty() {
                format!("process exited with status {status:?}")
            } else {
                detail
            };
            return Ok(Outcome {
                name,
                path,
                status,
                duration: start.elapsed(),
                detail,
            });
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            unregister_pid(pid);
            let output = child.wait_with_output()?;
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let detail = if stderr.is_empty() {
                format!("timed out after {:.3}s", timeout.as_secs_f64())
            } else {
                format!("timed out after {:.3}s; stderr: {stderr}", timeout.as_secs_f64())
            };
            return Ok(Outcome {
                name,
                path,
                status: Status::Timeout,
                duration: start.elapsed(),
                detail,
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn print_summary(outcomes: &[Outcome]) {
    println!("status\tduration_ms\tname\tpath\tdetail");
    for outcome in outcomes {
        let status = match outcome.status {
            Status::Solved => "solved",
            Status::Infeasible => "infeasible",
            Status::Timeout => "timeout",
            Status::Error => "error",
        };
        println!(
            "{status}\t{}\t{}\t{}\t{}",
            outcome.duration.as_millis(),
            outcome.name,
            outcome.path.display(),
            outcome.detail.replace('\n', " ")
        );
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
    let solver = resolve_solver_path(&cli.manifest, config.solver.as_ref())?;
    let mut outcomes = Vec::with_capacity(config.cases.len());
    for case in &config.cases {
        let timeout = Duration::from_secs(case.timeout_secs.unwrap_or(config.timeout_secs));
        outcomes.push(run_case(&solver, &cli.manifest, case, timeout)?);
    }
    print_summary(&outcomes);
    Ok(())
}
