use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::RunArgs,
    model::{EffectiveSuite, JobCollection, JobSpec, Manifest, SuiteKind},
    util::{
        canonical_display, canonicalize_existing, default_solver_args, resolve_command_path,
        resolve_relative,
    },
};

pub fn materialize_suites(
    manifest: &Manifest,
    args: &RunArgs,
    manifest_path: &Path,
) -> Vec<EffectiveSuite> {
    let base_dir = manifest_path.parent().unwrap_or(Path::new("."));
    let selected = if args.suite.is_empty() {
        None
    } else {
        Some(args.suite.iter().cloned().collect::<Vec<_>>())
    };
    let default_delis = args
        .delis
        .clone()
        .or_else(|| manifest.delis.clone())
        .map(|path| resolve_command_path(base_dir, &path))
        .unwrap_or_else(|| base_dir.join("target/release/delis"));
    let mut suites = Vec::new();
    for suite in &manifest.suites {
        if !suite.enabled || suite.kind != SuiteKind::Delis {
            continue;
        }
        if let Some(selected) = &selected {
            if !selected.iter().any(|name| name == &suite.name) {
                continue;
            }
        }
        suites.push(EffectiveSuite {
            name: suite.name.clone(),
            resolved_path: resolve_relative(base_dir, &suite.path),
            recursive: suite.recursive,
            mode: suite.mode,
            file_extensions: normalize_extensions(&suite.file_extensions),
            delis: suite
                .delis
                .clone()
                .map(|path| resolve_command_path(base_dir, &path))
                .unwrap_or_else(|| default_delis.clone()),
            delis_args: suite.delis_args.clone(),
            sygus_solver: suite
                .sygus_solver
                .clone()
                .map(|path| resolve_command_path(base_dir, &path)),
            sygus_solver_args: suite.sygus_solver_args.clone(),
            working_dir: suite
                .working_dir
                .clone()
                .map(|path| resolve_relative(base_dir, &path)),
            external_command: suite
                .external_command
                .clone()
                .map(|path| resolve_command_path(base_dir, &path)),
            external_command_args: suite.external_command_args.clone(),
            solve_timeout_secs: suite
                .solve_timeout_secs
                .or(manifest.solve_timeout_secs)
                .unwrap_or(600),
            sygus_timeout_secs: suite
                .sygus_timeout_secs
                .or(manifest.sygus_timeout_secs)
                .unwrap_or(600),
            capture_sygus_stats: suite
                .capture_sygus_stats
                .or(manifest.capture_sygus_stats)
                .unwrap_or(true),
        });
    }
    suites
}

pub fn collect_jobs(
    suites: &[EffectiveSuite],
    filter: &Option<String>,
) -> Result<JobCollection, Box<dyn std::error::Error>> {
    let mut jobs = Vec::new();
    let mut suite_order = Vec::new();
    for (suite_index, suite) in suites.iter().enumerate() {
        let files = collect_suite_files(
            &suite.resolved_path,
            suite.recursive,
            &suite.file_extensions,
        )?;
        suite_order.push(suite.clone());
        for file in files {
            let display_path = canonical_display(&file, &suite.resolved_path);
            if let Some(filter_text) = filter {
                if !display_path.contains(filter_text) && !suite.name.contains(filter_text) {
                    continue;
                }
            }
            jobs.push(JobSpec {
                suite_index,
                suite_name: suite.name.clone(),
                suite_mode: suite.mode,
                file_path: file.clone(),
                display_path: display_path.clone(),
                order_key: format!("{}	{}", suite.name, display_path),
                delis: suite.delis.clone(),
                delis_args: suite.delis_args.clone(),
                sygus_solver: suite.sygus_solver.clone(),
                sygus_solver_args: if suite.sygus_solver_args.is_empty() {
                    default_solver_args(suite.sygus_solver.as_deref())
                } else {
                    suite.sygus_solver_args.clone()
                },
                working_dir: suite.working_dir.clone(),
                external_command: suite.external_command.clone(),
                external_command_args: suite.external_command_args.clone(),
                solve_timeout_secs: suite.solve_timeout_secs,
                sygus_timeout_secs: suite.sygus_timeout_secs,
                capture_sygus_stats: suite.capture_sygus_stats,
            });
        }
    }
    jobs.sort_by(|a, b| {
        a.suite_index
            .cmp(&b.suite_index)
            .then_with(|| a.display_path.cmp(&b.display_path))
    });
    Ok(JobCollection { jobs, suite_order })
}

pub fn collect_suite_files(
    path: &Path,
    recursive: bool,
    extensions: &[String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if path.is_file() {
        if has_allowed_extension(path, extensions) {
            return Ok(vec![canonicalize_existing(path)?]);
        }
        return Err(format!(
            "benchmark path does not match configured extensions: {}",
            path.display()
        )
        .into());
    }
    if !path.is_dir() {
        return Err(format!("benchmark path not found: {}", path.display()).into());
    }
    let mut files = Vec::new();
    collect_dir_files(path, recursive, extensions, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_dir_files(
    path: &Path,
    recursive: bool,
    extensions: &[String],
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if recursive {
                collect_dir_files(&child, true, extensions, out)?;
            }
        } else if file_type.is_file() && has_allowed_extension(&child, extensions) {
            out.push(canonicalize_existing(&child)?);
        }
    }
    Ok(())
}

fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    if extensions.is_empty() {
        return vec!["sl".to_string()];
    }
    extensions
        .iter()
        .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .collect()
}

fn has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());
    match ext {
        Some(ext) => extensions.iter().any(|allowed| allowed == &ext),
        None => false,
    }
}
