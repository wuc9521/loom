use clap::Parser as ClapParser;
use delis::{
    compile,
    lang::runtime::Session,
    parser::{load_module_file, DelisCmd, DelisProg},
    process::{cleanup_children, register_pid, unregister_pid},
};

// mimalloc as the global allocator. Profiles showed `__rust_alloc`,
// `__rust_dealloc`, and `Arc::drop_slow` in the top self-time entries on
// the bisim and exact-nullary hot paths; mimalloc's per-thread heaps and
// faster small-allocation path generally give 5-15% on workloads like
// ours where short-lived `Vec` / `HashMap` / `Arc` churn dominates.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc},
    thread,
    time::Instant,
};

mod cegis;
use sygus_parser::SyGuSCmd;


fn solve(delis_file: &mut DelisProg, args: &CLI) -> Result<(), io::Error> {
    if args.dump_chc || args.dump_sygus || args.dump_sygus_stats {
        if args.dump_chc {
            let program = if std::env::var("DELIS_DUMP_CHC_SATURATED").is_ok() {
                delis_file.to_chc_ast_saturated().map_err(io::Error::other)?
            } else {
                delis_file.to_chc_ast().map_err(io::Error::other)?
            };
            println!(
                "{}",
                program.to_smt()
                    .iter()
                    .map(|sexpr| sexpr.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        if args.dump_sygus || args.dump_sygus_stats {
            let compile_start = Instant::now();
            let sygus_bytes =
                delis_file.to_sygus_bytes().map_err(io::Error::other)?;
            if args.dump_sygus {
                let stdout = io::stdout();
                let mut stdout = BufWriter::new(stdout.lock());
                stdout.write_all(&sygus_bytes)?;
                stdout.flush()?;
            }
            if args.dump_sygus_stats {
                let total_ms = compile_start.elapsed().as_millis();
                let bytes = sygus_bytes.len();
                let lines = sygus_bytes.iter().filter(|byte| **byte == b'\n').count();
                println!("SYGUS_STATS\t{}\t{}\t{}", total_ms, bytes, lines);
            }
        }
        return Ok(());
    }

    if args.r#enum {
        let result =
            delis_file.solve_by_closure().map_err(io::Error::other)?;
        if let Some(output) = result {
            println!("{output}");
            return Ok(());
        }
        return Err(io::Error::other(
            "enumerative backend did not produce a result",
        ));
    }

    // Both backends solve the same SyGuS BV problem; first-to-finish wins.
    // The "term enumerator" works at the SyGuSTerm level (fast for small/nullary
    // problems). The "BV searcher" compiles to BitCircuits and does bottom-up
    // enumeration with dedup + composition (handles larger problems).
    let semantic_program = delis_file.semantic_program().map_err(io::Error::other)?;
    let sygus_cmds: Arc<Vec<SyGuSCmd>> = Arc::new(
        delis_file
            .to_sygus_with(semantic_program)
            .map_err(io::Error::other)?,
    );

    let (tx, rx) = mpsc::channel();

    let term_tx = tx.clone();
    let term_file = delis_file.clone();
    let term_cmds = sygus_cmds.clone();
    const TERM_STACK_BYTES: usize = 64 * 1024 * 1024;
    thread::Builder::new()
        .name("term-enumerator".to_string())
        .stack_size(TERM_STACK_BYTES)
        .spawn(move || {
            let result = term_file
                .solve_by_closure_with_cmds((*term_cmds).clone())
                .map_err(io::Error::other);
            let _ = term_tx.send(BackendResult::Closure(result));
        })?;

    let bv_tx = tx.clone();
    let bv_file = delis_file.clone();
    let bv_cmds = sygus_cmds.clone();
    let bv_cegis = if args.cegis > 0 { Some(args.cegis) } else { None };
    let bv_backend = SygusBackend::from_cli(args);
    let bv_verbose = args.verbose;
    let bv_min = args.min;
    const BV_STACK_BYTES: usize = 128 * 1024 * 1024;
    thread::Builder::new()
        .name("bv-searcher".to_string())
        .stack_size(BV_STACK_BYTES)
        .spawn(move || {
            let result = solve_with_sygus(
                &bv_file,
                &bv_backend,
                bv_cegis,
                bv_verbose,
                bv_min,
                bv_cmds,
            );
            let _ = bv_tx.send(BackendResult::Sygus(result));
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

enum BackendResult {
    Closure(Result<Option<String>, io::Error>),
    Sygus(Result<String, io::Error>),
}

type Solver = (String, Vec<String>);

#[derive(Clone, Debug)]
enum SygusBackend {
    InProcessDryadSynth,
    External(Solver),
}

impl SygusBackend {
    fn from_cli(args: &CLI) -> Self {
        match &args.solver {
            Some(path) => Self::External((path.clone(), external_solver_args(path))),
            None => Self::InProcessDryadSynth,
        }
    }
}

fn external_solver_args(path: &str) -> Vec<String> {
    let path_buf = PathBuf::from(path);
    let name = path_buf
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path);
    match name {
        "dryadsynth-bv" => vec!["-in".to_string()],
        "cvc5" => vec!["--lang=sygus2".to_string()],
        _ => Vec::new(),
    }
}

fn spawn_solver(
    solver: &Solver,
    cmds: &[SyGuSCmd],
    min: bool,
) -> Result<Child, io::Error> {
    let mut args = solver.1.clone();
    if !min {
        let path_buf = PathBuf::from(&solver.0);
        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&solver.0);
        if name == "dryadsynth-bv" {
            args.push("--min=false".to_string());
        }
    }

    let mut input = Vec::new();
    sygus_parser::writer::write_cmds(&mut input, cmds)?;
    writeln!(&mut input)?;

    let mut child = Command::new(&solver.0)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    register_pid(child.id());
    if let Some(mut stdin) = child.stdin.take() {
        thread::Builder::new()
            .name("sygus-stdin-writer".to_string())
            .spawn(move || {
                let _ = stdin.write_all(&input);
                let _ = stdin.flush();
            })?;
    }
    Ok(child)
}

fn solve_cmds_with_backend(
    backend: &SygusBackend,
    cmds: &[SyGuSCmd],
    verbose: u8,
    min: bool,
) -> Result<String, io::Error> {
    match backend {
        SygusBackend::InProcessDryadSynth => dryadsynth_bv::solve_sygus_cmds(
            cmds,
            None,
            "",
            verbose,
            min,
        )
        .map_err(io::Error::other),
        SygusBackend::External(solver) => {
            let child = spawn_solver(solver, cmds, min)?;
            solver_stdout(child, &solver.0)
        }
    }
}

fn solve_with_sygus(
    delis_file: &DelisProg,
    backend: &SygusBackend,
    cegis_size: Option<usize>,
    verbose: u8,
    min: bool,
    sygus_cmds: Arc<Vec<SyGuSCmd>>,
) -> Result<String, io::Error> {
    let solve_start = Instant::now();
    let decoded = if let Some(cegis_size) = cegis_size {
        let solved = cegis::cegis_solve(delis_file, cegis_size, &sygus_cmds, |cmds| {
            let output = solve_cmds_with_backend(backend, cmds, verbose, min)?;
            if output.trim() == "infeasible" {
                return Ok("infeasible".to_string());
            }
            match backend {
                SygusBackend::InProcessDryadSynth => Ok(output),
                SygusBackend::External(_) => {
                    delis_file.decode_response(&output).map_err(io::Error::other)
                }
            }
        })?;
        log::info!("cegis: solved in {:?}", solve_start.elapsed());
        solved
    } else {
        let output =
            solve_cmds_with_backend(backend, &sygus_cmds, verbose, min)?;
        log::info!("sygus backend: solved in {:?}", solve_start.elapsed());
        match backend {
            SygusBackend::InProcessDryadSynth => output,
            SygusBackend::External(_) => delis_file.decode_response(&output).map_err(io::Error::other)?,
        }
    };
    Ok(delis_file.filter_synth_output(&decoded))
}

fn solver_stdout(child: Child, solver: &str) -> Result<String, io::Error> {
    let pid = child.id();
    let output = child.wait_with_output();
    unregister_pid(pid);
    let output = output?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr.trim();
        let detail = if message.is_empty() {
            format!("{solver} exited with status {}", output.status)
        } else {
            format!("{solver} failed: {message}")
        };
        return Err(io::Error::other(detail));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Command-line interface for Delis.
#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "Delis: Learning logic formulas from finite models"
)]
pub struct CLI {
    /// Path to the input file (if not using --in)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Read input from stdin instead of file
    #[arg(short, long)]
    r#in: bool,

    /// Increase log verbosity (use `-v` for INFO, `-vv` for DEBUG).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

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

    /// Enable exact-nullary relevance (bit minimization) for dryadsynth-bv; use --min=false to disable.
    #[arg(
        long = "min",
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        value_parser = clap::builder::BoolishValueParser::new()
    )]
    min: bool,

    /// Enable bisimulation-based aspect quotient to reduce BV width
    /// (use --bisim=false to disable).
    #[arg(
        short = 'B',
        long = "bisim",
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        value_parser = clap::builder::BoolishValueParser::new()
    )]
    bisim: bool,

    /// Use only Delis's internal enumerative backend (no SyGuS solver backend).
    #[arg(long = "enum", default_value_t = false)]
    r#enum: bool,

    /// Translate to SyGuS BV and print to stdout
    #[arg(short, long = "dump-sygus", default_value_t = false)]
    dump_sygus: bool,

    /// Translate to SyGuS BV, but print only machine-readable stats:
    /// `SYGUS_STATS<TAB>ms<TAB>bytes<TAB>lines`
    #[arg(long = "sygus-stats", default_value_t = false)]
    dump_sygus_stats: bool,

    #[arg(long = "dump-chc", default_value_t = false)]
    dump_chc: bool,

    /// Use counterexample-guided subset solving for the external SyGuS backend,
    /// seeding with up to N constraints. Off by default; pass `--cegis` to
    /// enable with the default seed size, or `--cegis=N` to set the seed.
    #[arg(
        long = "cegis",
        num_args = 0..=1,
        require_equals = true,
        default_value = "0",
        default_missing_value = "6",
        value_name = "N"
    )]
    cegis: usize,

    /// Use an explicit external SyGuS solver executable instead of the in-process dryadsynth-bv backend.
    #[arg(long = "solver", value_name = "PATH")]
    solver: Option<String>,
}

/// Run the Delis CLI.
fn main() -> Result<(), io::Error> {
    let args = CLI::parse();
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let prefix = match level {
                log::Level::Error => "error: ",
                log::Level::Warn => "warning: ",
                log::Level::Info => "",
                log::Level::Debug => "debug: ",
                log::Level::Trace => "trace: ",
            };
            if level >= log::Level::Debug {
                writeln!(
                    buf,
                    "{}{} ({}:{})",
                    prefix,
                    record.args(),
                    record.file().unwrap_or("?"),
                    record.line().unwrap_or(0)
                )
            } else {
                writeln!(buf, "{}{}", prefix, record.args())
            }
        })
        .filter_level(match args.verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            _ => log::LevelFilter::Debug,
        })
        .init();

    ctrlc::set_handler(move || {
        cleanup_children();
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    compile::init_options(compile::CompileOptions {
        optimize: args.optimize,
        bisim: args.bisim,
    });

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
    let results = DelisProg::parse_cmds(&input, Some(&mut session.runtime));
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
