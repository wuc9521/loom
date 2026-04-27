mod cvc5_sygus;

use clap::Parser as ClapParser;
use delis::{
    backends::smt2,
    compile,
    lang::runtime::Session,
    parser::{load_module_file, parse_commands, DelisCmd, DelisProg},
    process::cleanup_children,
};
use std::{
    fmt::Display,
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Instant,
};

fn solve(delis_file: &mut DelisProg, args: &CLI) -> Result<(), io::Error> {
    if args.dump_chc || args.dump_sygus {
        if args.dump_chc {
            let program = compile::to_chc_ast(delis_file).map_err(io::Error::other)?;
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
            let compile_start = Instant::now();
            let cmds = compile::to_sygus(delis_file, args.optimize).map_err(io::Error::other)?;
            log::info!("to_sygus completed in {:?}", compile_start.elapsed());
            let write_start = Instant::now();
            let stdout = io::stdout();
            let mut stdout = BufWriter::new(stdout.lock());
            write_cmds(&mut stdout, &cmds)?;
            writeln!(stdout)?;
            stdout.flush()?;
            log::info!("write SyGuS completed in {:?}", write_start.elapsed());
        }
        return Ok(());
    }
    let (tx, rx) = mpsc::channel();

    let closure_tx = tx.clone();
    let closure_file = delis_file.clone();
    let closure_optimize = args.optimize;
    const CLOSURE_STACK_BYTES: usize = 64 * 1024 * 1024;
    thread::Builder::new()
        .name("closure-backend".to_string())
        .stack_size(CLOSURE_STACK_BYTES)
        .spawn(move || {
            let result = compile::solve_by_closure(&closure_file, closure_optimize)
                .map_err(io::Error::other);
            let _ = closure_tx.send(BackendResult::Closure(result));
        })?;

    let sygus_tx = tx.clone();
    let sygus_file = delis_file.clone();
    let optimize = args.optimize;
    let enum_trigger = args.enum_trigger;
    const SYGUS_STACK_BYTES: usize = 128 * 1024 * 1024;
    thread::Builder::new()
        .name("sygus-backend".to_string())
        .stack_size(SYGUS_STACK_BYTES)
        .spawn(move || {
            let result = (|| {
                let step_start = Instant::now();
                let bv_sz = compile::bv_sz(&sygus_file).map_err(io::Error::other)?;
                log::info!("sygus-backend: bv_sz completed in {:?}", step_start.elapsed());

                let step_start = Instant::now();
                let sygus_cmds =
                    compile::to_sygus(&sygus_file, optimize).map_err(io::Error::other)?;
                log::info!(
                    "sygus-backend: to_sygus completed in {:?}",
                    step_start.elapsed()
                );

                let step_start = Instant::now();
                let sygus_solver = sygus_config(bv_sz, enum_trigger);
                log::info!(
                    "sygus-backend: sygus_config completed in {:?}",
                    step_start.elapsed()
                );

                let step_start = Instant::now();
                let solved =
                    cvc5_sygus::solve(&sygus_cmds, sygus_solver.use_fast_enum).map_err(io::Error::other)?;
                log::info!(
                    "sygus-backend: native cvc5 solve completed in {:?}",
                    step_start.elapsed()
                );
                Ok(solved)
            })();
            let _ = sygus_tx.send(BackendResult::Sygus(result));
        })?;

    drop(tx);
    let mut pending = 2;
    let mut first_error = None;
    while pending > 0 {
        let result = rx.recv().map_err(io::Error::other)?;
        pending -= 1;
        match result {
            BackendResult::Closure(Ok(Some(output))) | BackendResult::Sygus(Ok(output)) => {
                println!("{output}");
                cleanup_children();
                return Ok(());
            }
            BackendResult::Closure(Ok(None)) => {}
            BackendResult::Closure(Err(e)) | BackendResult::Sygus(Err(e)) => {
                first_error.get_or_insert(e);
            }
        }
    }
    cleanup_children();
    Err(first_error
        .unwrap_or_else(|| io::Error::other("neither closure nor SyGuS backend produced a result")))
}

fn write_cmds<W: Write, T: Display>(writer: &mut W, cmds: &[T]) -> Result<(), io::Error> {
    for (idx, cmd) in cmds.iter().enumerate() {
        if idx > 0 {
            writer.write_all(b"\n")?;
        }
        write!(writer, "{cmd}")?;
    }
    Ok(())
}

struct SolverConfig {
    use_fast_enum: bool,
}

enum BackendResult {
    Closure(Result<Option<String>, io::Error>),
    Sygus(Result<String, io::Error>),
}

fn sygus_config(bv_sz: usize, enum_trigger: usize) -> SolverConfig {
    SolverConfig {
        use_fast_enum: bv_sz >= enum_trigger,
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

    /// Translate to Z3 fixedpoint CHC witness query and print to stdout
    #[arg(long = "dump-chc-witness", default_value_t = false)]
    dump_chc_witness: bool,

    /// Solve using only the Z3 fixedpoint CHC witness path
    #[arg(long = "chc-witness", default_value_t = false)]
    chc_witness: bool,

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

    ctrlc::set_handler(move || {
        cleanup_children();
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

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
                return Err(io::Error::other(format!("parse error: {}", e)));
            }
        }
    }
    let mut delis_file = DelisProg {
        cmds: delis_cmd_vec,
    };
    solve(&mut delis_file, &args)
}
