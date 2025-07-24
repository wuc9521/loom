use clap::Parser as ClapParser;
use ls_parser::{
    parser::*,
    theory::{
        dispatch_extcmd,
        fol::FOLCtx,
        pml::{self, PMLCtx},
        re::{self, RECtx},
        Ctx, Logic,
    },
    utils::*,
};
use std::{fs, io::{self, Write}, path::PathBuf};

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
    #[arg(long = "to-bv", default_value_t = false)]
    to_bv: bool,

    #[arg(long = "from-sygus", default_value_t = false)]
    from_sygus: bool,
}

/// Run the language parsing application
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    if let Some(file_path) = &args.file {
        let input = fs::read_to_string(file_path)?;
        if args.from_sygus {
            let sygus_file = sygus_parser::SyGuSFile::from_str(&input.as_str()).unwrap();
            let logic_str = extract_logic_sygus(&sygus_file).unwrap();
            match logic_str {
                "SLIA" => Logic::SLIA,
                _ => {
                    eprintln!("Unsupported logic: {}", logic_str);
                    return Err(io::Error::new(io::ErrorKind::Other, "Unsupported logic"));
                }
            };
            let ls_file = re::sugar::desugar(&sygus_file);
            println!("{}", ls_file);
            return Ok(());
        }
        let mut problem = file::parse(&input.as_str()).unwrap();

        let logic_str = extract_logic(&problem).unwrap();
        let logic = match logic_str {
            "FOL" => Logic::FOL,
            "PML" => Logic::PML,
            "RE" => Logic::RE,
            _ => {
                eprintln!("Error: Unknown logic: {}", logic_str);
                return Err(io::Error::new(io::ErrorKind::Other, "Unknown logic"));
            }
        };

        // --to-bv dispatch
        if args.to_bv {
            match logic {
                Logic::PML => {
                    let cmds = pml::bv::translate_to_bv_cmds(&problem);
                    for cmd in cmds {
                        println!("{}", cmd);
                    }
                }
                Logic::RE => {
                    let cmds = re::bv::translate_to_bv_cmds(&problem);
                    for cmd in cmds {
                        println!("{}", cmd);
                    }
                }
                _ => {
                    eprintln!(";; Translation for this logic is not implemented.");
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "to-bv not implemented for this logic",
                    ));
                }
            }
        }

        // collect all ext_cmds
        let ext_cmds: Vec<_> = problem
            .cmds
            .iter()
            .filter_map(|cmd| {
                if let LSCmd::ThSpecCmd(thcmd) = cmd {
                    Some(thcmd.clone())
                } else {
                    None
                }
            })
            .collect();

        for i in 0..problem.cmds.len() {
            if let LSCmd::ModelDef(model) = &mut problem.cmds[i] {
                let mut ctx = match logic {
                    Logic::FOL => Ctx::FOL(FOLCtx::default()),
                    Logic::PML => Ctx::PML(PMLCtx::default()),
                    Logic::RE => Ctx::RE(RECtx::default()),
                    _ => unreachable!(),
                };
                for thcmd in &ext_cmds {
                    dispatch_extcmd(logic, &mut ctx, thcmd);
                }
                for extcmd in &model.ext_cmds {
                    dispatch_extcmd(logic, &mut ctx, extcmd);
                }
                model.ext_ctx = Some(ctx.clone());
            }
        }

        if let Some(dump_path) = &args.dump {
            use ls_parser::theory::fol::model_to_json_with_symbol_map as fol_model_to_json;
            use ls_parser::theory::pml::model_to_json_with_symbol_map as pml_model_to_json;
            use std::fs::{create_dir_all, File};
            if !dump_path.exists() {
                create_dir_all(dump_path).expect("Failed to create dump directory");
            }
            if !dump_path.is_dir() {
                eprintln!("Error: --dump path must be a directory");
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Dump path must be a directory",
                ));
            }
            for cmd in &problem.cmds {
                if let LSCmd::ModelDef(model) = cmd {
                    match &model.ext_ctx {
                        Some(Ctx::FOL(fol_ctx)) => {
                            let json = fol_model_to_json(model, fol_ctx);
                            let file_path = dump_path.join(format!("{}.json", model.name));
                            let mut file =
                                File::create(&file_path).expect("Failed to create dump file");
                            file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                                .expect("Failed to write dump file");
                        }
                        Some(Ctx::PML(pml_ctx)) => {
                            let json = pml_model_to_json(model, pml_ctx);
                            let file_path = dump_path.join(format!("{}.json", model.name));
                            let mut file =
                                File::create(&file_path).expect("Failed to create dump file");
                            file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                                .expect("Failed to write dump file");
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    } else {
        println!("ls-parser interactive mode (type :quit or :exit to exit)");
        let mut buffer = String::new();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            if io::stdin().read_line(&mut line)? == 0 {
                break;
            }
            let line = line.trim_end();
            if line == ":quit" || line == ":exit" {
                break;
            }
            buffer.push_str(line);
            buffer.push('\n');
            match file::parse(&buffer) {
                Ok(_) => {
                    println!("parse success, type :quit or :exit to exit");
                }
                Err(e) => {
                    println!("parse error: {:?}", e);
                }
            }
        }
        Ok(())
    }
}
