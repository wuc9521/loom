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
    solve_with_backends(delis_file, args)
}

fn cmds_to_text(cmds: &[impl ToString]) -> String {
    cmds.iter()
        .map(|cmd| cmd.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

type Solver = (String, Vec<String>);

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
    Ok(chc_result(&solver_stdout(child)?).map(str::to_string))
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

fn parse_sygus_define_fun(output: &str) -> Result<Option<SyGuSCmd>, io::Error> {
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

fn decode_sygus_output(delis_file: &DelisProg, output: &str) -> Result<String, io::Error> {
    let Some(cmd) = parse_sygus_define_fun(output)? else {
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
    if compile::uses_semantic_domains(delis_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    {
        let sygus_text = cmds_to_text(
            &compile::to_sygus(delis_file, args.optimize)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        );
        println!(
            "{}",
            decode_sygus_output(
                delis_file,
                &run_sygus_solver(&args.sygus_solver_config(), &sygus_text)?
            )?
        );
        return Ok(());
    }

    let chc_program =
        compile::to_chc_ast(delis_file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let chc_text = cmds_to_text(&smt2::emit::to_smt(&chc_program));
    let sygus_text = cmds_to_text(
        &compile::to_sygus(delis_file, args.optimize)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
    );
    let chc_solver = args.chc_solver_config();
    let sygus_solver = args.sygus_solver_config();
    if args.linear {
        match run_chc_solver(&chc_solver, &chc_text)?.as_deref() {
            Some("sat") => {
                println!("unsat");
                return Ok(());
            }
            Some("unsat") | None => {
                println!(
                    "{}",
                    decode_sygus_output(
                        delis_file,
                        &run_sygus_solver(&sygus_solver, &sygus_text)?
                    )?
                );
                return Ok(());
            }
            _ => unreachable!(),
        }
    }
    solve_parallel(
        delis_file,
        &chc_text,
        &sygus_text,
        &chc_solver,
        &sygus_solver,
    )
}

fn solve_parallel(
    delis_file: &DelisProg,
    chc_text: &str,
    sygus_text: &str,
    chc_solver: &Solver,
    sygus_solver: &Solver,
) -> Result<(), io::Error> {
    let mut chc = spawn_solver(chc_solver, chc_text)?;
    let mut sygus = spawn_solver(sygus_solver, sygus_text)?;
    loop {
        if chc.try_wait()?.is_some() {
            let chc_stdout = solver_stdout(chc)?;
            if chc_result(&chc_stdout) == Some("sat") {
                let _ = sygus.kill();
                let _ = sygus.wait();
                println!("unsat");
                return Ok(());
            }
            println!(
                "{}",
                decode_sygus_output(delis_file, &solver_stdout(sygus)?)?
            );
            return Ok(());
        }
        if sygus.try_wait()?.is_some() {
            let sygus_stdout = solver_stdout(sygus)?;
            let _ = chc.kill();
            let _ = chc.wait();
            println!("{}", decode_sygus_output(delis_file, &sygus_stdout)?);
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

    /// Run CHC first and SyGuS second instead of running both solvers in parallel
    #[arg(long = "linear", default_value_t = false)]
    linear: bool,

    /// CHC solver command. Defaults to `z3`.
    #[arg(long = "chc-solver", value_name = "COMMAND")]
    chc_solver: Option<String>,

    /// CHC solver argument. May be repeated. Defaults to `-in` unless solver command or arguments are overridden.
    #[arg(
        long = "chc-solver-arg",
        value_name = "ARG",
        action = clap::ArgAction::Append,
        allow_hyphen_values = true
    )]
    chc_solver_args: Vec<String>,

    /// SyGuS solver command. Defaults to `cvc5`.
    #[arg(long = "sygus-solver", value_name = "COMMAND")]
    sygus_solver: Option<String>,

    /// SyGuS solver argument. May be repeated. Defaults to `--lang=sygus2` unless solver command or arguments are overridden.
    #[arg(
        long = "sygus-solver-arg",
        value_name = "ARG",
        action = clap::ArgAction::Append,
        allow_hyphen_values = true
    )]
    sygus_solver_args: Vec<String>,
}

impl CLI {
    fn chc_solver_config(&self) -> Solver {
        let command = self.chc_solver.clone().unwrap_or_else(|| "z3".to_string());
        let args = if self.chc_solver.is_none() && self.chc_solver_args.is_empty() {
            vec!["-in".to_string()]
        } else {
            self.chc_solver_args.clone()
        };
        (command, args)
    }

    fn sygus_solver_config(&self) -> Solver {
        let command = self
            .sygus_solver
            .clone()
            .unwrap_or_else(|| "cvc5".to_string());
        let args = if self.sygus_solver.is_none() && self.sygus_solver_args.is_empty() {
            vec!["--lang=sygus2".to_string()]
        } else {
            self.sygus_solver_args.clone()
        };
        (command, args)
    }
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
