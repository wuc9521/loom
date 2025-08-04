use crate::lang::interpreter::Interpreter;
use crate::parser::ast::{LSFile, LSTerm};
use crate::translator::desugar::Desugarer;
use crate::translator::semantic::SemanticAnalyzer;
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
        info!("Starting CVC5 solver...");

        let mut command = Command::new(&self.executable_path);
        command.arg("--lang=sygus2");

        if let Some(timeout) = self.timeout {
            command.arg(format!("--tlimit={}", timeout.as_millis()));
        }

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| format!("Failed to start CVC5: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(sygus_program.as_bytes())
                .map_err(|e| format!("Failed to write to CVC5 stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to get CVC5 output: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        info!("CVC5 solver completed");
        debug!("CVC5 stdout: {}", stdout);
        debug!("CVC5 stderr: {}", stderr);

        self.decode_output(&stdout, &stderr)
    }

    fn decode_output(&self, stdout: &str, stderr: &str) -> Result<SolverResult, String> {
        let mut result = SolverResult {
            status: SolverStatus::Unknown,
            synthesis_results: HashMap::new(),
            error_message: None,
        };

        // Check for error messages
        if !stderr.is_empty() && !stderr.contains("WARNING") {
            result.status = SolverStatus::Error(stderr.to_string());
            return Ok(result);
        }

        // Parse solver status
        for line in stdout.lines() {
            let line = line.trim();
            if line == "sat" {
                result.status = SolverStatus::Sat;
            } else if line == "unsat" {
                result.status = SolverStatus::Unsat;
            } else if line == "unknown" {
                result.status = SolverStatus::Unknown;
            } else if line.starts_with("(define-fun") {
                // Parse synthesis result
                if let Some((func_name, func_body)) = self.parse_define_fun(line) {
                    result.synthesis_results.insert(func_name, func_body);
                }
            }
        }

        Ok(result)
    }

    fn parse_define_fun(&self, line: &str) -> Option<(String, String)> {
        // Simple parsing of (define-fun func_name ... body)
        if let Some(start) = line.find("(define-fun ") {
            let rest = &line[start + 12..]; // Skip "(define-fun "
            if let Some(end) = rest.find(' ') {
                let func_name = rest[..end].to_string();
                let func_body = line.to_string();
                return Some((func_name, func_body));
            }
        }
        None
    }
}

/// Main SyGuS translator with four-stage compilation pipeline
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
            context: TranslationContext::new(),
            synthesis_results: HashMap::new(),
            interpreter,
        }
    }

    /// Translate the LS file to SyGuS commands
    pub fn translate(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let semantic_result = semantic_analyzer.analyze(&self.original_file)?;
        let mut desugarer = Desugarer::new(self.original_file.clone(), semantic_result);
        let desugared_commands = desugarer.desugar_ast()?;
        Ok(desugared_commands)
    }

    /// Preprocess the LS file with the interpreter
    pub fn preprocess(&mut self) -> Result<(), String> {
        info!("Using legacy interpreter-based preprocessing");

        if let Some(ref mut interpreter) = self.interpreter {
            for cmd in &self.original_file.cmds {
                interpreter.proc(cmd)?;
            }

            // Extract type environment and runtime context
            self.context.type_environment = Some(interpreter.typing.global_scope().clone());

            // Extract runtime context from interpreter stack
            let mut runtime_context = HashMap::new();
            for scope in &interpreter.stack {
                for (name, value) in &scope.bindings {
                    runtime_context.insert(name.clone(), value.clone());
                }
            }
            self.context.runtime_context = Some(runtime_context);
        }

        Ok(())
    }

    /// Solve the SyGuS commands using the CVC5 solver
    pub fn solve(&self, sygus_commands: &[SyGuSCmd]) -> Result<SolverResult, String> {
        let solver = CVC5Solver::new();
        let sygus_string = self.commands_to_string(sygus_commands)?;
        solver.solve(&sygus_string)
    }

    /// Convert SyGuS commands to a string
    pub fn commands_to_string(&self, commands: &[SyGuSCmd]) -> Result<String, String> {
        let mut result = String::new();

        // Add set-logic if not present
        let has_logic = commands
            .iter()
            .any(|cmd| matches!(cmd, SyGuSCmd::SetLogic(_)));

        if !has_logic {
            result.push_str("(set-logic BV)\n");
        }

        // Add all commands
        for cmd in commands {
            result.push_str(&format!("{}\n", cmd));
        }

        // Add check-synth if not present
        let has_check = commands
            .iter()
            .any(|cmd| matches!(cmd, SyGuSCmd::CheckSynth));

        if !has_check {
            result.push_str("(check-synth)\n");
        }

        Ok(result)
    }

    /// Decode the solver result back to LS terms
    pub fn decode_result(&self, result: &SolverResult) -> Result<HashMap<String, LSTerm>, String> {
        let mut decoded_results = HashMap::new();

        match &result.status {
            SolverStatus::Sat => {
                for (func_name, func_body) in &result.synthesis_results {
                    if let Some(ls_term) = self.sygus_term_to_ls_term(func_body)? {
                        decoded_results.insert(func_name.clone(), ls_term);
                    }
                }
            }
            SolverStatus::Unsat => {
                info!("Solver returned UNSAT - no solution exists");
            }
            SolverStatus::Unknown => {
                info!("Solver returned UNKNOWN - could not determine satisfiability");
            }
            SolverStatus::Error(msg) => {
                return Err(format!("Solver error: {}", msg));
            }
        }

        Ok(decoded_results)
    }

    /// Convert a SyGuS term back to an LS term (simplified)
    fn sygus_term_to_ls_term(&self, _sygus_term: &str) -> Result<Option<LSTerm>, String> {
        // TODO: Implement proper SyGuS to LS term conversion
        // For now, return None to indicate no conversion
        Ok(None)
    }

    /// Convert the SyGuS commands to a string
    pub fn to_string(&mut self) -> Result<String, String> {
        let commands = self.translate()?;
        self.commands_to_string(&commands)
    }
}
