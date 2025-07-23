use clap::Parser as ClapParser;
use ls_parser::{
    parser::*,
    theories::{dispatch_extcmd, Logic},
};
use std::{fs, io, path::PathBuf};

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
    file: PathBuf,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn extract_logic(problem: &LSFile) -> Option<&str> {
    for cmd in &problem.cmds {
        if let LSCmd::As(sygus_cmd) = cmd {
            if let sygus_parser::SyGuSCmd::SetLogic(ref logic) = sygus_cmd {
                return Some(logic.as_str());
            }
        }
    }
    None
}

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    let input = fs::read_to_string(&args.file)?;
    let problem = file::parse(&input.as_str()).unwrap();

    let logic_str = extract_logic(&problem).unwrap();
    let logic = match logic_str {
        "FOL" => Logic::FOL,
        "PML" => Logic::PML,
        _ => {
            eprintln!("Error: Unknown logic: {}", logic_str);
            return Err(io::Error::new(io::ErrorKind::Other, "Unknown logic"));
        }
    };
    // print current logic
    println!("Current logic: {}", logic_str);

    for cmd in &problem.cmds {
        if let LSCmd::ModelDef(model) = cmd {
            for extcmd in &model.ext_cmds {
                dispatch_extcmd(logic, extcmd);
            }
        }
        if let LSCmd::ThSpecCmd(thcmd) = cmd {
            dispatch_extcmd(logic, thcmd);
        }
    }

    if args.verbose {
        println!("Parsed problem:\n {:?}", problem);
    }
    Ok(())
}
