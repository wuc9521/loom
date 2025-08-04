use clap::Parser;
use log::info;
use ls_parser::lang::runtime::Runtime;
use ls_parser::parser::LSFile;
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
    /// Use DryadSynth solver (if available)
    DryadSynth,
    /// Use no solver (just translate)
    None,
}

/// Process file with the new four-stage compilation pipeline
fn process_file(input: &str, args: &CLI) -> Result<(), io::Error> {
    // Parse file
    let ls_file = LSFile::parse(input)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Parse error: {}", e)))?;

    // Create interpreter for legacy compatibility
    let mut interpreter = Runtime::new();

    // Handle different output modes
    match args.output {
        OutputMode::Execute => {
            // Execute commands normally using interpreter
            for cmd in &ls_file.cmds {
                interpreter.run(cmd).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("Execution error: {}", e))
                })?;
            }
            info!("File executed successfully");
        }

        OutputMode::Show => {
            // Show parsed commands
            info!("Parsed LS commands:");
            for (i, cmd) in ls_file.cmds.iter().enumerate() {
                println!("  {i:>2}: {cmd}");
            }
        }

        OutputMode::Translate => {
            // Translate to standard SyGuS using new pipeline
            let mut translator = SyGuSTranslator::new(ls_file, Some(interpreter));
            match translator.translate() {
                Ok(commands) => {
                    info!("Translation completed successfully!");
                    info!("Generated {} SyGuS commands:", commands.len());
                    for cmd in commands.iter() {
                        println!("{cmd}");
                    }
                }
                Err(e) => {
                    eprintln!("Translation error: {}", e);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Translation error: {}", e),
                    ));
                }
            }
        }

        OutputMode::Save => {
            // Translate and save to file
            info!("Translating and saving to file...");
            let mut translator = SyGuSTranslator::new(ls_file, Some(interpreter));

            let output_file = args
                .output_file
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Output file not specified"))?;

            match translator.to_string() {
                Ok(sygus_string) => {
                    fs::write(output_file, sygus_string).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to write output file: {}", e),
                        )
                    })?;
                    println!("Translated SyGuS saved to: {}", output_file.display());
                }
                Err(e) => {
                    eprintln!("Translation error: {}", e);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Translation error: {}", e),
                    ));
                }
            }
        }

        OutputMode::Synthesize => {
            // Run synthesis using the specified solver
            info!("Running synthesis using {:?} solver...", args.solver);
            let mut translator = SyGuSTranslator::new(ls_file, Some(interpreter));

            match translator.translate() {
                Ok(commands) => {
                    info!("Translation completed, running synthesis...");

                    match translator.solve(&commands) {
                        Ok(result) => match result.status {
                            ls_parser::translator::SolverStatus::Sat => {
                                println!("Synthesis successful!");
                                if !result.synthesis_results.is_empty() {
                                    println!("Synthesized functions:");
                                    for (func_name, func_body) in &result.synthesis_results {
                                        println!("  {}: {}", func_name, func_body);
                                    }
                                }
                            }
                            ls_parser::translator::SolverStatus::Unsat => {
                                println!("Synthesis failed: No solution exists");
                            }
                            ls_parser::translator::SolverStatus::Unknown => {
                                println!("Synthesis result unknown: Solver could not determine satisfiability");
                            }
                            ls_parser::translator::SolverStatus::Error(msg) => {
                                eprintln!("Solver error: {}", msg);
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    format!("Solver error: {}", msg),
                                ));
                            }
                        },
                        Err(e) => {
                            eprintln!("Solver error: {}", e);
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Solver error: {}", e),
                            ));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Translation error: {}", e);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Translation error: {}", e),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args = CLI::parse();

    // Set up logging
    if args.verbose {
        env_logger::init();
    }

    // Handle file input or interactive mode
    if let Some(file_path) = &args.file {
        // Read and process file
        let input = fs::read_to_string(&file_path).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to read file: {}", e))
        })?;
        process_file(&input, &args)?;
    } else {
        repl::repl()?;
    }

    Ok(())
}
