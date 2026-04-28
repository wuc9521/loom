use clap::Parser as ClapParser;
use delis::{
    backends::smt2,
    compile,
    lang::runtime::Session,
    parser::{load_module_file, parse_commands, DelisCmd, DelisProg},
    process::{cleanup_children, register_pid, unregister_pid},
};
use std::{
    collections::HashSet,
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::Instant,
};
use sygus_parser::{Sort, SyGuSCmd, SyGuSFile};

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
            sygus_parser::writer::write_cmds(&mut stdout, &cmds)?;
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
    let sygus_optimize = args.optimize;
    let sygus_enum_trigger = args.enum_trigger;
    let sygus_cegis = args.cegis;
    const SYGUS_STACK_BYTES: usize = 128 * 1024 * 1024;
    thread::Builder::new()
        .name("sygus-backend".to_string())
        .stack_size(SYGUS_STACK_BYTES)
        .spawn(move || {
            let result =
                solve_with_sygus(&sygus_file, sygus_optimize, sygus_enum_trigger, sygus_cegis);
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
                log::error!("backend failed: {}", e);
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

fn sygus_config(bv_sz: usize, enum_trigger: usize) -> Solver {
    let mut args = vec!["--lang=sygus2".to_string()];
    if bv_sz >= enum_trigger {
        args.push("--sygus-enum=fast".to_string());
    }
    ("cvc5".to_string(), args)
}

fn spawn_solver(solver: &Solver, cmds: &[SyGuSCmd]) -> Result<Child, io::Error> {
    let mut input = Vec::new();
    sygus_parser::writer::write_cmds(&mut input, cmds)?;
    writeln!(&mut input)?;

    let mut child = Command::new(&solver.0)
        .args(&solver.1)
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

fn solve_with_sygus(
    delis_file: &DelisProg,
    optimize: bool,
    enum_trigger: usize,
    cegis_size: Option<usize>,
) -> Result<String, io::Error> {
    let solve_start = Instant::now();
    let decoded = if let Some(cegis_size) = cegis_size {
        let step_start = Instant::now();
        let full_sygus_cmds = compile::to_sygus(delis_file, optimize).map_err(io::Error::other)?;
        log::info!(
            "sygus-backend: full to_sygus for verification completed in {:?}",
            step_start.elapsed()
        );
        let step_start = Instant::now();
        let solved = cegis_solve(
            delis_file,
            &full_sygus_cmds,
            optimize,
            enum_trigger,
            cegis_size,
        )?;
        log::info!(
            "sygus-backend: cegis completed in {:?}",
            step_start.elapsed()
        );
        solved
    } else {
        let step_start = Instant::now();
        let sygus_cmds = compile::to_sygus(delis_file, optimize).map_err(io::Error::other)?;
        log::info!(
            "sygus-backend: to_sygus completed in {:?}",
            step_start.elapsed()
        );

        let step_start = Instant::now();
        let bv_sz = sygus_bit_width(&sygus_cmds).map_err(io::Error::other)?;
        log::info!(
            "sygus-backend: derived bv width completed in {:?}",
            step_start.elapsed()
        );

        let step_start = Instant::now();
        let sygus_solver = sygus_config(bv_sz, enum_trigger);
        log::info!(
            "sygus-backend: sygus_config completed in {:?}",
            step_start.elapsed()
        );

        let step_start = Instant::now();
        let child = spawn_solver(&sygus_solver, &sygus_cmds)?;
        log::info!(
            "sygus-backend: spawn_solver completed in {:?}",
            step_start.elapsed()
        );

        let step_start = Instant::now();
        let output = solver_stdout(child, &sygus_solver.0)?;
        log::info!(
            "sygus-backend: solver_stdout completed in {:?}",
            step_start.elapsed()
        );
        decode_cvc5_response(delis_file, &output)?
    };
    log::info!(
        "sygus-backend: solve completed in {:?}",
        solve_start.elapsed()
    );
    Ok(decoded)
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

fn cegis_solve(
    delis_file: &DelisProg,
    full_cmds: &[SyGuSCmd],
    optimize: bool,
    enum_trigger: usize,
    seed_size: usize,
) -> Result<String, io::Error> {
    let seed_size = seed_size.max(1);
    let constraints = problem_constraints(delis_file);
    let constraint_count = constraints.len();
    if constraint_count <= seed_size {
        let bv_sz = compile::bv_sz(delis_file).map_err(io::Error::other)?;
        let cmds = compile::to_sygus(delis_file, optimize).map_err(io::Error::other)?;
        let solver = sygus_config(bv_sz, enum_trigger);
        let child = spawn_solver(&solver, &cmds)?;
        let output = solver_stdout(child, &solver.0)?;
        return decode_cvc5_response(delis_file, &output);
    }

    let polarities = constraints
        .iter()
        .map(|term| constraint_polarity(term))
        .collect::<Vec<_>>();
    let mut active = initial_cegis_constraints(&polarities, seed_size);
    let mut round = 0usize;
    loop {
        round += 1;
        log::info!(
            "cegis round {}: solving with {}/{} constraints",
            round,
            active.len(),
            constraint_count
        );
        let reduced_problem = subset_problem(delis_file, &active);
        let round_start = Instant::now();
        let bv_sz = compile::bv_sz(&reduced_problem).map_err(io::Error::other)?;
        let subset_cmds =
            compile::to_sygus(&reduced_problem, optimize).map_err(io::Error::other)?;
        let solver = sygus_config(bv_sz, enum_trigger);
        log::info!(
            "cegis round {}: reduced compilation completed in {:?}",
            round,
            round_start.elapsed()
        );
        let child = spawn_solver(&solver, &subset_cmds)?;
        let output = solver_stdout(child, &solver.0)?;
        let trimmed = output.trim();
        if trimmed == "infeasible" {
            log::info!("cegis round {}: subset is infeasible", round);
            return Ok("infeasible".to_string());
        }
        let decoded = decode_cvc5_response(delis_file, &output)?;
        let failing =
            compile::failing_constraint_indices(full_cmds, &decoded).map_err(io::Error::other)?;
        if failing.is_empty() {
            log::info!("cegis round {}: candidate satisfies all constraints", round);
            return Ok(decoded);
        }
        let added = add_counterexamples(&mut active, &failing, &polarities);
        log::info!(
            "cegis round {}: candidate failed {} constraints, added {}",
            round,
            failing.len(),
            added
        );
        if added == 0 {
            return Err(io::Error::other(
                "CEGIS made no progress while verification still fails",
            ));
        }
    }
}

fn sygus_bit_width(cmds: &[SyGuSCmd]) -> Result<usize, String> {
    let sort = cmds
        .iter()
        .find_map(|cmd| match cmd {
            SyGuSCmd::SynthFun(_, _, sort, _) => Some(sort),
            _ => None,
        })
        .ok_or_else(|| "missing synth-fun in generated SyGuS".to_string())?;
    let text = sort.to_string();
    let prefix = "(_ BitVec ";
    let suffix = ")";
    if let Some(width) = text
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        return width
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("failed to parse synth-fun bit-width {width}: {e}"));
    }
    Err(format!(
        "expected synth-fun to return a BitVec, found {}",
        sort
    ))
}

fn subset_problem(problem: &DelisProg, active: &HashSet<usize>) -> DelisProg {
    let mut cmds = Vec::with_capacity(problem.cmds.len());
    let mut constraint_idx = 0usize;
    for cmd in &problem.cmds {
        match cmd {
            DelisCmd::As(SyGuSCmd::Constraint(_)) => {
                if active.contains(&constraint_idx) {
                    cmds.push(cmd.clone());
                }
                constraint_idx += 1;
            }
            _ => cmds.push(cmd.clone()),
        }
    }
    DelisProg { cmds }
}

fn problem_constraints(problem: &DelisProg) -> Vec<sygus_parser::SyGuSTerm> {
    problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::As(SyGuSCmd::Constraint(term)) => Some(term.clone()),
            _ => None,
        })
        .collect()
}

fn constraint_polarity(term: &sygus_parser::SyGuSTerm) -> Option<bool> {
    constraint_expected_truth(term, false)
}

fn constraint_expected_truth(term: &sygus_parser::SyGuSTerm, negated: bool) -> Option<bool> {
    if let sygus_parser::SyGuSTerm::Application(id, args) = term {
        if id.get_name() == "not" && args.len() == 1 {
            return constraint_expected_truth(&args[0], !negated);
        }
        if id.get_name() == "=" && args.len() == 2 {
            if let Some(value) = binary_truth_value(&args[0]) {
                return constraint_expected_truth(&args[1], negated ^ !value);
            }
            if let Some(value) = binary_truth_value(&args[1]) {
                return constraint_expected_truth(&args[0], negated ^ !value);
            }
        }
    }
    Some(!negated)
}

fn binary_truth_value(term: &sygus_parser::SyGuSTerm) -> Option<bool> {
    match term {
        sygus_parser::SyGuSTerm::Literal(sygus_parser::Literal::Bool(value)) => Some(*value),
        sygus_parser::SyGuSTerm::Literal(sygus_parser::Literal::BinConst(bits)) => {
            match bits.as_str() {
                "#b1" => Some(true),
                "#b0" => Some(false),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sygus_parser::ast::{Identifier, Literal, SyGuSTerm};

    fn app(name: &str, args: Vec<SyGuSTerm>) -> SyGuSTerm {
        SyGuSTerm::Application(Identifier::from_str(name).unwrap(), args)
    }

    fn bool_lit(value: bool) -> SyGuSTerm {
        SyGuSTerm::Literal(Literal::Bool(value))
    }

    fn bin_lit(bits: &str) -> SyGuSTerm {
        SyGuSTerm::Literal(Literal::BinConst(bits.into()))
    }

    fn atom(name: &str) -> SyGuSTerm {
        app(name, vec![])
    }

    #[test]
    fn constraint_polarity_handles_nested_negation_and_truth_equalities() {
        assert_eq!(constraint_polarity(&atom("p")), Some(true));
        assert_eq!(
            constraint_polarity(&app("not", vec![atom("p")])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("not", vec![app("not", vec![atom("p")])])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bool_lit(true)])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bool_lit(false)])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bin_lit("#b1")])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bin_lit("#b0")])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("not", vec![app("=", vec![atom("p"), bin_lit("#b0")])])),
            Some(true)
        );
    }
}

fn initial_cegis_constraints(polarities: &[Option<bool>], seed_size: usize) -> HashSet<usize> {
    let seed_size = seed_size.max(1);
    let mut active = HashSet::new();
    let target_pos = seed_size.div_ceil(2);
    let target_neg = seed_size / 2;
    for (target, class_target) in [(Some(true), target_pos), (Some(false), target_neg)] {
        let mut class_count = 0usize;
        for (idx, polarity) in polarities.iter().enumerate() {
            if *polarity == target && !active.contains(&idx) {
                active.insert(idx);
                class_count += 1;
                if active.len() >= seed_size {
                    return active;
                }
                if class_count >= class_target {
                    break;
                }
            }
        }
    }
    for idx in 0..polarities.len() {
        if active.len() >= seed_size {
            break;
        }
        active.insert(idx);
    }
    active
}

fn add_counterexamples(
    active: &mut HashSet<usize>,
    failing: &[usize],
    polarities: &[Option<bool>],
) -> usize {
    let before = active.len();
    for target in [Some(true), Some(false)] {
        if let Some(idx) = failing
            .iter()
            .copied()
            .find(|idx| !active.contains(idx) && polarities[*idx] == target)
        {
            active.insert(idx);
        }
    }
    if active.len() == before {
        if let Some(idx) = failing.iter().copied().find(|idx| !active.contains(idx)) {
            active.insert(idx);
        }
    }
    active.len() - before
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
        .ok_or_else(|| io::Error::other("missing synth-fun command"))
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
    about = "Delis: Learning logic formulas from finite models"
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
    #[arg(short = 'E', long = "enum-trigger", default_value_t = 512)]
    enum_trigger: usize,

    /// Use counterexample-guided subset solving for the external SyGuS backend,
    /// seeding with up to N constraints.
    #[arg(
        long = "cegis",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "6",
        value_name = "N"
    )]
    cegis: Option<usize>,
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
