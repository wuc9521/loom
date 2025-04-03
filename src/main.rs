use clap::Parser as ClapParser;
use std::fs;
use std::io;
use std::path::PathBuf;
// use sygus_parser_rs::sygus::parser::SyGuSV2Parser as SyGuSV2Parser;
use sygus_parser_rs::parser::ls::LSParser as LSParser;
use sygus_parser_rs::parser::ls::Rule;
use pest::Parser as PestParser;

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
    let result = LSParser::parse(Rule::LSProg, &input);
    match result {
        Ok(pairs) => {
            for pair in pairs {
                print!("{}", &pair);
            }
        }
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    }
    Ok(())
}
