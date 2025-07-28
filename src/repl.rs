use ls_parser::lang::runtime::Session;
use ls_parser::theory::ModelCtx;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::io;

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

fn handle_repl_command(cmd: &str, session: &mut Session) -> bool {
    if cmd == ":help" {
        println!("Available commands:\n  :help          show help\n  :load <file>   load file\n  :ctx           print current context and symbol table\n  :save <file>   save last model as json\n  :quit/:exit    exit");
        true
    } else if cmd.starts_with(":load ") {
        let filename = cmd[6..].trim();
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                for res in
                    ls_parser::parser::ast::parse_commands(&content, None)
                {
                    match res {
                        Ok(cmd) => {
                            let _ = interpreter.process_command(&cmd);
                        }
                        Err(e) => println!("{}", e),
                    }
                }
            }
            Err(e) => println!("cannot read file {}: {}", filename, e),
        }
        true
    } else if cmd == ":ctx" {
        println!("current ctx: {:?}", interpreter.ctx);
        println!("symbol table:");
        for (name, value) in interpreter.iter() {
            println!("  {}: {:?}", name, value);
        }
        true
    } else if cmd.starts_with(":save ") {
        let filename = cmd[6..].trim();
        match &interpreter.last_model {
            Some((model, ModelCtx::FOL(fol_ctx))) => {
                let json = ls_parser::theory::fol::model_to_json_with_symbol_map(model, fol_ctx);
                std::fs::write(filename, serde_json::to_string_pretty(&json).unwrap()).unwrap();
                println!("FOL model json saved to {}", filename);
            }
            Some((model, ModelCtx::PML(pml_ctx))) => {
                let json = ls_parser::theory::pml::model_to_json_with_symbol_map(model, pml_ctx);
                std::fs::write(filename, serde_json::to_string_pretty(&json).unwrap()).unwrap();
                println!("PML model json saved to {}", filename);
            }
            Some((model, ModelCtx::RE(re_ctx))) => {
                let json = ls_parser::theory::re::model_to_json_with_symbol_map(model, re_ctx);
                std::fs::write(filename, serde_json::to_string_pretty(&json).unwrap()).unwrap();
                println!("RE model json saved to {}", filename);
            }
            None => println!("no model to save yet"),
        }
        true
    } else if cmd == ":quit" || cmd == ":exit" {
        std::process::exit(0);
    } else {
        println!("unknown command: {}", cmd);
        true
    }
}

pub fn repl() -> io::Result<()> {
    let mut buffer = String::new();
    let mut interpreter = Interpreter::new();
    let mut rl =
        Editor::<(), DefaultHistory>::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    loop {
        let readline = rl.readline("@wuc9521> ");
        match readline {
            Ok(line) => {
                let line = line.trim_end();
                let _ = rl.add_history_entry(line); // handle error
                buffer.push_str(line);
                buffer.push('\n');
                if paren_balance(buffer.as_str()) != 0 {
                    continue;
                }
                if buffer.trim_start().starts_with(':') {
                    let cmd = buffer.trim();
                    if handle_repl_command(cmd, &mut session) {
                        buffer.clear();
                        continue;
                    }
                }
                let mut has_error = false;
                let results =
                    ls_parser::parser::ast::parse_commands(&buffer, None);
                for res in &results {
                    match res {
                        Ok(cmd) => {
                            let _ = interpreter.process_command(&cmd);
                        }
                        Err(e) => {
                            println!("{}", e);
                            has_error = true;
                            break;
                        }
                    }
                }
                if has_error {
                    buffer.clear();
                    continue;
                }
                buffer.clear();
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                println!("readline error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
