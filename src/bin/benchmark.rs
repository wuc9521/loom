// scripts/benchmark.rs
use chrono::Local;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Rust benchmark for ls-parser + cvc5 pipeline"
)]
struct Args {
    /// File or directory containing .sl files
    input_path: String,
    /// Number of benchmark runs (default: 10)
    #[arg(default_value_t = 10)]
    iterations: usize,
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn benchmark_file(file: &Path, output_dir: &Path, iterations: usize) {
    let file_name = file.file_stem().unwrap().to_string_lossy();
    let file_output_dir = output_dir.join(file_name.as_ref());
    fs::create_dir_all(&file_output_dir).unwrap();
    println!("Benchmarking: {}", file.display());
    let command = format!(
        "./target/release/ls-parser --file '{}' --dump-sygus --dump '{}' | cvc5 --lang=sygus2",
        file.display(),
        file_output_dir.display()
    );
    let results_json = file_output_dir.join("results.json");
    let results_md = file_output_dir.join("results.md");
    let status = Command::new("hyperfine")
        .args([
            "--warmup",
            "2",
            "--min-runs",
            "3",
            "--max-runs",
            &iterations.to_string(),
            "--export-json",
            results_json.to_str().unwrap(),
            "--export-markdown",
            results_md.to_str().unwrap(),
            "--command-name",
            &file_name,
            &command,
        ])
        .status()
        .expect("failed to run hyperfine");
    if !status.success() {
        eprintln!("hyperfine failed for {}", file.display());
    }
    println!("Results saved to: {}", file_output_dir.display());
}

fn benchmark_directory(dir: &Path, output_dir: &Path, iterations: usize) {
    println!("Benchmarking all .sl files in: {}", dir.display());
    let mut files = vec![];
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "sl")
        {
            files.push(entry.path().to_path_buf());
        }
    }
    if files.is_empty() {
        eprintln!("No .sl files found in {}", dir.display());
        std::process::exit(1);
    }
    println!("Found {} .sl files", files.len());
    for file in &files {
        benchmark_file(file, output_dir, iterations);
    }
    generate_summary(output_dir);
}

fn generate_summary(output_dir: &Path) {
    let summary_file = output_dir.join("summary.md");
    let mut summary = String::new();
    summary.push_str("# Benchmark Summary\n");
    summary.push_str(&format!("Generated on: {}\n\n", Local::now().to_rfc3339()));
    let mut result_files = vec![];
    for entry in WalkDir::new(output_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.file_name() == "results.md" {
            result_files.push(entry.path().to_path_buf());
        }
    }
    if result_files.is_empty() {
        summary.push_str("No benchmark results found\n");
        fs::write(&summary_file, summary).unwrap();
        return;
    }
    summary.push_str("## Results by File\n\n");
    for result_file in &result_files {
        let file_name = result_file
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        summary.push_str(&format!("### {}\n\n", file_name));
        let content = fs::read_to_string(result_file).unwrap();
        summary.push_str(&content);
        summary.push_str("\n");
    }
    fs::write(&summary_file, summary).unwrap();
    println!("Summary report generated: {}", summary_file.display());
}

fn main() {
    let args = Args::parse();
    if !check_command_exists("hyperfine") {
        eprintln!("hyperfine not found. Please install hyperfine.");
        std::process::exit(1);
    }
    if !check_command_exists("cvc5") {
        eprintln!("cvc5 not found. Please install cvc5.");
        std::process::exit(1);
    }
    let output_dir = PathBuf::from(format!(
        "benchmark/{}",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    fs::create_dir_all(&output_dir).unwrap();
    println!("Starting benchmark...");
    println!("Input: {}", args.input_path);
    println!("Iterations: {}", args.iterations);
    println!("Output directory: {}", output_dir.display());
    println!("");
    let input_path = Path::new(&args.input_path);
    if input_path.is_file() {
        benchmark_file(input_path, &output_dir, args.iterations);
    } else if input_path.is_dir() {
        benchmark_directory(input_path, &output_dir, args.iterations);
    } else {
        eprintln!("Error: {} is not a file or directory", args.input_path);
        std::process::exit(1);
    }
    println!("");
    println!(
        "Benchmark completed! Results saved to: {}",
        output_dir.display()
    );
}
