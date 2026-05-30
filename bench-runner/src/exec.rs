use std::{
    fs::{self, File},
    io,
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    model::{
        ArtifactPaths, CommandOutcome, DumpStats, ExternalCommandMetadata, JobRecord, JobSpec,
        SuiteMode,
    },
    util::{
        blank_to_none, combine_detail, count_lines, display_relative_path, file_size,
        make_relative_string, read_text_file, safe_relative_path, sanitize_field,
        suffix_with_summary, summarize_text_file, temp_file_base, unix_now_secs,
    },
};

pub fn worker_loop(
    queue: Arc<Mutex<std::collections::VecDeque<JobSpec>>>,
    tx: mpsc::Sender<JobRecord>,
    stop: Arc<AtomicBool>,
    run_dir: std::path::PathBuf,
    run_id: String,
) {
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let job = match queue.lock() {
            Ok(mut guard) => guard.pop_front(),
            Err(_) => return,
        };
        let Some(job) = job else {
            return;
        };
        let record = match execute_job(&run_dir, &run_id, &job) {
            Ok(record) => record,
            Err(error) => JobRecord {
                run_id: run_id.clone(),
                suite_name: job.suite_name.clone(),
                suite_mode: job.suite_mode,
                display_path: job.display_path.clone(),
                order_key: job.order_key.clone(),
                status: "driver_error".to_string(),
                total_ms: 0,
                sygus_ms: 0,
                solve_ms: 0,
                sygus_bytes: 0,
                sygus_lines: 0,
                solution_bytes: 0,
                delis_exit_code: None,
                solver_exit_code: None,
                delis_args: job.delis_args.clone(),
                solver: job_solver_name(&job),
                solver_args: job_solver_args(&job),
                solution_path: None,
                stdout_path: None,
                stderr_path: None,
                sygus_path: None,
                detail: error.to_string(),
                completed_at_utc: unix_now_secs().to_string(),
            },
        };
        if tx.send(record).is_err() {
            return;
        }
    }
}

pub fn execute_job(
    run_dir: &Path,
    run_id: &str,
    job: &JobSpec,
) -> Result<JobRecord, Box<dyn std::error::Error>> {
    let artifacts = allocate_artifacts(run_dir, job)?;
    match job.suite_mode {
        SuiteMode::Native => execute_native_job(run_id, job, artifacts),
        SuiteMode::ExternalSygus => execute_external_sygus_job(run_id, job, artifacts),
        SuiteMode::ExternalCommand => execute_external_command_job(run_dir, run_id, job, artifacts),
    }
}

fn execute_native_job(
    run_id: &str,
    job: &JobSpec,
    artifacts: ArtifactPaths,
) -> Result<JobRecord, Box<dyn std::error::Error>> {
    let mut stats = DumpStats {
        sygus_ms: 0,
        sygus_bytes: 0,
        sygus_lines: 0,
        detail_prefix: None,
    };
    if job.capture_sygus_stats {
        stats = run_sygus_stats(job)?;
    }

    let mut command = Command::new(&job.delis);
    command.args(&job.delis_args);
    command.arg(&job.file_path);
    let outcome = run_command_with_output(
        &mut command,
        None,
        &artifacts.solver_stdout_abs,
        &artifacts.driver_stderr_abs,
        Duration::from_secs(job.solve_timeout_secs),
    )?;

    let stdout_text = read_text_file(&artifacts.solver_stdout_abs)?;
    let stderr_summary = summarize_text_file(&artifacts.driver_stderr_abs)?;
    let solution_bytes = file_size(&artifacts.solver_stdout_abs)?;
    let status;
    let detail;
    let delis_exit_code;
    let solution_path;
    if outcome.timed_out {
        status = "solve_timeout".to_string();
        detail = combine_detail(
            stats.detail_prefix.clone(),
            Some(format!(
                "delis timed out after {:.3}s{}",
                outcome.duration_ms as f64 / 1000.0,
                suffix_with_summary(&stderr_summary)
            )),
        );
        delis_exit_code = None;
        solution_path = None;
    } else if outcome.exit_code == Some(0) {
        status = if stdout_text.trim() == "infeasible" {
            "infeasible".to_string()
        } else {
            "solved".to_string()
        };
        detail = combine_detail(stats.detail_prefix.clone(), blank_to_none(stderr_summary));
        delis_exit_code = outcome.exit_code;
        solution_path = artifacts.solution_rel.clone();
        if let Some(solution_abs) = &artifacts.solution_abs {
            fs::copy(&artifacts.solver_stdout_abs, solution_abs)?;
        }
    } else {
        status = "error".to_string();
        detail = combine_detail(
            stats.detail_prefix.clone(),
            Some(if stderr_summary.is_empty() {
                format!(
                    "delis exited with status {}",
                    outcome.exit_code.unwrap_or_default()
                )
            } else {
                stderr_summary
            }),
        );
        delis_exit_code = outcome.exit_code;
        solution_path = None;
    }

    Ok(JobRecord {
        run_id: run_id.to_string(),
        suite_name: job.suite_name.clone(),
        suite_mode: job.suite_mode,
        display_path: job.display_path.clone(),
        order_key: job.order_key.clone(),
        status,
        total_ms: stats.sygus_ms + outcome.duration_ms,
        sygus_ms: stats.sygus_ms,
        solve_ms: outcome.duration_ms,
        sygus_bytes: stats.sygus_bytes,
        sygus_lines: stats.sygus_lines,
        solution_bytes,
        delis_exit_code,
        solver_exit_code: None,
        delis_args: job.delis_args.clone(),
        solver: None,
        solver_args: Vec::new(),
        solution_path,
        stdout_path: Some(artifacts.solver_stdout_rel),
        stderr_path: Some(artifacts.driver_stderr_rel),
        sygus_path: None,
        detail,
        completed_at_utc: unix_now_secs().to_string(),
    })
}

fn execute_external_sygus_job(
    run_id: &str,
    job: &JobSpec,
    artifacts: ArtifactPaths,
) -> Result<JobRecord, Box<dyn std::error::Error>> {
    let solver = job
        .sygus_solver
        .as_ref()
        .ok_or("external_sygus mode requires `sygus_solver`")?;
    let persistent_sygus_abs = persistent_sygus_path(job)
        .or_else(|| artifacts.sygus_abs.clone())
        .ok_or("missing SyGuS artifact path")?;
    if let Some(parent) = persistent_sygus_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut dump_command = Command::new(&job.delis);
    dump_command.args(&job.delis_args);
    dump_command.arg("--dump-sygus");
    dump_command.arg(&job.file_path);
    let dump_outcome = run_command_with_output(
        &mut dump_command,
        None,
        &persistent_sygus_abs,
        &artifacts.driver_stderr_abs,
        Duration::from_secs(job.sygus_timeout_secs),
    )?;
    let dump_stderr = summarize_text_file(&artifacts.driver_stderr_abs)?;
    let sygus_rel = persistent_sygus_rel(run_id, job, &persistent_sygus_abs, &artifacts);
    if dump_outcome.timed_out {
        return Ok(JobRecord {
            run_id: run_id.to_string(),
            suite_name: job.suite_name.clone(),
            suite_mode: job.suite_mode,
            display_path: job.display_path.clone(),
            order_key: job.order_key.clone(),
            status: "sygus_timeout".to_string(),
            total_ms: 0,
            sygus_ms: dump_outcome.duration_ms,
            solve_ms: 0,
            sygus_bytes: 0,
            sygus_lines: 0,
            solution_bytes: 0,
            delis_exit_code: None,
            solver_exit_code: None,
            delis_args: job.delis_args.clone(),
            solver: Some(solver.to_string_lossy().into_owned()),
            solver_args: job.sygus_solver_args.clone(),
            solution_path: None,
            stdout_path: None,
            stderr_path: Some(artifacts.driver_stderr_rel),
            sygus_path: sygus_rel.clone(),
            detail: format!(
                "SyGuS dump timed out after {:.3}s{}",
                dump_outcome.duration_ms as f64 / 1000.0,
                suffix_with_summary(&dump_stderr)
            ),
            completed_at_utc: unix_now_secs().to_string(),
        });
    }
    if dump_outcome.exit_code != Some(0) {
        return Ok(JobRecord {
            run_id: run_id.to_string(),
            suite_name: job.suite_name.clone(),
            suite_mode: job.suite_mode,
            display_path: job.display_path.clone(),
            order_key: job.order_key.clone(),
            status: "error".to_string(),
            total_ms: 0,
            sygus_ms: dump_outcome.duration_ms,
            solve_ms: 0,
            sygus_bytes: 0,
            sygus_lines: 0,
            solution_bytes: 0,
            delis_exit_code: dump_outcome.exit_code,
            solver_exit_code: None,
            delis_args: job.delis_args.clone(),
            solver: Some(solver.to_string_lossy().into_owned()),
            solver_args: job.sygus_solver_args.clone(),
            solution_path: None,
            stdout_path: None,
            stderr_path: Some(artifacts.driver_stderr_rel),
            sygus_path: sygus_rel.clone(),
            detail: if dump_stderr.is_empty() {
                format!(
                    "delis --dump-sygus exited with status {}",
                    dump_outcome.exit_code.unwrap_or_default()
                )
            } else {
                dump_stderr
            },
            completed_at_utc: unix_now_secs().to_string(),
        });
    }

    let sygus_bytes = file_size(&persistent_sygus_abs)?;
    let sygus_lines = count_lines(&persistent_sygus_abs)?;
    let solution_abs = artifacts
        .solution_abs
        .as_ref()
        .ok_or("missing solution artifact path")?;
    let mut solver_command = Command::new(solver);
    solver_command.args(&job.sygus_solver_args);
    solver_command.arg(&persistent_sygus_abs);
    let solver_outcome = run_command_with_output(
        &mut solver_command,
        None,
        solution_abs,
        &artifacts.solver_stdout_abs,
        Duration::from_secs(job.solve_timeout_secs),
    )?;
    let solver_stderr = summarize_text_file(&artifacts.solver_stdout_abs)?;
    let solution_text = read_text_file(solution_abs)?;
    let solution_bytes = file_size(solution_abs)?;
    let status;
    let detail;
    let solver_exit_code;
    let solution_path;
    if solver_outcome.timed_out {
        status = "solve_timeout".to_string();
        detail = format!(
            "{} timed out after {:.3}s{}",
            solver.display(),
            solver_outcome.duration_ms as f64 / 1000.0,
            suffix_with_summary(&solver_stderr)
        );
        solver_exit_code = None;
        solution_path = None;
    } else if solver_outcome.exit_code == Some(0) {
        status = if solution_text.trim() == "infeasible" {
            "infeasible".to_string()
        } else {
            "solved".to_string()
        };
        detail = solver_stderr;
        solver_exit_code = solver_outcome.exit_code;
        solution_path = artifacts.solution_rel.clone();
    } else {
        status = "error".to_string();
        detail = if solver_stderr.is_empty() {
            format!(
                "{} exited with status {}",
                solver.display(),
                solver_outcome.exit_code.unwrap_or_default()
            )
        } else {
            solver_stderr
        };
        solver_exit_code = solver_outcome.exit_code;
        solution_path = None;
    }

    Ok(JobRecord {
        run_id: run_id.to_string(),
        suite_name: job.suite_name.clone(),
        suite_mode: job.suite_mode,
        display_path: job.display_path.clone(),
        order_key: job.order_key.clone(),
        status,
        total_ms: solver_outcome.duration_ms,
        sygus_ms: dump_outcome.duration_ms,
        solve_ms: solver_outcome.duration_ms,
        sygus_bytes,
        sygus_lines,
        solution_bytes,
        delis_exit_code: dump_outcome.exit_code,
        solver_exit_code,
        delis_args: job.delis_args.clone(),
        solver: Some(solver.to_string_lossy().into_owned()),
        solver_args: job.sygus_solver_args.clone(),
        solution_path,
        stdout_path: Some(artifacts.solver_stdout_rel),
        stderr_path: Some(artifacts.driver_stderr_rel),
        sygus_path: sygus_rel,
        detail,
        completed_at_utc: unix_now_secs().to_string(),
    })
}

fn execute_external_command_job(
    run_dir: &Path,
    run_id: &str,
    job: &JobSpec,
    artifacts: ArtifactPaths,
) -> Result<JobRecord, Box<dyn std::error::Error>> {
    let command_path = job
        .external_command
        .as_ref()
        .ok_or("external_command mode requires `external_command`")?;
    fs::create_dir_all(&artifacts.artifact_dir_abs)?;

    let input_name = job
        .file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let input_stem = job
        .file_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let input_dir = job.file_path.parent().unwrap_or(Path::new("."));
    let solution_abs = artifacts
        .solution_abs
        .as_ref()
        .ok_or("missing external solution path")?;

    let mut command = Command::new(command_path);
    for arg in &job.external_command_args {
        command.arg(expand_external_arg(
            arg,
            &job.file_path,
            input_dir,
            &input_name,
            &input_stem,
            &job.suite_name,
            &artifacts.artifact_dir_abs,
            &artifacts.command_result_abs,
            solution_abs,
        ));
    }
    if let Some(working_dir) = &job.working_dir {
        command.current_dir(working_dir);
    }
    command.env("DELIS_BENCH_RUN_ID", run_id);
    command.env("DELIS_BENCH_SUITE", &job.suite_name);
    command.env("DELIS_BENCH_INPUT", &job.file_path);
    command.env("DELIS_BENCH_INPUT_DIR", input_dir);
    command.env("DELIS_BENCH_INPUT_NAME", &input_name);
    command.env("DELIS_BENCH_INPUT_STEM", &input_stem);
    command.env("DELIS_BENCH_ARTIFACT_DIR", &artifacts.artifact_dir_abs);
    command.env("DELIS_BENCH_RESULT_JSON", &artifacts.command_result_abs);
    command.env("DELIS_BENCH_SOLUTION_PATH", solution_abs);

    let outcome = run_command_with_output(
        &mut command,
        None,
        &artifacts.solver_stdout_abs,
        &artifacts.driver_stderr_abs,
        Duration::from_secs(job.solve_timeout_secs),
    )?;
    let stdout_summary = summarize_text_file(&artifacts.solver_stdout_abs)?;
    let stderr_summary = summarize_text_file(&artifacts.driver_stderr_abs)?;
    let metadata = load_external_command_metadata(&artifacts.command_result_abs)?;
    let solution_exists = solution_abs.is_file();
    let solution_bytes = match &metadata {
        Some(meta) => meta
            .solution_bytes
            .or_else(|| {
                if solution_exists {
                    file_size(solution_abs).ok()
                } else {
                    None
                }
            })
            .unwrap_or(0),
        None => {
            if solution_exists {
                file_size(solution_abs)?
            } else {
                0
            }
        }
    };

    let status = if outcome.timed_out {
        "solve_timeout".to_string()
    } else if let Some(meta) = &metadata {
        meta.status.clone().unwrap_or_else(|| {
            if outcome.exit_code == Some(0) {
                "solved".to_string()
            } else {
                "error".to_string()
            }
        })
    } else if outcome.exit_code == Some(0) {
        "solved".to_string()
    } else {
        "error".to_string()
    };

    let detail = if outcome.timed_out {
        combine_detail(
            blank_to_none(format!(
                "{} timed out after {:.3}s",
                command_path.display(),
                outcome.duration_ms as f64 / 1000.0
            )),
            combine_output_summaries(stdout_summary, stderr_summary),
        )
    } else if let Some(meta) = &metadata {
        combine_detail(
            meta.detail.clone(),
            combine_output_summaries(stdout_summary, stderr_summary),
        )
    } else {
        combine_detail(
            blank_to_none(if outcome.exit_code == Some(0) {
                String::new()
            } else {
                format!(
                    "{} exited with status {}",
                    command_path.display(),
                    outcome.exit_code.unwrap_or_default()
                )
            }),
            combine_output_summaries(stdout_summary, stderr_summary),
        )
    };

    let solution_path = if solution_exists {
        Some(sanitize_field(&make_relative_string(solution_abs, run_dir)))
    } else if let Some(meta) = &metadata {
        meta.solution_path.clone()
    } else {
        None
    };

    Ok(JobRecord {
        run_id: run_id.to_string(),
        suite_name: job.suite_name.clone(),
        suite_mode: job.suite_mode,
        display_path: job.display_path.clone(),
        order_key: job.order_key.clone(),
        status,
        total_ms: metadata
            .as_ref()
            .and_then(|meta| meta.total_ms)
            .unwrap_or(outcome.duration_ms),
        sygus_ms: 0,
        solve_ms: metadata
            .as_ref()
            .and_then(|meta| meta.solve_ms)
            .unwrap_or(outcome.duration_ms),
        sygus_bytes: 0,
        sygus_lines: 0,
        solution_bytes,
        delis_exit_code: None,
        solver_exit_code: if outcome.timed_out {
            None
        } else {
            metadata
                .as_ref()
                .and_then(|meta| meta.solver_exit_code)
                .or(outcome.exit_code)
        },
        delis_args: job.delis_args.clone(),
        solver: Some(command_path.to_string_lossy().into_owned()),
        solver_args: job.external_command_args.clone(),
        solution_path,
        stdout_path: Some(artifacts.solver_stdout_rel),
        stderr_path: Some(artifacts.driver_stderr_rel),
        sygus_path: None,
        detail,
        completed_at_utc: unix_now_secs().to_string(),
    })
}

fn run_sygus_stats(job: &JobSpec) -> Result<DumpStats, Box<dyn std::error::Error>> {
    let base = temp_file_base("sygus-stats");
    let stdout_path = base.with_extension("stdout");
    let stderr_path = base.with_extension("stderr");
    let mut command = Command::new(&job.delis);
    command.args(&job.delis_args);
    command.arg("--sygus-stats");
    command.arg(&job.file_path);
    let outcome = run_command_with_output(
        &mut command,
        None,
        &stdout_path,
        &stderr_path,
        Duration::from_secs(job.sygus_timeout_secs),
    )?;
    let stderr_summary = summarize_text_file(&stderr_path)?;
    let stdout = read_text_file(&stdout_path)?;
    let detail_prefix = if outcome.timed_out {
        Some("sygus-stats timed out".to_string())
    } else if outcome.exit_code != Some(0) {
        Some(if stderr_summary.is_empty() {
            "sygus-stats failed".to_string()
        } else {
            stderr_summary.clone()
        })
    } else {
        None
    };
    let stats = parse_sygus_stats_line(stdout.lines().next());
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);
    Ok(DumpStats {
        sygus_ms: stats.map(|(ms, _, _)| ms).unwrap_or(0),
        sygus_bytes: stats.map(|(_, bytes, _)| bytes).unwrap_or(0),
        sygus_lines: stats.map(|(_, _, lines)| lines).unwrap_or(0),
        detail_prefix,
    })
}

fn parse_sygus_stats_line(line: Option<&str>) -> Option<(u64, u64, u64)> {
    let line = line?;
    let parts = line.split('\t').collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != "SYGUS_STATS" {
        return None;
    }
    Some((
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
        parts[3].parse().ok()?,
    ))
}

fn run_command_with_output(
    command: &mut Command,
    stdin_path: Option<&Path>,
    stdout_path: &Path,
    stderr_path: &Path,
    timeout: Duration,
) -> Result<CommandOutcome, Box<dyn std::error::Error>> {
    if let Some(parent) = stdout_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = stderr_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stdout = File::create(stdout_path)?;
    let stderr = File::create(stderr_path)?;
    if let Some(stdin_path) = stdin_path {
        command.stdin(Stdio::from(File::open(stdin_path)?));
    } else {
        command.stdin(Stdio::null());
    }
    command.stdout(Stdio::from(stdout));
    command.stderr(Stdio::from(stderr));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    let start = Instant::now();
    let mut child = command.spawn()?;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(CommandOutcome {
                timed_out: false,
                exit_code: status.code(),
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }
        if start.elapsed() >= timeout {
            #[cfg(unix)]
            unsafe {
                libc::killpg(child.id() as i32, libc::SIGKILL);
            }
            let _ = child.kill();
            let status = child.wait()?;
            return Ok(CommandOutcome {
                timed_out: true,
                exit_code: status.code(),
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn allocate_artifacts(
    run_dir: &Path,
    job: &JobSpec,
) -> Result<ArtifactPaths, Box<dyn std::error::Error>> {
    let relative = safe_relative_path(&job.display_path);
    let artifact_base = run_dir
        .join("artifacts")
        .join(&job.suite_name)
        .join(&relative);
    let log_base = run_dir.join("logs").join(&job.suite_name).join(&relative);
    let artifact_dir_abs = artifact_base.with_extension("work");
    if let Some(parent) = artifact_base.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = log_base.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(&artifact_dir_abs)?;
    let solver_stdout_abs = log_base.with_extension("solver-stderr.txt");
    let driver_stderr_abs = log_base.with_extension("driver-stderr.txt");
    let solution_abs = artifact_base.with_extension("solution.txt");
    let command_result_abs = artifact_base.with_extension("result.json");
    let sygus_abs = if job.suite_mode == SuiteMode::ExternalSygus {
        Some(artifact_base.with_extension("sygus2"))
    } else {
        None
    };
    Ok(ArtifactPaths {
        artifact_dir_abs,
        command_result_abs,
        solver_stdout_rel: sanitize_field(&make_relative_string(&solver_stdout_abs, run_dir)),
        driver_stderr_rel: sanitize_field(&make_relative_string(&driver_stderr_abs, run_dir)),
        solver_stdout_abs,
        driver_stderr_abs,
        solution_rel: Some(sanitize_field(&make_relative_string(
            &solution_abs,
            run_dir,
        ))),
        solution_abs: Some(solution_abs),
        sygus_rel: sygus_abs
            .as_ref()
            .map(|path| sanitize_field(&make_relative_string(path, run_dir))),
        sygus_abs,
    })
}

fn persistent_sygus_path(job: &JobSpec) -> Option<std::path::PathBuf> {
    let root = job.persistent_sygus_dir.as_ref()?;
    Some(root.join(display_relative_path(&job.display_path)))
}

fn persistent_sygus_rel(
    run_id: &str,
    job: &JobSpec,
    path: &Path,
    artifacts: &ArtifactPaths,
) -> Option<String> {
    if let Some(root) = &job.persistent_sygus_dir {
        if let Some(base) = root
            .ancestors()
            .find(|ancestor| ancestor.ends_with("benchmark"))
        {
            if let Ok(rel) = path.strip_prefix(base) {
                return Some(sanitize_field(&format!(
                    "benchmark/{}",
                    rel.to_string_lossy()
                )));
            }
        }
        if let Ok(rel) = path.strip_prefix(root) {
            return Some(sanitize_field(&rel.to_string_lossy()));
        }
    }
    if let Some(rel) = &artifacts.sygus_rel {
        return Some(rel.clone());
    }
    Some(sanitize_field(&format!(
        "runs/{run_id}/artifacts/{}/{}.sygus2",
        job.suite_name, job.display_path
    )))
}

fn job_solver_name(job: &JobSpec) -> Option<String> {
    match job.suite_mode {
        SuiteMode::Native => None,
        SuiteMode::ExternalSygus => job
            .sygus_solver
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        SuiteMode::ExternalCommand => job
            .external_command
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
    }
}

fn job_solver_args(job: &JobSpec) -> Vec<String> {
    match job.suite_mode {
        SuiteMode::Native => Vec::new(),
        SuiteMode::ExternalSygus => job.sygus_solver_args.clone(),
        SuiteMode::ExternalCommand => job.external_command_args.clone(),
    }
}

fn combine_output_summaries(stdout: String, stderr: String) -> Option<String> {
    let stdout = blank_to_none(stdout);
    let stderr = blank_to_none(stderr);
    match (stdout, stderr) {
        (Some(stdout), Some(stderr)) => Some(format!("stdout: {stdout} | stderr: {stderr}")),
        (Some(stdout), None) => Some(format!("stdout: {stdout}")),
        (None, Some(stderr)) => Some(format!("stderr: {stderr}")),
        (None, None) => None,
    }
}

fn load_external_command_metadata(
    path: &Path,
) -> Result<Option<ExternalCommandMetadata>, Box<dyn std::error::Error>> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = read_text_file(path)?;
    let metadata = serde_json::from_str::<ExternalCommandMetadata>(&text)?;
    Ok(Some(metadata))
}

fn expand_external_arg(
    template: &str,
    input: &Path,
    input_dir: &Path,
    input_name: &str,
    input_stem: &str,
    suite: &str,
    artifacts_dir: &Path,
    result_json: &Path,
    solution_path: &Path,
) -> String {
    template
        .replace("{input}", &input.to_string_lossy())
        .replace("{input_dir}", &input_dir.to_string_lossy())
        .replace("{input_name}", input_name)
        .replace("{input_stem}", input_stem)
        .replace("{suite}", suite)
        .replace("{artifacts_dir}", &artifacts_dir.to_string_lossy())
        .replace("{result_json}", &result_json.to_string_lossy())
        .replace("{solution}", &solution_path.to_string_lossy())
}
