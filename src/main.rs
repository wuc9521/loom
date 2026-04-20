use clap::Parser as ClapParser;
use delis::{
    backends::smt2,
    compile,
    lang::runtime::Session,
    parser::{load_module_file, parse_commands, DelisCmd, DelisProg},
};
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
};
use sygus_parser::{Sort, SyGuSCmd, SyGuSFile};

fn solve(delis_file: &mut DelisProg, args: &CLI) -> Result<(), io::Error> {
    if args.dump_chc || args.dump_sygus || args.dump_semgus {
        if args.dump_chc {
            let program = compile::to_chc_ast(delis_file)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            println!(
                "{}",
                smt2::emit::to_smt(&program)
                    .iter()
                    .map(|sexpr| sexpr.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        if args.dump_sygus {
            let cmds = compile::to_sygus(delis_file, args.optimize)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            println!(
                "{}",
                cmds.iter()
                    .map(|cmd| cmd.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        if args.dump_semgus {
            println!(
                "{}",
                delis::backends::semgus::compile(delis_file)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            );
        }
        return Ok(());
    }
    let (tx, rx) = mpsc::channel();

    let chc_tx = tx.clone();
    let chc_file = delis_file.clone();
    thread::Builder::new()
        .name("chc-backend".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let result = (|| {
                let chc_program = compile::to_chc_ast(&chc_file)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let chc_text = cmds_to_text(&smt2::emit::to_smt(&chc_program));
                run_chc_solver(&chc_solver_config(), &chc_text)
            })();
            let _ = chc_tx.send(BackendResult::Chc(result));
        })?;

    let sygus_tx = tx.clone();
    let sygus_file = delis_file.clone();
    let optimize = args.optimize;
    let enum_trigger = args.enum_trigger;
    thread::Builder::new()
        .name("sygus-backend".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let result = (|| {
                let bv_sz = compile::bv_sz(&sygus_file)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let sygus_cmds = compile::to_sygus(&sygus_file, optimize)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let sygus_text = cmds_to_text(&sygus_cmds);
                let sygus_solver = cvc5_config(bv_sz, enum_trigger);
                let output = run_sygus_solver(&sygus_solver, &sygus_text)?;
                decode_cvc5_response(&sygus_file, &output)
            })();
            let _ = sygus_tx.send(BackendResult::Sygus(result));
        })?;

    drop(tx);
    let mut pending = 2;
    let mut first_error = None;
    while pending > 0 {
        let result = rx
            .recv()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        pending -= 1;
        match result {
            BackendResult::Chc(Ok(Some(output))) | BackendResult::Sygus(Ok(output)) => {
                println!("{output}");
                return Ok(());
            }
            BackendResult::Chc(Ok(None)) => {}
            BackendResult::Chc(Err(e)) | BackendResult::Sygus(Err(e)) => {
                first_error.get_or_insert(e);
            }
        }
    }
    Err(first_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "neither CHC nor SyGuS backend produced a result",
        )
    }))
}

fn cmds_to_text(cmds: &[impl ToString]) -> String {
    cmds.iter()
        .map(|cmd| cmd.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

type Solver = (String, Vec<String>);

enum BackendResult {
    Chc(Result<Option<String>, io::Error>),
    Sygus(Result<String, io::Error>),
}

fn chc_solver_config() -> Solver {
    ("cvc5".to_string(), vec![])
}

fn cvc5_config(bv_sz: usize, enum_trigger: usize) -> Solver {
    let mut args = vec!["--lang=sygus2".to_string()];
    if bv_sz >= enum_trigger {
        args.push("--sygus-enum=fast".to_string());
    }
    ("cvc5".to_string(), args)
}

fn spawn_solver(solver: &Solver, input: &str) -> Result<Child, io::Error> {
    let mut child = Command::new(&solver.0)
        .args(&solver.1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }
    Ok(child)
}

fn solver_stdout(child: Child) -> Result<String, io::Error> {
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn chc_result(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| *line == "sat" || *line == "unsat")
}

fn run_chc_solver(solver: &Solver, chc_text: &str) -> Result<Option<String>, io::Error> {
    let child = spawn_solver(solver, chc_text)?;
    Ok(match chc_result(&solver_stdout(child)?) {
        Some("sat") => Some("unsat".to_string()),
        _ => None,
    })
}

fn run_sygus_solver(solver: &Solver, sygus_text: &str) -> Result<String, io::Error> {
    let child = spawn_solver(solver, sygus_text)?;
    Ok(solver_stdout(child)?.trim().to_string())
}

fn synth_signature(
    delis_file: &DelisProg,
) -> Result<(String, Vec<sygus_parser::SortedVar>, Sort), io::Error> {
    delis_file
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::As(SyGuSCmd::SynthFun(name, args, sort, _)) => {
                Some((name.clone(), args.clone(), sort.clone()))
            }
            _ => None,
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing synth-fun command"))
}

fn decode_cvc5_response(delis_file: &DelisProg, output: &str) -> Result<String, io::Error> {
    let trimmed = output.trim();
    let content = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let Ok(file) = SyGuSFile::from_str(content) else {
        return Ok(trimmed.to_string());
    };
    let (synth_name, synth_args, synth_sort) = synth_signature(delis_file)?;
    let Some(SyGuSCmd::DefineFun(name, _args, _sort, body)) = file
        .cmds
        .into_iter()
        .find(|cmd| matches!(cmd, SyGuSCmd::DefineFun(..)))
    else {
        return Ok(trimmed.to_string());
    };
    let decoded = SyGuSCmd::DefineFun(
        if name == synth_name { name } else { synth_name },
        synth_args,
        synth_sort,
        body,
    );
    Ok(format!("(\n{}\n)", decoded))
}

/// CLI application for parsing your custom language
#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "Parse and analyze your custom language files"
)]
pub struct CLI {
    /// Path to the input file (if not using --in)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Read input from stdin instead of file
    #[arg(short, long)]
    r#in: bool,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Enable SyGuS optimization passes (use -O=false or --optimize=false to disable)
    #[arg(
        short = 'O',
        long = "optimize",
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        value_parser = clap::builder::BoolishValueParser::new()
    )]
    optimize: bool,

    /// Translate to SyGuS BV and print to stdout
    #[arg(short, long = "dump-sygus", default_value_t = false)]
    dump_sygus: bool,

    /// Translate to CHC (HORN) and print to stdout
    #[arg(long = "dump-chc", default_value_t = false)]
    dump_chc: bool,

    /// Translate to SemGuS and print to stdout
    #[arg(long = "dump-semgus", default_value_t = false)]
    dump_semgus: bool,

    /// Use `cvc5 --sygus-enum=fast` when the semantic BV encoding length is at least this value.
    #[arg(long = "enum-trigger", default_value_t = 512)]
    enum_trigger: usize,
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
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
        .init();

    // Read input from stdin or file
    let input = if args.r#in {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        // Read from file
        let file_path = args.file.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Either provide a file path or use --in to read from stdin",
            )
        })?;
        load_module_file(file_path)?
    };
    let mut session = Session::new();
    let results = parse_commands(&input, Some(&mut session.runtime));
    let mut delis_cmd_vec: Vec<DelisCmd> = vec![];
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
    let mut delis_file = DelisProg {
        cmds: delis_cmd_vec,
    };
    solve(&mut delis_file, &args)
}
