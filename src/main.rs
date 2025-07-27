use clap::Parser as ClapParser;
use ls_parser::{
    lang::runtime::Session,
    parser::ast::LSCmd,
    theory::{fol, pml, re, Logic, ModelCtx, SygusDesugar},
    utils::extract_logic_sygus,
};
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

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    let file_path = args.file;
    // let theory_file = args.theory_file.unwrap();
    if file_path == None {
        return repl::repl();
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
    let results = ls_parser::parser::ast::parse_commands(&input, Some(&mut session.runtime));
    let mut lscmd_vec: Vec<ls_parser::parser::ast::LSCmd> = vec![];
    for res in results {
        match res {
            Ok(cmd) => {
                let _ = session.handle_command(&cmd);
                lscmd_vec.push(cmd);
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)));
            }
        }
    }
    let mut ls_file = ls_parser::parser::ast::LSFile { cmds: lscmd_vec };
    if args.dump_ls {
        for cmd in &ls_file.cmds {
            println!("{}", cmd);
        }
    }
    if args.dump_sygus {
        match session.logic {
            Some(Logic::FOL) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = fol::FOModelCtx::default();
                        for thcmd in &model.ext_cmds {
                            fol::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::FOL(ctx));
                    }
                }
                for cmd in fol::to_sygus(&ls_file) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::PML) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = pml::KripkeCtx::default();
                        for thcmd in &model.ext_cmds {
                            pml::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::PML(ctx));
                    }
                }
                for cmd in pml::to_sygus(&ls_file) {
                    println!("{}", cmd);
                }
            }
            Some(Logic::RE) => {
                for cmd in &mut ls_file.cmds {
                    if let LSCmd::DefineModel(model) = cmd {
                        let mut ctx = re::StringCtx::default();
                        for thcmd in &model.ext_cmds {
                            re::handle_model_extcmd(&mut ctx, thcmd);
                        }
                        model.ext_ctx = Some(ModelCtx::RE(ctx));
                    }
                }
                for cmd in re::to_sygus(&ls_file) {
                    println!("{}", cmd);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
