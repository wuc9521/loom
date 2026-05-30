use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

use crate::util::sanitize_field;

pub const TSV_HEADER: &str = "group	path	status	total_ms	sygus_ms	solve_ms	sygus_bytes	sygus_lines	solution_bytes	delis_exit_code	solver_exit_code	delis_args	solver	solver_args	solution_path	detail";

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub delis: Option<PathBuf>,
    pub results_dir: Option<PathBuf>,
    pub solve_timeout_secs: Option<u64>,
    pub sygus_timeout_secs: Option<u64>,
    pub capture_sygus_stats: Option<bool>,
    pub jobs: Option<usize>,
    pub run_label: Option<String>,
    #[serde(default, alias = "folders")]
    pub suites: Vec<SuiteManifest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuiteManifest {
    pub name: String,
    pub path: PathBuf,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub rerun: bool,
    #[serde(default)]
    pub kind: SuiteKind,
    #[serde(default)]
    pub mode: SuiteMode,
    pub delis: Option<PathBuf>,
    #[serde(default)]
    pub delis_args: Vec<String>,
    pub sygus_solver: Option<PathBuf>,
    #[serde(default)]
    pub sygus_solver_args: Vec<String>,
    pub persistent_sygus_dir: Option<PathBuf>,
    #[serde(default)]
    pub file_extensions: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub external_command: Option<PathBuf>,
    #[serde(default)]
    pub external_command_args: Vec<String>,
    pub solve_timeout_secs: Option<u64>,
    pub sygus_timeout_secs: Option<u64>,
    pub capture_sygus_stats: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SuiteKind {
    #[default]
    Delis,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SuiteMode {
    #[default]
    Native,
    ExternalSygus,
    ExternalCommand,
}

#[derive(Debug, Clone)]
pub struct EffectiveSuite {
    pub name: String,
    pub resolved_path: PathBuf,
    pub recursive: bool,
    pub mode: SuiteMode,
    pub file_extensions: Vec<String>,
    pub delis: PathBuf,
    pub delis_args: Vec<String>,
    pub sygus_solver: Option<PathBuf>,
    pub sygus_solver_args: Vec<String>,
    pub persistent_sygus_dir: Option<PathBuf>,
    pub working_dir: Option<PathBuf>,
    pub external_command: Option<PathBuf>,
    pub external_command_args: Vec<String>,
    pub solve_timeout_secs: u64,
    pub sygus_timeout_secs: u64,
    pub capture_sygus_stats: bool,
}

#[derive(Debug, Clone)]
pub struct JobSpec {
    pub suite_index: usize,
    pub suite_name: String,
    pub suite_mode: SuiteMode,
    pub file_path: PathBuf,
    pub display_path: String,
    pub order_key: String,
    pub delis: PathBuf,
    pub delis_args: Vec<String>,
    pub sygus_solver: Option<PathBuf>,
    pub sygus_solver_args: Vec<String>,
    pub persistent_sygus_dir: Option<PathBuf>,
    pub working_dir: Option<PathBuf>,
    pub external_command: Option<PathBuf>,
    pub external_command_args: Vec<String>,
    pub solve_timeout_secs: u64,
    pub sygus_timeout_secs: u64,
    pub capture_sygus_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub run_id: String,
    pub suite_name: String,
    pub suite_mode: SuiteMode,
    pub display_path: String,
    pub order_key: String,
    pub status: String,
    pub total_ms: u64,
    pub sygus_ms: u64,
    pub solve_ms: u64,
    pub sygus_bytes: u64,
    pub sygus_lines: u64,
    pub solution_bytes: u64,
    pub delis_exit_code: Option<i32>,
    pub solver_exit_code: Option<i32>,
    pub delis_args: Vec<String>,
    pub solver: Option<String>,
    pub solver_args: Vec<String>,
    pub solution_path: Option<String>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub sygus_path: Option<String>,
    pub detail: String,
    pub completed_at_utc: String,
}

impl JobRecord {
    pub fn tsv_row(&self) -> String {
        format!(
            "{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}	{}",
            sanitize_field(&self.suite_name),
            sanitize_field(&self.display_path),
            sanitize_field(&self.status),
            self.total_ms,
            self.sygus_ms,
            self.solve_ms,
            self.sygus_bytes,
            self.sygus_lines,
            self.solution_bytes,
            self.delis_exit_code
                .map(|code| code.to_string())
                .unwrap_or_default(),
            self.solver_exit_code
                .map(|code| code.to_string())
                .unwrap_or_default(),
            sanitize_field(&self.delis_args.join(" ")),
            sanitize_field(self.solver.as_deref().unwrap_or("")),
            sanitize_field(&self.solver_args.join(" ")),
            sanitize_field(self.solution_path.as_deref().unwrap_or("")),
            sanitize_field(&self.detail),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    pub run_id: String,
    pub manifest_path: String,
    pub manifest_hash: String,
    pub target_fingerprint: String,
    pub results_dir: String,
    pub run_dir: String,
    pub run_label: Option<String>,
    pub created_at_unix_secs: u64,
    pub updated_at_unix_secs: u64,
    pub machine: MachineInfo,
    pub git: Option<GitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestPointer {
    pub run_id: String,
    pub manifest_path: String,
    pub manifest_hash: String,
    pub target_fingerprint: String,
    pub run_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub logical_cpus: usize,
    pub machine_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub root: String,
    pub branch: String,
    pub commit: String,
    pub dirty: bool,
}

#[derive(Debug)]
pub struct RunLayout {
    pub results_dir: PathBuf,
    pub run_dir: PathBuf,
    pub state_path: PathBuf,
    pub metadata_path: PathBuf,
    pub latest_pointer_path: PathBuf,
}

#[derive(Debug)]
pub struct StatusSummary {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub by_status: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub timed_out: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DumpStats {
    pub sygus_ms: u64,
    pub sygus_bytes: u64,
    pub sygus_lines: u64,
    pub detail_prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    pub artifact_dir_abs: PathBuf,
    pub command_result_abs: PathBuf,
    pub solver_stdout_rel: String,
    pub driver_stderr_rel: String,
    pub solver_stdout_abs: PathBuf,
    pub driver_stderr_abs: PathBuf,
    pub solution_rel: Option<String>,
    pub solution_abs: Option<PathBuf>,
    pub sygus_rel: Option<String>,
    pub sygus_abs: Option<PathBuf>,
}

#[derive(Debug)]
pub struct JobCollection {
    pub jobs: Vec<JobSpec>,
    pub suite_order: Vec<EffectiveSuite>,
}

#[derive(Debug)]
pub struct StateLoad {
    pub records: std::collections::HashMap<String, JobRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalCommandMetadata {
    pub status: Option<String>,
    pub total_ms: Option<u64>,
    pub solve_ms: Option<u64>,
    pub detail: Option<String>,
    pub solution_path: Option<String>,
    pub solution_bytes: Option<u64>,
    pub solver_exit_code: Option<i32>,
}

pub fn default_enabled() -> bool {
    true
}
