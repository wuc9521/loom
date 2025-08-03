use clap::Parser;
use ls_parser::lang::interpreter::Interpreter;
use ls_parser::parser::ast::parse;
use ls_parser::translator::SyGuSTranslator;
use std::fs;
use std::io;
use std::path::PathBuf;

mod repl;

#[derive(Parser)]
#[command(author, version, about = "LS Parser - Extended SyGuS Language Interpreter", long_about = None)]
pub struct CLI {
    /// Input file to process (if not provided, starts interactive mode)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Theory file to load before processing main file
    #[arg(short, long = "theory", value_name = "THEORY_FILE")]
    theory_file: Option<PathBuf>,

    /// Output mode: what to do with the processed file
    #[arg(short, long, value_enum, default_value_t = OutputMode::Execute)]
    output: OutputMode,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Solver to use for synthesis (requires CVC5 to be installed)
    #[arg(short, long, value_enum, default_value_t = Solver::CVC5)]
    solver: Solver,

    /// Output file for translated SyGuS (when using --output save)
    #[arg(long = "output-file", value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputMode {
    /// Execute the file normally (default)
    Execute,
    /// Print the parsed LS commands
    Show,
    /// Translate to standard SyGuS and print
    Translate,
    /// Translate to standard SyGuS and save to file
    Save,
    /// Run synthesis using the specified solver
    Synthesize,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Solver {
    /// Use CVC5 solver (default)
    CVC5,
    /// Use Z3 solver (if available)
    Z3,
    /// Use no solver (just translate)
    None,
}


/// Process file with the new unified approach
fn process_file(input: &str, args: &CLI) -> Result<(), io::Error> {
    // Parse file
    let ls_file = parse(input)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Parse error: {}", e)))?;

    // Create interpreter
    let mut interpreter = Interpreter::new();
    
    // Load theory file if provided
    if let Some(ref theory_file) = args.theory_file {
        println!("Loading theory file: {}", theory_file.display());
        interpreter.load_theory_file(theory_file.to_str().unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Theory loading error: {}", e)))?;
    }
    
    // Handle different output modes
    match args.output {
        OutputMode::Execute => {
            // Execute commands normally
            for cmd in &ls_file.cmds {
                interpreter.proc(cmd)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Execution error: {}", e)))?;
            }
            println!("File executed successfully");
        }
        
        OutputMode::Show => {
            // Show parsed commands
            println!("Parsed LS commands:");
            for (i, cmd) in ls_file.cmds.iter().enumerate() {
                println!("  {}: {}", i + 1, cmd);
            }
        }
        
        OutputMode::Translate => {
            // Translate to standard SyGuS and print using dynamic translation
            let mut translator = SyGuSTranslator::new(ls_file, Some(interpreter));
            match translator.preprocess_with_interpreter() {
                Ok(_) => {
                    match translator.translate_dynamically() {
                        Ok(translated_commands) => {
                            for cmd in translated_commands {
                                println!("{}", cmd);
                            }
                        }
                        Err(e) => {
                            return Err(io::Error::new(io::ErrorKind::Other, format!("Dynamic translation error: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, format!("Preprocessing error: {}", e)));
                }
            }
        }
        
        OutputMode::Save => {
            // Translate and save to file using dynamic translation
            let mut translator = SyGuSTranslator::new(ls_file, Some(interpreter));
            match translator.preprocess_with_interpreter() {
                Ok(_) => {
                    match translator.translate_dynamically() {
                        Ok(translated_commands) => {
                            let output_path = args.output_file.as_ref()
                                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Output file required for save mode"))?;
                            
                            let sygus_program = translator.commands_to_string(&translated_commands)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("String conversion error: {}", e)))?;
                            
                            fs::write(output_path, sygus_program)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("File write error: {}", e)))?;
                            
                            println!("Translated SyGuS saved to: {}", output_path.display());
                        }
                        Err(e) => {
                            return Err(io::Error::new(io::ErrorKind::Other, format!("Dynamic translation error: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, format!("Preprocessing error: {}", e)));
                }
            }
        }
        
        OutputMode::Synthesize => {
            // Execute and run synthesis
            for cmd in &ls_file.cmds {
                interpreter.proc(cmd)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Execution error: {}", e)))?;
            }
            
            match args.solver {
                Solver::CVC5 => {
                    println!("Running synthesis with CVC5...");
                    // The synthesis will be triggered automatically by check-synth commands
                    // The interpreter's solve_with_bv_solver will handle the translation and solving
                }
                Solver::Z3 => {
                    println!("Z3 solver not yet implemented");
                    return Err(io::Error::new(io::ErrorKind::Other, "Z3 solver not yet implemented"));
                }
                Solver::None => {
                    println!("Synthesis skipped (no solver specified)");
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args = CLI::parse();

    // Setup logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .format_module_path(false)
            .format_target(false)
            .init();
    }

    // Handle different modes
    if let Some(file_path) = &args.file {
        // File mode
        let input = fs::read_to_string(file_path)?;
        process_file(&input, &args)
    } else {
        // Interactive mode
        println!("LS Parser - Extended SyGuS Language Interpreter");
        println!("Type :help for available commands");
        repl::repl()
    }
}
