use clap::Parser;
use log::{error, warn};
use ls_parser::lang::interpreter::Interpreter;
use ls_parser::parser::ast::{parse, LSCmd};
use ls_parser::theory::fol;
use ls_parser::theory::pml;
use ls_parser::theory::re;
use ls_parser::theory::{Logic, ModelCtx, SygusDesugar};
use std::fs;
use std::io;
use std::path::PathBuf;

mod repl;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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

    #[arg(long = "dump-ls", default_value_t = false)]
    dump_ls: bool,

    #[arg(short, long = "as-sygus", default_value_t = false)]
    as_sygus: bool,

    #[arg(short, long = "theory-file")]
    theory_file: Option<PathBuf>,

    #[arg(short, long = "guard", default_value_t = false)]
    guard: bool,
}

fn extract_logic_sygus(sygus_file: &sygus_parser::SyGuSFile) -> Option<String> {
    for cmd in &sygus_file.cmds {
        if let sygus_parser::SyGuSCmd::SetLogic(logic) = cmd {
            return Some(logic.clone());
        }
    }
    None
}

/// unified file processing function
fn process_file(input: &str, args: &CLI) -> Result<(), io::Error> {
    // parse file
    let ls_file = parse(input)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

    // create interpreter and execute commands
    let mut interpreter = Interpreter::new();
    
    // if theory file is provided, load it first
    if let Some(ref theory_file) = args.theory_file {
        interpreter.load_theory_file(theory_file.to_str().unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
    }
    
    // execute commands directly from parsed file
    for cmd in &ls_file.cmds {
        interpreter.proc(cmd)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
    }

    // execute different outputs based on arguments
    if args.dump_ls {
        for cmd in &ls_file.cmds {
            println!("{}", cmd);
        }
    }

    if args.dump_sygus {
        let mut ls_file_mut = ls_file;

        // process models based on logic type
        match interpreter.logic {
            Some(Logic::FOL) => {
                process_models_for_logic(&mut ls_file_mut, |model| {
                    let mut ctx = fol::FOModelCtx::default();
                    for thcmd in &model.ext_cmds {
                        fol::handle_model_extcmd(&mut ctx, thcmd);
                    }
                    model.ext_ctx = Some(ModelCtx::FOL(ctx));
                });
                for cmd in fol::to_sygus(&ls_file_mut) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::PML) => {
                process_models_for_logic(&mut ls_file_mut, |model| {
                    let mut ctx = pml::KripkeCtx::default();
                    for thcmd in &model.ext_cmds {
                        pml::handle_model_extcmd(&mut ctx, thcmd);
                    }
                    model.ext_ctx = Some(ModelCtx::PML(ctx));
                });
                for cmd in pml::to_sygus(&ls_file_mut) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::RE) => {
                process_models_for_logic(&mut ls_file_mut, |model| {
                    let mut ctx = re::StringCtx::default();
                    for thcmd in &model.ext_cmds {
                        re::handle_model_extcmd(&mut ctx, thcmd);
                    }
                    model.ext_ctx = Some(ModelCtx::RE(ctx));
                });
                for cmd in re::to_sygus(&ls_file_mut) {
                    println!("{}", cmd);
                }
            }
            _ => {
                warn!("Unknown logic type: {}", interpreter.logic.unwrap());
            }
        }
    }

    Ok(())
}

/// generic function to process model definitions
fn process_models_for_logic<F>(ls_file: &mut ls_parser::parser::ast::LSFile, processor: F)
where
    F: Fn(&mut ls_parser::lang::model::Model),
{
    for cmd in &mut ls_file.cmds {
        if let LSCmd::ModelDef(model) = cmd {
            processor(model);
        }
    }
}

fn main() -> Result<(), io::Error> {
    let args = CLI::parse();

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .format_module_path(false)
            .format_target(false)
            .init();
    }

    let file_path = &args.file;

    if file_path.is_none() {
        return repl::repl();
    }

    let input = fs::read_to_string(file_path.as_ref().unwrap())?;

    if args.as_sygus {
        let sygus_file = sygus_parser::SyGuSFile::from_str(&input.as_str()).unwrap();
        let logic_str = extract_logic_sygus(&sygus_file).unwrap();
        let logic = match logic_str.as_str() {
            "SLIA" => Logic::RE,
            _ => {
                error!("Unsupported logic: {}", logic_str);
                return Err(io::Error::new(io::ErrorKind::Other, "Unsupported logic"));
            }
        };
        println!("{}", logic.desugar(&sygus_file));
        return Ok(());
    }

    process_file(&input, &args)
}
