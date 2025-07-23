use clap::Parser as ClapParser;
use ls_parser::{
    parser::*,
    theory::{dispatch_extcmd, fol::FOLCtx, pml::PMLCtx, Ctx, Logic},
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

    /// Dump model info as JSON to this path
    #[arg(short = 'd', long = "dump")]
    dump: Option<PathBuf>,

    /// Translate to SyGuS BV and print to stdout
    #[arg(long = "to-bv", default_value_t = false)]
    to_bv: bool,
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
    let mut problem = file::parse(&input.as_str()).unwrap();

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

    // --to-bv 分流
    if args.to_bv {
        match logic {
            Logic::PML => {
                use ls_parser::theory::pml::bv::translate_to_bv;
                let bv_str = translate_to_bv(&problem);
                println!("{}", bv_str);
                return Ok(());
            }
            // 其它逻辑分流可在此扩展
            _ => {
                eprintln!("[to-bv] Translation for this logic is not implemented.");
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "to-bv not implemented for this logic",
                ));
            }
        }
    }

    // 先收集所有 ext_cmd
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

    // 初始化 context
    let ctx = match logic {
        Logic::FOL => Ctx::FOL(FOLCtx::default()),
        Logic::PML => Ctx::PML(PMLCtx::default()),
        _ => unreachable!(),
    };

    for i in 0..problem.cmds.len() {
        if let LSCmd::ModelDef(model) = &mut problem.cmds[i] {
            let mut ctx = match logic {
                Logic::FOL => Ctx::FOL(FOLCtx::default()),
                Logic::PML => Ctx::PML(PMLCtx::default()),
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

    if args.verbose {
        println!("Parsed problem:\n {:?}", problem);
        println!("Context: {:?}", ctx);
    }

    if let Some(dump_path) = &args.dump {
        use ls_parser::theory::fol::model_to_json_with_symbol_map as fol_model_to_json;
        use ls_parser::theory::pml::model_to_json_with_symbol_map as pml_model_to_json;
        use std::fs::{create_dir_all, File};
        use std::io::Write;
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
            eprintln!("[MAIN DEBUG] LSCmd variant: {:?}", cmd);
            if let LSCmd::ModelDef(model) = cmd {
                match &model.ext_ctx {
                    Some(Ctx::FOL(fol_ctx)) => {
                        eprintln!("[MAIN DEBUG] model {} ext_ctx is FOL", model.name);
                        let json = fol_model_to_json(model, fol_ctx);
                        let file_path = dump_path.join(format!("{}.json", model.name));
                        let mut file =
                            File::create(&file_path).expect("Failed to create dump file");
                        file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                            .expect("Failed to write dump file");
                        println!(
                            "Model {} info dumped to {}",
                            model.name,
                            file_path.display()
                        );
                    }
                    Some(Ctx::PML(pml_ctx)) => {
                        eprintln!("[MAIN DEBUG] model {} ext_ctx is PML", model.name);
                        let json = pml_model_to_json(model, pml_ctx);
                        let file_path = dump_path.join(format!("{}.json", model.name));
                        let mut file =
                            File::create(&file_path).expect("Failed to create dump file");
                        file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                            .expect("Failed to write dump file");
                        println!(
                            "Model {} info dumped to {}",
                            model.name,
                            file_path.display()
                        );
                    }
                    other => {
                        eprintln!(
                            "[MAIN DEBUG] model {} ext_ctx is OTHER: {:?}",
                            model.name, other
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
