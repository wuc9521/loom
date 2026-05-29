use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "delis-bench")]
#[command(about = "Robust benchmark runner for Delis and SyGuS comparisons")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run(RunArgs),
    List(ListArgs),
    Status(StatusArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    pub manifest: PathBuf,
    #[arg(long)]
    pub suite: Vec<String>,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub results_dir: Option<PathBuf>,
    #[arg(long)]
    pub delis: Option<PathBuf>,
    #[arg(long)]
    pub jobs: Option<String>,
    #[arg(long)]
    pub fresh: bool,
    #[arg(long)]
    pub machine_tag: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct ListArgs {
    pub manifest: PathBuf,
    #[arg(long)]
    pub suite: Vec<String>,
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct StatusArgs {
    pub manifest: PathBuf,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub results_dir: Option<PathBuf>,
}
