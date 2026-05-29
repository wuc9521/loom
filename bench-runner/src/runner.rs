use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
};

use crate::{
    cli::{ListArgs, RunArgs, StatusArgs},
    exec::worker_loop,
    manifest::{collect_jobs, materialize_suites},
    model::Manifest,
    state::{
        append_job_record, build_target_fingerprint, create_run_layout, export_suite_tsvs,
        initialize_run_metadata, load_job_state, load_latest_pointer, persist_latest_pointer,
        print_status_lines, resolve_results_dir, select_run_id, snapshot_manifest,
        summarize_status,
    },
    util::{
        canonical_string, canonicalize_existing, collect_git_info, collect_machine_info,
        hash_bytes, resolve_jobs,
    },
};

pub fn run_command(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = canonicalize_existing(&args.manifest)?;
    let manifest_source = fs::read_to_string(&manifest_path)?;
    let manifest_hash = hash_bytes(manifest_source.as_bytes());
    let manifest: Manifest = toml::from_str(&manifest_source)?;
    let machine = collect_machine_info(args.machine_tag.clone());
    let git = collect_git_info(manifest_path.parent().unwrap_or(Path::new(".")));
    let target_fingerprint = build_target_fingerprint(&manifest, &args, &git);
    let results_dir = resolve_results_dir(&manifest, &args, &manifest_path);
    fs::create_dir_all(&results_dir)?;

    let force_new_run =
        args.fresh || !args.suite.is_empty() || manifest.suites.iter().any(|suite| suite.rerun);
    let run_id = select_run_id(
        &results_dir,
        &manifest_path,
        &manifest_hash,
        &target_fingerprint,
        args.run_id.clone(),
        force_new_run,
        manifest.run_label.clone(),
        &git,
    )?;
    let layout = create_run_layout(&results_dir, &run_id)?;
    let mut metadata = initialize_run_metadata(
        &layout,
        &manifest_path,
        &manifest_hash,
        &target_fingerprint,
        manifest.run_label.clone(),
        machine,
        git,
    )?;
    persist_latest_pointer(&layout, &metadata)?;
    snapshot_manifest(&layout, &manifest_source)?;

    let suites = materialize_suites(&manifest, &args, &manifest_path);
    let job_collection = collect_jobs(&suites, &args.filter)?;
    let jobs = job_collection.jobs;
    let suite_order = job_collection.suite_order;
    if jobs.is_empty() {
        println!("no benchmark jobs matched the current filters");
        return Ok(());
    }

    let existing = load_job_state(&layout.state_path)?;
    let mut active_records = existing.records;
    let pending_jobs: Vec<_> = jobs
        .into_iter()
        .filter(|job| !active_records.contains_key(&job.order_key))
        .collect();

    let status_before = summarize_status(&suite_order, &active_records, &pending_jobs);
    print_status_lines("before", &status_before);
    export_suite_tsvs(&layout.results_dir, &suite_order, &active_records)?;

    if pending_jobs.is_empty() {
        println!("run {} is already complete", metadata.run_id);
        metadata.updated_at_unix_secs = crate::util::unix_now_secs();
        crate::util::write_json_atomic(&layout.metadata_path, &metadata)?;
        return Ok(());
    }

    let jobs_setting = args.jobs.or_else(|| manifest.jobs.map(|n| n.to_string()));
    let worker_count = resolve_jobs(jobs_setting.as_deref(), pending_jobs.len());
    println!(
        "run {}: {} pending job(s), {} worker(s), results at {}",
        metadata.run_id,
        pending_jobs.len(),
        worker_count,
        layout.run_dir.display()
    );

    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        let stop_flag = Arc::clone(&stop_flag);
        ctrlc::set_handler(move || {
            stop_flag.store(true, Ordering::SeqCst);
        })?;
    }

    let queue = Arc::new(Mutex::new(VecDeque::from(pending_jobs)));
    let (tx, rx) = mpsc::channel();
    let mut workers = Vec::new();
    for worker_id in 0..worker_count {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        let stop = Arc::clone(&stop_flag);
        let run_dir = layout.run_dir.clone();
        let run_id = metadata.run_id.clone();
        workers.push(
            thread::Builder::new()
                .name(format!("bench-worker-{worker_id}"))
                .spawn(move || worker_loop(queue, tx, stop, run_dir, run_id))?,
        );
    }
    drop(tx);

    let total_pending = queue.lock().map(|guard| guard.len()).unwrap_or(0);
    let mut completed_now = 0usize;
    while let Ok(record) = rx.recv() {
        completed_now += 1;
        println!(
            "[{}/{}] {} :: {}",
            completed_now, total_pending, record.suite_name, record.display_path
        );
        append_job_record(&layout.state_path, &record)?;
        active_records.insert(record.order_key.clone(), record);
        export_suite_tsvs(&layout.results_dir, &suite_order, &active_records)?;
        metadata.updated_at_unix_secs = crate::util::unix_now_secs();
        crate::util::write_json_atomic(&layout.metadata_path, &metadata)?;
    }

    for worker in workers {
        let _ = worker.join();
    }

    let queue_remaining = queue.lock().map(|guard| guard.len()).unwrap_or(0);
    let final_status = summarize_status(&suite_order, &active_records, &[]);
    print_status_lines("after", &final_status);
    if stop_flag.load(Ordering::SeqCst) && queue_remaining > 0 {
        eprintln!(
            "interrupted: {} job(s) left pending; rerun the same command to resume",
            queue_remaining
        );
    }
    metadata.updated_at_unix_secs = crate::util::unix_now_secs();
    crate::util::write_json_atomic(&layout.metadata_path, &metadata)?;
    Ok(())
}

pub fn list_command(args: ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = canonicalize_existing(&args.manifest)?;
    let manifest_source = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = toml::from_str(&manifest_source)?;
    let suites = materialize_suites(
        &manifest,
        &RunArgs {
            manifest: manifest_path.clone(),
            suite: args.suite.clone(),
            filter: args.filter.clone(),
            run_id: None,
            results_dir: None,
            delis: None,
            jobs: None,
            fresh: false,
            machine_tag: None,
        },
        &manifest_path,
    );
    let collection = collect_jobs(&suites, &args.filter)?;
    for suite in &collection.suite_order {
        let count = collection
            .jobs
            .iter()
            .filter(|job| job.suite_name == suite.name)
            .count();
        println!(
            "{}\tmode={:?}\tfiles={}\tpath={}",
            suite.name,
            suite.mode,
            count,
            suite.resolved_path.display()
        );
    }
    Ok(())
}

pub fn status_command(args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = canonicalize_existing(&args.manifest)?;
    let manifest_source = fs::read_to_string(&manifest_path)?;
    let manifest_hash = hash_bytes(manifest_source.as_bytes());
    let manifest: Manifest = toml::from_str(&manifest_source)?;
    let results_dir = if let Some(path) = args.results_dir {
        path
    } else if let Some(path) = manifest.results_dir.clone() {
        crate::util::resolve_relative(manifest_path.parent().unwrap_or(Path::new(".")), &path)
    } else {
        manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("bench-results")
    };
    let run_id = if let Some(run_id) = args.run_id {
        run_id
    } else {
        let pointer = load_latest_pointer(&results_dir)?;
        if pointer.manifest_hash != manifest_hash
            || canonical_string(&manifest_path) != pointer.manifest_path
        {
            return Err("latest run does not match the requested manifest".into());
        }
        pointer.run_id
    };
    let layout = create_run_layout(&results_dir, &run_id)?;
    let state = load_job_state(&layout.state_path)?;
    let suites = materialize_suites(
        &manifest,
        &RunArgs {
            manifest: manifest_path.clone(),
            suite: Vec::new(),
            filter: None,
            run_id: None,
            results_dir: None,
            delis: None,
            jobs: None,
            fresh: false,
            machine_tag: None,
        },
        &manifest_path,
    );
    let collection = collect_jobs(&suites, &None)?;
    let pending: Vec<_> = collection
        .jobs
        .into_iter()
        .filter(|job| !state.records.contains_key(&job.order_key))
        .collect();
    let summary = summarize_status(&collection.suite_order, &state.records, &pending);
    print_status_lines("status", &summary);
    Ok(())
}
