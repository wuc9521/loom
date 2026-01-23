use clap::Parser as ClapParser;
use delis::{
    lang::runtime::Session,
    parser::ast::DelisCmd,
    theory::{fol, pml, re, Logic, ModelCtx, SygusDesugar},
    utils::extract_logic_sygus,
};
use std::{
    fs,
    io::{self},
    path::PathBuf,
};

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

    /// Translate to SyGuS BV and print to stdout
    #[arg(short, long = "dump-sygus", default_value_t = false)]
    dump_sygus: bool,

    /// Translate to CHC (HORN) and print to stdout
    #[arg(long = "dump-chc", default_value_t = false)]
    dump_chc: bool,

    #[arg(short, long = "as-sygus", default_value_t = false)]
    as_sygus: bool,
}

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    let file_path = args.file;
    if file_path == None {
        eprintln!("missing --file argument");
        return Ok(());
    }
    let input = fs::read_to_string(file_path.unwrap())?;
    if args.as_sygus {
        let sygus_file = sygus_parser::SyGuSFile::from_str(&input.as_str()).unwrap();
        let logic_str = extract_logic_sygus(&sygus_file).unwrap();
        let logic = match logic_str {
            "SLIA" => Logic::RE,
            _ => {
                eprintln!("Unsupported logic: {}", logic_str);
                return Err(io::Error::new(io::ErrorKind::Other, "Unsupported logic"));
            }
        };
        println!("{}", logic.desugar(&sygus_file));
        return Ok(());
    }
    let mut session = Session::new();
    let results = delis::parser::ast::parse_commands(&input, Some(&mut session.runtime));
    let mut delis_cmd_vec: Vec<delis::parser::ast::DelisCmd> = vec![];
    for res in results {
        match res {
            Ok(cmd) => {
                let _ = session.handle_command(&cmd);
                delis_cmd_vec.push(cmd);
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
    let mut delis_file = delis::parser::ast::DelisFile {
        cmds: delis_cmd_vec,
    };
    if args.dump_sygus {
        match session.logic {
            Some(Logic::FOL) => {
                for cmd in &mut delis_file.cmds {
                    if let DelisCmd::DefineModel(model) = cmd {
                        let mut ctx = fol::FOModelCtx::default();
                        for thcmd in &model.ext_cmds {
                            fol::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::FOL(ctx));
                    }
                }
                for cmd in fol::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::PML) => {
                for cmd in &mut delis_file.cmds {
                    if let DelisCmd::DefineModel(model) = cmd {
                        let mut ctx = pml::KripkeCtx::default();
                        for thcmd in &model.ext_cmds {
                            pml::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::PML(ctx));
                    }
                }
                for cmd in pml::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::RE) => {
                for cmd in &mut delis_file.cmds {
                    if let DelisCmd::DefineModel(model) = cmd {
                        let mut ctx = re::StringCtx::default();
                        for thcmd in &model.ext_cmds {
                            re::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::RE(ctx));
                    }
                }
                for cmd in re::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
            _ => {}
        }
    }
    if args.dump_chc {
        match session.logic {
            Some(Logic::RE) => {
                for cmd in re::to_chc(&delis_file) {
                    println!("{}", cmd);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
