use log::error;
use ls_parser::lang::interpreter::Interpreter;
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

fn handle_repl_command(_cmd: &str, _interpreter: &mut Interpreter) -> bool {
    unimplemented!()
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
                    if handle_repl_command(cmd, &mut interpreter) {
                        buffer.clear();
                        continue;
                    }
                }
                let mut has_error = false;
                match ls_parser::parser::ast::parse(&buffer) {
                    Ok(ls_file) => {
                        for cmd in &ls_file.cmds {
                            let _ = interpreter.proc(cmd).unwrap();
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        has_error = true;
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
                error!("readline error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
