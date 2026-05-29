use serde::Serialize;
use std::{
    fs::{self, File},
    hash::{DefaultHasher, Hasher},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::model::{GitInfo, MachineInfo};

pub fn resolve_jobs(setting: Option<&str>, pending_jobs: usize) -> usize {
    if pending_jobs == 0 {
        return 1;
    }
    let available = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let requested = match setting {
        None => available,
        Some("auto") => available,
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|n| *n > 0)
            .unwrap_or(available),
    };
    requested.clamp(1, pending_jobs)
}

pub fn default_solver_args(path: Option<&Path>) -> Vec<String> {
    let Some(path) = path else {
        return Vec::new();
    };
    match path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
    {
        "dryadsynth-bv" => vec!["-in".to_string()],
        "cvc5" => vec!["--lang=sygus2".to_string()],
        _ => Vec::new(),
    }
}

pub fn collect_machine_info(machine_tag: Option<String>) -> MachineInfo {
    MachineInfo {
        hostname: detect_hostname(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        logical_cpus: thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1),
        machine_tag,
    }
}

pub fn collect_git_info(base_dir: &Path) -> Option<GitInfo> {
    let root = run_git(base_dir, ["rev-parse", "--show-toplevel"]).ok()?;
    let branch = run_git(base_dir, ["rev-parse", "--abbrev-ref", "HEAD"]).ok()?;
    let commit = run_git(base_dir, ["rev-parse", "--short", "HEAD"]).ok()?;
    let dirty = !run_git(base_dir, ["status", "--porcelain", "--untracked-files=no"])
        .unwrap_or_default()
        .trim()
        .is_empty();
    Some(GitInfo {
        root,
        branch,
        commit,
        dirty,
    })
}

pub fn run_git<const N: usize>(
    base_dir: &Path,
    args: [&str; N],
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(base_dir)
        .output()?;
    if !output.status.success() {
        return Err("git command failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn detect_hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            Command::new("hostname").output().ok().and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| "unknown-host".to_string())
}

pub fn canonical_display(file: &Path, root: &Path) -> String {
    if root.is_file() {
        return file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
    }
    make_relative_string(file, root)
}

pub fn canonicalize_existing(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(fs::canonicalize(path)?)
}

pub fn canonical_string(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

pub fn resolve_relative(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

pub fn slugify(text: &str) -> String {
    let mut output = String::new();
    let mut last_dash = false;
    for ch in text.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            output.push(lower);
            last_dash = false;
        } else if !last_dash {
            output.push('-');
            last_dash = true;
        }
    }
    output.trim_matches('-').to_string()
}

pub fn safe_relative_path(text: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in Path::new(text).components() {
        let cleaned = sanitize_field(&component.as_os_str().to_string_lossy())
            .replace('/', "_")
            .replace('\\', "_");
        if !cleaned.is_empty() {
            path.push(cleaned);
        }
    }
    path
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    format!("{:016x}", hasher.finish())
}

pub fn sanitize_field(text: &str) -> String {
    text.replace('\t', " ").replace('\n', " ")
}

pub fn blank_to_none(text: String) -> Option<String> {
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn suffix_with_summary(summary: &str) -> String {
    if summary.is_empty() {
        String::new()
    } else {
        format!("; {summary}")
    }
}

pub fn combine_detail(prefix: Option<String>, detail: Option<String>) -> String {
    match (prefix, detail) {
        (Some(prefix), Some(detail)) if !detail.is_empty() => format!("{prefix} | {detail}"),
        (Some(prefix), _) => prefix,
        (_, Some(detail)) => detail,
        _ => String::new(),
    }
}

pub fn summarize_text_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let text = read_text_file(path)?;
    let summary = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if summary.len() > 240 {
        Ok(format!("{}...", &summary[..240]))
    } else {
        Ok(summary)
    }
}

pub fn read_text_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    Ok(String::from_utf8_lossy(&data).to_string())
}

pub fn count_lines(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(read_text_file(path)?.lines().count() as u64)
}

pub fn file_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(fs::metadata(path)?.len())
}

pub fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(value)?;
    write_string_atomic(path, &text)
}

pub fn write_string_atomic(path: &Path, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut writer = BufWriter::new(File::create(&tmp)?);
        writer.write_all(text.as_bytes())?;
        writer.flush()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn make_relative_string(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

pub fn temp_file_base(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        unix_now_nanos()
    ));
    path
}

pub fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn unix_now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
