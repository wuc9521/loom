use clap::Parser;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "LS Parser - Extended SyGuS Language Interpreter", long_about = None)]
pub struct CLI {
    /// Input file to process (if not provided, starts interactive mode)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output mode: what to do with the processed file
    #[arg(short, long, value_enum, default_value_t = OutputMode::Execute)]
    output: OutputMode,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Solver to use for synthesis (requires CVC5 to be installed)
    #[arg(short, long, value_enum, default_value_t = Solver::CVC5)]
    solver: Solver,

    /// Output file for translated SyGuS (when using --output save)
    #[arg(long = "output-file", value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputMode {
    /// Execute the file normally (default)
    Execute,
    /// Print the parsed LS commands
    Show,
    /// Translate to standard SyGuS and print
    Translate,
    /// Translate to standard SyGuS and save to file
    Save,
    /// Run synthesis using the specified solver
    Synthesize,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Solver {
    /// Use CVC5 solver (default)
    CVC5,
    /// Use DryadSynth solver (if available)
    DryadSynth,
    /// Use no solver (just translate)
    None,
}


fn main() -> Result<(), io::Error> {
        Ok(())
}
