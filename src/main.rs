use clap::Parser as ClapParser;
use ls_parser::parser::*;
use std::fs;
use std::io;
use std::path::PathBuf;

/// CLI application for parsing your custom language
#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "Parse and analyze your custom language files"
)]
pub struct CLI {
    /// Path to the input file
    #[arg(short, long, required = true)]
    input: PathBuf,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    let input = fs::read_to_string(&args.input)?;
    let problem = file::parse(&input.as_str()).unwrap();
    if args.verbose {
        println!("Parsed problem: {:?}", problem);
    }
    Ok(())
}
