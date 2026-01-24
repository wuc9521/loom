use clap::Parser as ClapParser;
use delis::{
    lang::runtime::Session,
    parser::ast::DelisCmd,
    theory::{fol, pml, re, Decoder, Logic, ModelCtx, SygusDesugar},
    utils::extract_logic_sygus,
};
use log::{error, warn};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

/// Parse cvc5 output: extract define-fun commands from big parenthesis
fn parse_cvc5_define_funs(output: &str) -> Result<Vec<sygus_parser::SyGuSCmd>, io::Error> {
    let trimmed = output.trim();
    let content = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let mut define_funs = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in content.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '(' if !in_string => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    let cmd_str = &content[start..=i];
                    match sygus_parser::SyGuSCmd::from_str(cmd_str) {
                        Ok(cmd) if matches!(cmd, sygus_parser::SyGuSCmd::DefineFun(..)) => {
                            define_funs.push(cmd);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("Failed to parse define-fun: {} in '{}'", e, cmd_str),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if define_funs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No define-fun commands found in cvc5 output",
        ));
    }
    Ok(define_funs)
}

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
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Error => "\x1b[31mERROR\x1b[0m",
                log::Level::Warn => "\x1b[33mWARN\x1b[0m",
                log::Level::Info => "\x1b[34mINFO\x1b[0m",
                log::Level::Debug => "\x1b[35mDEBUG\x1b[0m",
                log::Level::Trace => "\x1b[36mTRACE\x1b[0m",
            };
            let msg = if record.target() == "parsed" {
                format!("\x1b[32m{}\x1b[0m", record.args())
            } else {
                record.args().to_string()
            };
            writeln!(
                buf,
                "{} {} {}:{}",
                level,
                msg,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0)
            )
        })
        .filter_level(if args.verbose {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Warn
        })
        .init();
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
            let chc_cmds = re::to_chc(&delis_file, args.verbose);
            let chc_text = chc_cmds
                .iter()
                .map(|cmd| cmd.to_string())
                .collect::<Vec<_>>()
                .join("\n");

            if args.dump_chc {
                println!("{}", chc_text);
                return Ok(());
            }

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

            // Check z3 result
            let mut z3_result = None;
            for line in stdout.lines() {
                let trimmed = line.trim();
                if (trimmed == "sat" || trimmed == "unsat") && z3_result.is_none() {
                    z3_result = Some(trimmed);
                } else if args.verbose && !trimmed.is_empty() {
                    error!("z3: {}", trimmed);
                }
            }

            match z3_result {
                Some("unsat") => {
                    println!("unsat");
                    return Ok(());
                }
                Some("sat") => {
                    // Generate sygus and call cvc5
                    let sygus_cmds = re::to_sygus(&delis_file);
                    let sygus_text = sygus_cmds
                        .iter()
                        .map(|cmd| cmd.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");

                    let mut cvc5 = Command::new("cvc5")
                        .arg("--lang=sygus2")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?;

                    if let Some(mut stdin) = cvc5.stdin.take() {
                        stdin.write_all(sygus_text.as_bytes())?;
                    }

                    let cvc5_output = cvc5.wait_with_output()?;
                    let cvc5_stdout = String::from_utf8_lossy(&cvc5_output.stdout);
                    let cvc5_stderr = String::from_utf8_lossy(&cvc5_output.stderr);

                    if !cvc5_stderr.is_empty() && args.verbose {
                        eprint!("{}", cvc5_stderr);
                    }

                    // Parse and decode cvc5 output
                    let define_funs = parse_cvc5_define_funs(&cvc5_stdout)?;
                    match Logic::RE.decode(&define_funs) {
                        Ok(result) => println!("(\n{}\n)", result),
                        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
                    }
                }
                _ => {
                    if z3_result.is_none() {
                        warn!("z3 returned no sat/unsat result");
                    }
                }
            }

            if args.dump_sygus {
                for cmd in re::to_sygus(&delis_file) {
                    println!("{}", cmd);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
