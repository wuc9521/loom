use ls_parser::lang::interpreter::Interpreter;
use ls_parser::parser::ast::LSFile;
use ls_parser::translator::SyGuSTranslator;

fn main() {
    // Set up logging
    env_logger::init();

    // Create a simple test file
    let test_content = r#"
(set-logic RE)
(declare-datatype RE (
  (re.empty) (re.eps) 
  (re.sym (sym Char))
  (re.union (lhs RE) (rhs RE))
  (re.++    (lhs RE) (rhs RE))
  (re.*     (body RE))
  (re.inter (lhs RE) (rhs RE))
  (re.comp (body RE))
))
(synth-fun regex () RE)
(constraint (= regex re.empty))
(check-synth)
"#;

    println!("Testing translation framework...");
    println!("Input file:\n{}", test_content);

    // Parse the file
    match ls_parser::parser::ast::parse(test_content) {
        Ok(ls_file) => {
            println!("Parsed successfully!");
            
            // Create interpreter
            let mut interpreter = Interpreter::new();
            
            // Create translator
            let mut translator = SyGuSTranslator::new(ls_file, &interpreter);
            
            // Test translation
            match translator.translate() {
                Ok(translated_commands) => {
                    println!("Translation successful!");
                    println!("Generated {} commands:", translated_commands.len());
                    
                    for (i, cmd) in translated_commands.iter().enumerate() {
                        println!("  {}: {}", i, cmd);
                    }
                    
                    // Test CVC5 integration (if available)
                    println!("\nTesting CVC5 integration...");
                    match translator.solve(&translated_commands) {
                        Ok(solver_result) => {
                            println!("CVC5 result: {:?}", solver_result.status);
                            match solver_result.status {
                                ls_parser::translator::SolverStatus::Sat => {
                                    println!("Synthesis successful!");
                                    for (func_name, body) in &solver_result.synthesis_results {
                                        println!("  {}: {}", func_name, body);
                                    }
                                }
                                ls_parser::translator::SolverStatus::Unsat => {
                                    println!("Synthesis problem is unsatisfiable");
                                }
                                ls_parser::translator::SolverStatus::Unknown => {
                                    println!("Synthesis result is unknown");
                                }
                                ls_parser::translator::SolverStatus::Error(msg) => {
                                    println!("Solver error: {}", msg);
                                }
                            }
                        }
                        Err(e) => {
                            println!("CVC5 solver failed: {}", e);
                            println!("This is expected if CVC5 is not installed or not in PATH");
                        }
                    }
                }
                Err(e) => {
                    println!("Translation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {}", e);
        }
    }
} 