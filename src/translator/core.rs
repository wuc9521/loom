use crate::lang::interpreter::Interpreter;
use crate::parser::ast::{LSCmd, LSFile, LSTerm};
use crate::translator::desugar::Desugarer;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;
use sygus_parser::SyGuSCmd;

/// Symbol mapping for translation context
#[derive(Debug, Clone)]
pub struct SymbolMapping {
    /// Forward mapping: original symbol -> translated symbol
    pub forward: HashMap<String, String>,
    /// Reverse mapping: translated symbol -> original symbol
    pub reverse: HashMap<String, String>,
    /// Object method mappings: (object_name, method_name) -> flattened_function_name
    pub object_methods: HashMap<(String, String), String>,
    /// Class method mappings: (class_name, method_name) -> template_function_name
    pub class_methods: HashMap<(String, String), String>,
    /// Used symbols to avoid conflicts
    pub used_symbols: HashSet<String>,
    /// Counter for generating unique names
    pub counter: u32,
}

impl SymbolMapping {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            object_methods: HashMap::new(),
            class_methods: HashMap::new(),
            used_symbols: HashSet::new(),
            counter: 0,
        }
    }

    /// Map a class method to a template function name
    pub fn map_class_method(&mut self, class_name: &str, method_name: &str) -> String {
        let key = (class_name.to_string(), method_name.to_string());
        if let Some(mapped) = self.class_methods.get(&key) {
            return mapped.clone();
        }

        let template_name = format!("{}@{}", class_name, method_name);
        self.class_methods.insert(key, template_name.clone());
        self.used_symbols.insert(template_name.clone());

        debug!(
            "Mapped class method: {}.{} -> {}",
            class_name, method_name, template_name
        );
        template_name
    }
}

/// Translation context containing type and runtime information
pub struct TranslationContext {
    pub symbol_mapping: SymbolMapping,
    pub type_environment: Option<crate::lang::types::TypeEnv>,
    pub runtime_context: Option<HashMap<String, crate::lang::runtime::Value>>,
    /// Object field mappings: (object_name, field_name) -> flattened_function_name
    pub object_fields: HashMap<(String, String), String>,
}

impl TranslationContext {
    pub fn new() -> Self {
        Self {
            symbol_mapping: SymbolMapping::new(),
            type_environment: None,
            runtime_context: None,
            object_fields: HashMap::new(),
        }
    }
}

/// Solver status
#[derive(Debug, Clone)]
pub enum SolverStatus {
    Sat,
    Unsat,
    Unknown,
    Error(String),
}

/// Solver result
#[derive(Debug)]
pub struct SolverResult {
    pub status: SolverStatus,
    pub synthesis_results: HashMap<String, String>,
    pub error_message: Option<String>,
}

/// CVC5 solver interface
pub struct CVC5Solver {
    pub executable_path: String,
    pub timeout: Option<Duration>,
}

impl CVC5Solver {
    pub fn new() -> Self {
        Self {
            executable_path: "cvc5".to_string(),
            timeout: Some(Duration::from_secs(30)),
        }
    }

    pub fn solve(&self, sygus_program: &str) -> Result<SolverResult, String> {
        debug!("Calling CVC5 solver...");
        let mut cmd = Command::new(&self.executable_path);
        cmd.arg("--lang=sygus2");
        if let Some(timeout) = self.timeout {
            cmd.arg(format!("--tlimit={}", timeout.as_millis()));
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn CVC5: {}", e))?;
        // Write the SyGuS program to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(sygus_program.as_bytes())
                .map_err(|e| format!("Failed to write to CVC5 stdin: {}", e))?;
        }

        // Wait for completion
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to get CVC5 output: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        debug!("CVC5 stdout: {}", stdout);
        debug!("CVC5 stderr: {}", stderr);

        // Parse the result
        self.decode_output(&stdout, &stderr)
    }

    fn decode_output(&self, stdout: &str, stderr: &str) -> Result<SolverResult, String> {
        let mut synthesis_results = HashMap::new();
        let mut status = SolverStatus::Unknown;
        let mut error_message = None;

        // Check for error in stderr
        if !stderr.is_empty() && !stderr.contains("WARNING") {
            error_message = Some(stderr.to_string());
            status = SolverStatus::Error(stderr.to_string());
        } else {
            // Parse synthesis results from stdout
            for line in stdout.lines() {
                if line.starts_with("(define-fun") {
                    // Parse synthesis result
                    if let Some((func_name, body)) = self.parse_define_fun(line) {
                        synthesis_results.insert(func_name, body);
                    }
                } else if line.contains("sat") {
                    status = SolverStatus::Sat;
                } else if line.contains("unsat") {
                    status = SolverStatus::Unsat;
                } else if line.contains("unknown") {
                    status = SolverStatus::Unknown;
                }
            }
        }

        Ok(SolverResult {
            status,
            synthesis_results,
            error_message,
        })
    }

    fn parse_define_fun(&self, line: &str) -> Option<(String, String)> {
        // Simple parsing of (define-fun func_name ... body)
        // This is a basic implementation - can be enhanced later
        if let Some(start) = line.find("(define-fun ") {
            let content = &line[start..];
            if let Some(end) = content.rfind(')') {
                let func_def = &content[..=end];
                // Extract function name (simplified)
                if let Some(name_start) = func_def.find(' ') {
                    if let Some(name_end) = func_def[name_start..].find(' ') {
                        let func_name = func_def[name_start + 1..name_start + name_end].to_string();
                        return Some((func_name, func_def.to_string()));
                    }
                }
            }
        }
        None
    }
}

/// Enhanced SyGuS translator with dynamic translation capabilities
pub struct SyGuSTranslator {
    pub original_file: LSFile,
    pub context: TranslationContext,
    pub synthesis_results: HashMap<String, LSTerm>,
    pub interpreter: Option<Interpreter>, // For dynamic translation
}

impl SyGuSTranslator {
    pub fn new(original_file: LSFile, interpreter: Option<Interpreter>) -> Self {
        Self {
            original_file,
            context: TranslationContext {
                symbol_mapping: SymbolMapping::new(),
                type_environment: None,
                runtime_context: None,
                object_fields: HashMap::new(),
            },
            synthesis_results: HashMap::new(),
            interpreter,
        }
    }

    /// Phase 1: Preprocess and type check using interpreter
    pub fn preprocess_with_interpreter(&mut self) -> Result<(), String> {
        debug!("Phase 1: Preprocessing with interpreter...");

        if let Some(interpreter) = &mut self.interpreter {
            // Process all commands to build complete type environment
            for cmd in &self.original_file.cmds {
                interpreter.proc(cmd)?;
            }

            // Extract type environment and runtime context
            let type_env = interpreter.typing.global_scope().clone();
            self.context.type_environment = Some(type_env);

            // Extract runtime context (simplified - in practice you might want more sophisticated extraction)
            let mut runtime_context = HashMap::new();
            for scope in &interpreter.stack {
                for (name, value) in &scope.bindings {
                    runtime_context.insert(name.clone(), value.clone());
                }
            }
            self.context.runtime_context = Some(runtime_context);

            debug!("Preprocessing completed with type environment and runtime context");
        } else {
            return Err("No interpreter provided for dynamic translation".to_string());
        }

        Ok(())
    }

    /// Phase 2: Dynamic translation based on complete context
    pub fn translate_dynamically(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        if self.context.type_environment.is_none() {
            return Err(
                "Type environment not available. Run preprocess_with_interpreter first."
                    .to_string(),
            );
        }

        info!("Starting new multi-pass translation pipeline...");

        // Step 1: Desugaring pass - transform high-level constructs into simple SyGuS
        let type_env = self.context.type_environment.as_ref().unwrap();
        let runtime_context = self.context.runtime_context.as_ref();

        let mut desugarer = Desugarer::new(
            self.original_file.clone(),
            Some(type_env.clone()),
            runtime_context.cloned(),
        );

        // Call the unified desugaring method
        let desugared_commands = desugarer.desugar_ast()?;
        info!(
            "Desugaring completed. Generated {} simplified commands.",
            desugared_commands.len()
        );

        // Step 2: Bit-vector translation pass - transform simple SyGuS into bit-vector logic
        let mut final_commands = Vec::new();
        final_commands.push(SyGuSCmd::SetLogic("BV".to_string()));

        for cmd in desugared_commands {
            let translated_cmd = self.translate_simple_sygus_command(&cmd)?;
            final_commands.push(translated_cmd);
        }

        info!(
            "Translation completed. Generated {} final commands.",
            final_commands.len()
        );
        Ok(final_commands)
    }

    /// Translate a simple SyGuS command (after desugaring)
    fn translate_simple_sygus_command(&self, cmd: &SyGuSCmd) -> Result<SyGuSCmd, String> {
        // For now, just pass through the command as-is
        // In the future, this could handle bit-vector specific translations
        Ok(cmd.clone())
    }

    /// Preprocess: collect all type information and symbols (legacy method)
    pub fn preprocess(&mut self) -> Result<(), String> {
        debug!("Preprocessing translation context...");

        // For now, just initialize empty symbol mapping
        // We'll implement proper symbol collection later

        debug!(
            "Preprocessing completed with {} symbols",
            self.context.symbol_mapping.forward.len()
        );
        Ok(())
    }

    /// Translate: convert LS to standard SyGuS (legacy method)
    pub fn translate(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        debug!("Starting translation...");

        self.preprocess()?;

        let mut translated_commands = Vec::new();

        // Process commands in order, keeping standard SyGuS commands unchanged
        for cmd in &self.original_file.cmds {
            match cmd {
                LSCmd::As(sygus_cmd) => {
                    // Keep standard SyGuS commands as-is
                    translated_commands.push(sygus_cmd.clone());
                }
                LSCmd::DefineSortEnum { name, elems } => {
                    // Convert enum sort definition to standard SyGuS
                    let enum_cmd = self.translate_enum_sort(name, elems)?;
                    translated_commands.push(enum_cmd);
                }
                LSCmd::ClassDef(_) => {
                    // Skip class definitions for now
                    debug!("Skipping class definition");
                }
                LSCmd::ObjectDef(_) => {
                    // Skip object definitions for now
                    debug!("Skipping object definition");
                }
                LSCmd::DefineTheory { name, cmds: _ } => {
                    // Skip theory definitions for now
                    debug!("Skipping theory definition: {}", name);
                }
                LSCmd::Echo(_) => {
                    // Skip echo commands
                    debug!("Skipping echo command");
                }
                LSCmd::DeclareFunc(_) => {
                    // Skip function declarations for now
                    debug!("Skipping function declaration");
                }
                LSCmd::DefineFunc(_) => {
                    // Skip function definitions for now
                    debug!("Skipping function definition");
                }
                LSCmd::DefineFuncRec(_) => {
                    // Skip recursive function definitions for now
                    debug!("Skipping recursive function definition");
                }
                LSCmd::ModelDef(_) => {
                    // Skip model definitions for now
                    debug!("Skipping model definition");
                }
                LSCmd::ThSpec(_) => {
                    // Skip theory-specific commands for now
                    debug!("Skipping theory-specific command");
                }
                LSCmd::SynthFunExt(_) => {
                    // Skip synthesis function extensions for now
                    debug!("Skipping synthesis function extension");
                }
            }
        }

        debug!(
            "Translation completed with {} commands",
            translated_commands.len()
        );
        Ok(translated_commands)
    }

    /// Translate enum sort definition to standard SyGuS (legacy method)
    fn translate_enum_sort(&self, name: &str, elems: &[String]) -> Result<SyGuSCmd, String> {
        debug!("Translating enum sort: {} with elements: {:?}", name, elems);

        // For now, skip enum sort translation to avoid complex type issues
        // We'll implement this properly later
        debug!("Skipping enum sort translation for now");

        // Return a placeholder command
        Ok(SyGuSCmd::DeclareSort(
            name.to_string(),
            0, // arity
        ))
    }

    /// Solve: call CVC5
    pub fn solve(&self, sygus_commands: &[SyGuSCmd]) -> Result<SolverResult, String> {
        debug!("Preparing to call CVC5 solver...");

        // Convert commands to string
        let sygus_program = self.commands_to_string(sygus_commands)?;
        debug!("SyGuS program:\n{}", sygus_program);

        // Call CVC5
        let solver = CVC5Solver::new();
        solver.solve(&sygus_program)
    }

    /// Convert SyGuS commands to string
    pub fn commands_to_string(&self, commands: &[SyGuSCmd]) -> Result<String, String> {
        let mut program = String::new();

        for cmd in commands {
            program.push_str(&cmd.to_string());
            program.push('\n');
        }

        Ok(program)
    }

    /// Decode: convert CVC5 result back to LS format
    pub fn decode_result(&self, result: &SolverResult) -> Result<HashMap<String, LSTerm>, String> {
        debug!("Decoding CVC5 result...");

        let mut decoded_results = HashMap::new();

        match &result.status {
            SolverStatus::Sat => {
                for (func_name, sygus_body) in &result.synthesis_results {
                    debug!("Decoding synthesis result for function: {}", func_name);

                    // Convert SyGuS term back to LSTerm
                    if let Some(ls_term) = self.sygus_term_to_ls_term(sygus_body)? {
                        decoded_results.insert(func_name.clone(), ls_term);
                    }
                }
            }
            SolverStatus::Unsat => {
                return Err("Synthesis problem is unsatisfiable".to_string());
            }
            SolverStatus::Unknown => {
                return Err("Synthesis result is unknown".to_string());
            }
            SolverStatus::Error(msg) => {
                return Err(format!("Solver error: {}", msg));
            }
        }

        debug!("Decoded {} synthesis results", decoded_results.len());
        Ok(decoded_results)
    }

    /// Convert SyGuS term to LSTerm (simplified implementation)
    fn sygus_term_to_ls_term(&self, sygus_term: &str) -> Result<Option<LSTerm>, String> {
        debug!("Converting SyGuS term to LSTerm: {}", sygus_term);

        // This is a simplified implementation
        // In a full implementation, we would parse the SyGuS term properly
        // For now, we'll just return None to indicate we can't decode yet
        debug!("Term conversion not implemented yet");
        Ok(None)
    }

    /// Get the translated file as a string (legacy method)
    pub fn to_string(&mut self) -> Result<String, String> {
        let translated_commands = self.translate()?;
        self.commands_to_string(&translated_commands)
    }

    /// Get the dynamically translated file as a string
    pub fn to_string_dynamic(&mut self) -> Result<String, String> {
        let translated_commands = self.translate_dynamically()?;
        self.commands_to_string(&translated_commands)
    }
}
