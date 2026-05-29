use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    hash::{DefaultHasher, Hasher},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::{
    cli::RunArgs,
    model::{
        EffectiveSuite, GitInfo, JobRecord, JobSpec, LatestPointer, Manifest, RunLayout,
        RunMetadata, StateLoad, StatusSummary, TSV_HEADER,
    },
    util::{
        canonical_string, resolve_relative, slugify, unix_now_secs, write_json_atomic,
        write_string_atomic,
    },
};

pub fn load_job_state(path: &Path) -> Result<StateLoad, Box<dyn std::error::Error>> {
    let mut records = HashMap::new();
    if !path.exists() {
        return Ok(StateLoad { records });
    }
    let reader = BufReader::new(File::open(path)?);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: JobRecord = serde_json::from_str(&line)?;
        records.insert(record.order_key.clone(), record);
    }
    Ok(StateLoad { records })
}

pub fn append_job_record(
    path: &Path,
    record: &JobRecord,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut writer, record)?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn export_suite_tsvs(
    results_dir: &Path,
    suites: &[EffectiveSuite],
    records: &HashMap<String, JobRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    for suite in suites {
        let mut rows: Vec<&JobRecord> = records
            .values()
            .filter(|record| record.suite_name == suite.name)
            .collect();
        rows.sort_by(|a, b| a.display_path.cmp(&b.display_path));
        let mut content = String::from(TSV_HEADER);
        content.push('\n');
        for row in rows {
            content.push_str(&row.tsv_row());
            content.push('\n');
        }
        let output = results_dir.join(format!("{}.tsv", suite.name));
        write_string_atomic(&output, &content)?;
    }
    Ok(())
}

pub fn summarize_status(
    suites: &[EffectiveSuite],
    records: &HashMap<String, JobRecord>,
    pending_jobs: &[JobSpec],
) -> BTreeMap<String, StatusSummary> {
    let mut summary = BTreeMap::new();
    for suite in suites {
        let mut by_status = BTreeMap::new();
        let completed_rows: Vec<&JobRecord> = records
            .values()
            .filter(|record| record.suite_name == suite.name)
            .collect();
        for row in &completed_rows {
            *by_status.entry(row.status.clone()).or_insert(0) += 1;
        }
        let pending = pending_jobs
            .iter()
            .filter(|job| job.suite_name == suite.name)
            .count();
        let completed = completed_rows.len();
        summary.insert(
            suite.name.clone(),
            StatusSummary {
                total: completed + pending,
                completed,
                pending,
                by_status,
            },
        );
    }
    summary
}

pub fn print_status_lines(label: &str, summary: &BTreeMap<String, StatusSummary>) {
    println!("{label}:");
    for (suite, item) in summary {
        let statuses = item
            .by_status
            .iter()
            .map(|(status, count)| format!("{status}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {}: completed={} pending={} total={}{}",
            suite,
            item.completed,
            item.pending,
            item.total,
            if statuses.is_empty() {
                String::new()
            } else {
                format!(" [{statuses}]")
            }
        );
    }
}

pub fn initialize_run_metadata(
    layout: &RunLayout,
    manifest_path: &Path,
    manifest_hash: &str,
    target_fingerprint: &str,
    run_label: Option<String>,
    machine: crate::model::MachineInfo,
    git: Option<GitInfo>,
) -> Result<RunMetadata, Box<dyn std::error::Error>> {
    if layout.metadata_path.exists() {
        let text = fs::read_to_string(&layout.metadata_path)?;
        let mut metadata: RunMetadata = serde_json::from_str(&text)?;
        metadata.updated_at_unix_secs = unix_now_secs();
        return Ok(metadata);
    }
    let metadata = RunMetadata {
        run_id: layout
            .run_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        manifest_path: canonical_string(manifest_path),
        manifest_hash: manifest_hash.to_string(),
        target_fingerprint: target_fingerprint.to_string(),
        results_dir: canonical_string(&layout.results_dir),
        run_dir: canonical_string(&layout.run_dir),
        run_label,
        created_at_unix_secs: unix_now_secs(),
        updated_at_unix_secs: unix_now_secs(),
        machine,
        git,
    };
    write_json_atomic(&layout.metadata_path, &metadata)?;
    Ok(metadata)
}

pub fn create_run_layout(
    results_dir: &Path,
    run_id: &str,
) -> Result<RunLayout, Box<dyn std::error::Error>> {
    let run_dir = results_dir.join("runs").join(run_id);
    fs::create_dir_all(&run_dir)?;
    Ok(RunLayout {
        results_dir: results_dir.to_path_buf(),
        run_dir: run_dir.clone(),
        state_path: run_dir.join("results.jsonl"),
        metadata_path: run_dir.join("run.json"),
        latest_pointer_path: results_dir.join("latest-run.json"),
    })
}

pub fn select_run_id(
    results_dir: &Path,
    manifest_path: &Path,
    manifest_hash: &str,
    target_fingerprint: &str,
    explicit_run_id: Option<String>,
    force_new_run: bool,
    run_label: Option<String>,
    git: &Option<GitInfo>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(run_id) = explicit_run_id {
        return Ok(run_id);
    }
    if !force_new_run {
        if let Ok(pointer) = load_latest_pointer(results_dir) {
            if pointer.manifest_hash == manifest_hash
                && pointer.target_fingerprint == target_fingerprint
                && pointer.manifest_path == canonical_string(manifest_path)
            {
                return Ok(pointer.run_id);
            }
        }
    }
    let label = run_label
        .or_else(|| {
            git.as_ref()
                .map(|info| format!("{}-{}", info.branch, info.commit))
        })
        .unwrap_or_else(|| "run".to_string());
    Ok(format!("{}-{}", unix_now_secs(), slugify(&label)))
}

pub fn persist_latest_pointer(
    layout: &RunLayout,
    metadata: &RunMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let pointer = LatestPointer {
        run_id: metadata.run_id.clone(),
        manifest_path: metadata.manifest_path.clone(),
        manifest_hash: metadata.manifest_hash.clone(),
        target_fingerprint: metadata.target_fingerprint.clone(),
        run_dir: metadata.run_dir.clone(),
    };
    write_json_atomic(&layout.latest_pointer_path, &pointer)
}

pub fn load_latest_pointer(
    results_dir: &Path,
) -> Result<LatestPointer, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(results_dir.join("latest-run.json"))?;
    Ok(serde_json::from_str(&text)?)
}

pub fn snapshot_manifest(
    layout: &RunLayout,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = layout.run_dir.join("manifest.snapshot.toml");
    if !path.exists() {
        write_string_atomic(&path, source)?;
    }
    Ok(())
}

pub fn resolve_results_dir(
    manifest: &Manifest,
    args: &RunArgs,
    manifest_path: &Path,
) -> std::path::PathBuf {
    let base_dir = manifest_path.parent().unwrap_or(Path::new("."));
    if let Some(path) = &args.results_dir {
        path.clone()
    } else if let Some(path) = &manifest.results_dir {
        resolve_relative(base_dir, path)
    } else {
        base_dir.join("bench-results")
    }
}

pub fn build_target_fingerprint(
    manifest: &Manifest,
    args: &RunArgs,
    git: &Option<GitInfo>,
) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(manifest.run_label.as_deref().unwrap_or("").as_bytes());
    if let Some(delis) = args.delis.as_ref().or(manifest.delis.as_ref()) {
        hasher.write(canonical_string(delis).as_bytes());
    }
    if let Some(git) = git {
        hasher.write(git.branch.as_bytes());
        hasher.write(git.commit.as_bytes());
        hasher.write_u8(git.dirty as u8);
    }
    format!("{:016x}", hasher.finish())
}
