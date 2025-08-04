use log::error;
use ls_parser::lang::runtime::Runtime;
use ls_parser::parser::LSFile;
use ls_parser::translator::SyGuSTranslator;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::io;

/// REPL command handler
fn handle_repl_command(cmd: &str, interpreter: &mut Runtime) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    match parts[0] {
        ":help" | ":h" => {
            println!("Available commands:");
            println!("  :help, :h           - Show this help");
            println!("  :quit, :q           - Exit the REPL");
            println!("  :clear, :c          - Clear the current buffer");
            println!("  :show, :s           - Show current buffer");
            println!("  :execute, :e        - Execute current buffer");
            println!("  :translate, :t      - Translate current buffer to SyGuS");
            println!("  :synthesize, :syn   - Execute and run synthesis");
            println!("  :load <file>        - Load and execute a file");
            println!("  :env                - Show current environment");
            println!("  :history            - Show command history");
            true
        }
        ":quit" | ":q" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        ":clear" | ":c" => {
            // This will be handled by the caller
            true
        }
        ":show" | ":s" => {
            // This will be handled by the caller
            true
        }
        ":execute" | ":e" => {
            // This will be handled by the caller
            true
        }
        ":translate" | ":t" => {
            // This will be handled by the caller
            true
        }
        ":synthesize" | ":syn" => {
            // This will be handled by the caller
            true
        }
        ":load" => {
            if parts.len() != 2 {
                println!("Usage: :load <file>");
                return true;
            }
            match std::fs::read_to_string(parts[1]) {
                Ok(content) => {
                    println!("Loading file: {}", parts[1]);
                    match LSFile::parse(&content) {
                        Ok(ls_file) => {
                            for cmd in &ls_file.cmds {
                                if let Err(e) = interpreter.run(cmd) {
                                    println!("Error executing command: {}", e);
                                }
                            }
                            println!("File loaded successfully");
                        }
                        Err(e) => {
                            println!("Parse error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading file: {}", e);
                }
            }
            true
        }
        ":env" => {
            println!("Current environment:");
            println!("  Global scope bindings: {}", interpreter.global_scope().bindings.len());
            println!("  Type environment scopes: {}", interpreter.typing.stack.len());
            true
        }
        ":history" => {
            println!("Command history not yet implemented");
            true
        }
        _ => {
            println!("Unknown command: {}. Type :help for available commands.", parts[0]);
            true
        }
    }
}

/// Check if a string has balanced parentheses
fn paren_balance(s: &str) -> i32 {
    let mut count = 0;
    for c in s.chars() {
        match c {
            '(' => count += 1,
            ')' => count -= 1,
            _ => {}
        }
    }
    count
}

/// Execute buffer content
fn execute_buffer(buffer: &str, interpreter: &mut Runtime) -> Result<(), String> {
    if buffer.trim().is_empty() {
        return Ok(());
    }

    match LSFile::parse(buffer) {
        Ok(ls_file) => {
            for cmd in &ls_file.cmds {
                interpreter.run(cmd)?;
            }
            Ok(())
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Translate buffer content to SyGuS
fn translate_buffer(buffer: &str, _interpreter: &Runtime) -> Result<(), String> {
    if buffer.trim().is_empty() {
        return Ok(());
    }

    let ls_file = LSFile::parse(buffer).map_err(|e| e.to_string())?;
    let mut translator = SyGuSTranslator::new(ls_file, None);
    
    match translator.translate() {
        Ok(translated_commands) => {
            for cmd in translated_commands {
                println!("{}", cmd);
            }
            Ok(())
        }
        Err(e) => Err(format!("Translation error: {}", e)),
    }
}

pub fn repl() -> io::Result<()> {
    let mut buffer = String::new();
    let mut interpreter = Runtime::new();
    let mut rl = Editor::<(), DefaultHistory>::new()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("LS Parser REPL - Type :help for available commands");
    println!("Enter LS commands (use Ctrl+C to exit):");

    loop {
        let prompt = if buffer.is_empty() {
            "ls> "
        } else {
            "  > "
        };

        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim_end();
                let _ = rl.add_history_entry(line);

                // Handle REPL commands
                if line.starts_with(':') {
                    if handle_repl_command(line, &mut interpreter) {
                        match line.split_whitespace().next().unwrap() {
                            ":clear" | ":c" => {
                                buffer.clear();
                                println!("Buffer cleared");
                            }
                            ":show" | ":s" => {
                                if buffer.is_empty() {
                                    println!("Buffer is empty");
                                } else {
                                    println!("Current buffer:");
                                    println!("{}", buffer);
                                }
                            }
                            ":execute" | ":e" => {
                                match execute_buffer(&buffer, &mut interpreter) {
                                    Ok(_) => {
                                        println!("Buffer executed successfully");
                                        buffer.clear();
                                    }
                                    Err(e) => {
                                        println!("Execution error: {}", e);
                                    }
                                }
                            }
                            ":translate" | ":t" => {
                                match translate_buffer(&buffer, &interpreter) {
                                    Ok(_) => {
                                        println!("Translation completed");
                                        buffer.clear();
                                    }
                                    Err(e) => {
                                        println!("Translation error: {}", e);
                                    }
                                }
                            }
                            ":synthesize" | ":syn" => {
                                match execute_buffer(&buffer, &mut interpreter) {
                                    Ok(_) => {
                                        println!("Buffer executed, synthesis will be triggered by check-synth commands");
                                        buffer.clear();
                                    }
                                    Err(e) => {
                                        println!("Execution error: {}", e);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }

                // Add line to buffer
                buffer.push_str(line);
                buffer.push('\n');

                // Check if we have a complete expression
                if paren_balance(&buffer) == 0 && !buffer.trim().is_empty() {
                    // Try to execute the buffer
                    match execute_buffer(&buffer, &mut interpreter) {
                        Ok(_) => {
                            println!("✓ Executed successfully");
                            buffer.clear();
                        }
                        Err(e) => {
                            println!("✗ Error: {}", e);
                            // Don't clear buffer on error, let user fix it
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                if buffer.is_empty() {
                    println!("\nUse Ctrl+C again to exit");
                } else {
                    println!("\nBuffer cleared (Ctrl+C)");
                    buffer.clear();
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                error!("Readline error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
