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
    thread,
    time::Duration,
};
use sygus_parser::{Sort, SyGuSCmd};

fn solve_general(delis_file: &mut DelisProg, args: &CLI) -> Result<(), io::Error> {
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
            let cmds = compile::to_sygus(delis_file)
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
    solve_with_backends(delis_file, args)
}

fn cmds_to_text(cmds: &[impl ToString]) -> String {
    cmds.iter()
        .map(|cmd| cmd.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn spawn_solver(command: &str, args: &[&str], input: &str) -> Result<Child, io::Error> {
    let mut child = Command::new(command)
        .args(args)
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

fn z3_result(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| *line == "sat" || *line == "unsat")
}

fn run_z3(chc_text: &str) -> Result<Option<String>, io::Error> {
    let child = spawn_solver("z3", &["-in"], chc_text)?;
    Ok(z3_result(&solver_stdout(child)?).map(str::to_string))
}

fn run_cvc5(sygus_text: &str) -> Result<String, io::Error> {
    let child = spawn_solver("cvc5", &["--lang=sygus2"], sygus_text)?;
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

fn parse_cvc5_define_fun(output: &str) -> Result<Option<SyGuSCmd>, io::Error> {
    let trimmed = output.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("unknown")
        || trimmed.eq_ignore_ascii_case("infeasible")
        || trimmed.eq_ignore_ascii_case("fail")
    {
        return Ok(None);
    }
    let content = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let file = sygus_parser::SyGuSFile::from_str(content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(file
        .cmds
        .into_iter()
        .find(|cmd| matches!(cmd, SyGuSCmd::DefineFun(..))))
}

fn decode_cvc5_output(delis_file: &DelisProg, output: &str) -> Result<String, io::Error> {
    let Some(cmd) = parse_cvc5_define_fun(output)? else {
        return Ok(output.trim().to_string());
    };
    let (synth_name, synth_args, synth_sort) = synth_signature(delis_file)?;
    let SyGuSCmd::DefineFun(name, _args, _sort, body) = cmd else {
        unreachable!();
    };
    let decoded = SyGuSCmd::DefineFun(
        if name == synth_name { name } else { synth_name },
        synth_args,
        synth_sort,
        body,
    );
    Ok(format!("(\n{}\n)", decoded))
}

fn solve_with_backends(delis_file: &DelisProg, args: &CLI) -> Result<(), io::Error> {
    let chc_program =
        compile::to_chc_ast(delis_file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let chc_text = cmds_to_text(&smt2::emit::to_smt(&chc_program));
    let sygus_text = cmds_to_text(
        &compile::to_sygus(delis_file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
    );
    if args.linear {
        match run_z3(&chc_text)?.as_deref() {
            Some("unsat") => {
                println!("unsat");
                return Ok(());
            }
            Some("sat") | None => {
                println!(
                    "{}",
                    decode_cvc5_output(delis_file, &run_cvc5(&sygus_text)?)?
                );
                return Ok(());
            }
            _ => unreachable!(),
        }
    }
    solve_parallel(delis_file, &chc_text, &sygus_text)
}

fn solve_parallel(
    delis_file: &DelisProg,
    chc_text: &str,
    sygus_text: &str,
) -> Result<(), io::Error> {
    let mut z3 = spawn_solver("z3", &["-in"], chc_text)?;
    let mut cvc5 = spawn_solver("cvc5", &["--lang=sygus2"], sygus_text)?;
    loop {
        if z3.try_wait()?.is_some() {
            let z3_stdout = solver_stdout(z3)?;
            if z3_result(&z3_stdout) == Some("unsat") {
                let _ = cvc5.kill();
                let _ = cvc5.wait();
                println!("unsat");
                return Ok(());
            }
            println!("{}", decode_cvc5_output(delis_file, &solver_stdout(cvc5)?)?);
            return Ok(());
        }
        if cvc5.try_wait()?.is_some() {
            let cvc5_stdout = solver_stdout(cvc5)?;
            let _ = z3.kill();
            let _ = z3.wait();
            println!("{}", decode_cvc5_output(delis_file, &cvc5_stdout)?);
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
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

    /// Translate to SyGuS BV and print to stdout
    #[arg(short, long = "dump-sygus", default_value_t = false)]
    dump_sygus: bool,

    /// Translate to CHC (HORN) and print to stdout
    #[arg(long = "dump-chc", default_value_t = false)]
    dump_chc: bool,

    /// Translate to SemGuS and print to stdout
    #[arg(long = "dump-semgus", default_value_t = false)]
    dump_semgus: bool,

    /// Run CHC first and SyGuS second instead of running both solvers in parallel
    #[arg(long = "linear", default_value_t = false)]
    linear: bool,
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
    solve_general(&mut delis_file, &args)
}
