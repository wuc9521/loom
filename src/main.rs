use clap::Parser as ClapParser;
use delis::{
    lang::runtime::Session,
    parser::ast::DelisCmd,
    theory::{fol, pml, re, Logic, ModelCtx, SygusDesugar},
    utils::extract_logic_sygus,
};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
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
            if args.dump_sygus {
                for cmd in fol::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
            if args.dump_chc {}
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
        }
        Some(Logic::RE) => {
            for cmd in &mut delis_file.cmds {
                if let DelisCmd::DefineModel(model) = cmd {
                    model.ext_ctx = Some(ModelCtx::RE(re::StringCtx::default()));
                }
            }
            let chc_cmds = re::to_chc(&delis_file);
            let chc_text = chc_cmds
                .iter()
                .map(|cmd| cmd.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            let mut child = Command::new("z3")
                .arg("-in")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(chc_text.as_bytes())?;
            }
            let output = child.wait_with_output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut result = None;
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed == "sat" || trimmed == "unsat" {
                    result = Some(trimmed.to_string());
                    break;
                } else {
                    // error
                    eprintln!("Z3 output: {}", trimmed);
                    return Ok(());
                }
            }
            if result.as_deref() == Some("unsat") {
                println!("unsat");
            } else {
                // run saturate.
            }
            if args.dump_sygus {
                for cmd in re::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
            if args.dump_chc {
                println!("{}", chc_text);
            }
        }
        _ => {}
    }
    Ok(())
}
