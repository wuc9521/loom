use clap::Parser as ClapParser;
use ls_parser::lang::runtime::Session;
use ls_parser::parser::ast::LSCmd;
use ls_parser::theory::pml;
use ls_parser::theory::SygusDesugar;
use ls_parser::theory::{fol, re, Logic, ModelCtx};
use ls_parser::utils::extract_logic_sygus;
use std::{
    fs,
    io::{self},
    path::PathBuf,
};

mod repl;

/// CLI application for parsing your custom language
#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "Parse and analyze your custom language files"
)]
pub struct CLI {
    /// Path to the input file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Dump model info as JSON to this path
    #[arg(short = 'd', long = "dump")]
    dump: Option<PathBuf>,

    /// Translate to SyGuS BV and print to stdout
    #[arg(long = "dump-sygus", default_value_t = false)]
    dump_sygus: bool,

    #[arg(long = "from-sygus", default_value_t = false)]
    from_sygus: bool,

    #[arg(long = "guard", default_value_t = false)]
    guard: bool,
}

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    let file_path = args.file;
    if file_path == None {
        return repl::repl();
    }
    let input = fs::read_to_string(file_path.unwrap())?;
    if args.from_sygus {
        let sygus_file = sygus_parser::SyGuSFile::from_str(&input.as_str()).unwrap();
        let logic_str = extract_logic_sygus(&sygus_file).unwrap();
        let logic = match logic_str {
            "RE" => Logic::RE,
            _ => {
                eprintln!("Unsupported logic: {}", logic_str);
                return Err(io::Error::new(io::ErrorKind::Other, "Unsupported logic"));
            }
        };
        println!("{}", logic.desugar(&sygus_file));
        return Ok(());
    }
    let mut session = Session::new();
    let results = ls_parser::parser::ast::parse_commands(&input, Some(&mut session.runtime));
    let mut lscmd_vec: Vec<ls_parser::parser::ast::LSCmd> = vec![];
    for res in results {
        match res {
            Ok(cmd) => {
                let _ = session.handle_command(&cmd);
                lscmd_vec.push(cmd);
            }
            Err(e) => {
                eprintln!("parse error: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("parse error: {}", e),
                ));
            }
        }
    }
    let mut ls_file = ls_parser::parser::ast::LSFile { cmds: lscmd_vec };
    if args.dump_sygus {
        match session.logic {
            Some(Logic::FOL) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = fol::FOModelCtx::default();
                        for thcmd in &model.ext_cmds {
                            fol::handle_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::FOL(ctx));
                    }
                }
                let output = pml::to_sygus(&ls_file);
                for cmd in output {
                    println!("{}", cmd);
                }
            }
            Some(Logic::PML) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = pml::KripkeCtx::default();
                        for thcmd in &model.ext_cmds {
                            pml::handle_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::PML(ctx));
                    }
                }
                let output = pml::to_sygus(&ls_file);
                for cmd in output {
                    println!("{}", cmd);
                }
            }
            Some(Logic::RE) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = re::StringCtx::default();
                        for thcmd in &model.ext_cmds {
                            re::handle_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::RE(ctx));
                    }
                }
                let output = re::to_sygus(&ls_file);
                for cmd in output {
                    println!("{}", cmd);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
