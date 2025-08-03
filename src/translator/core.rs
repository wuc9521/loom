use crate::lang::interpreter::Interpreter;
use crate::parser::ast::{LSCmd, LSFile, LSTerm};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;

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

    /// Generate a unique symbol name
    pub fn generate_unique_name(&mut self, base_name: &str, namespace: &str) -> String {
        let mut candidate = format!("{}@{}", base_name, namespace);
        let mut counter = 0;

        while self.used_symbols.contains(&candidate) {
            counter += 1;
            candidate = format!("{}@{}.{}", base_name, namespace, counter);
        }

        self.used_symbols.insert(candidate.clone());
        candidate
    }

    /// Map a symbol with namespace
    pub fn map_symbol(&mut self, original: &str, namespace: &str) -> String {
        if let Some(mapped) = self.forward.get(original) {
            return mapped.clone();
        }

        let translated = self.generate_unique_name(original, namespace);
        self.forward
            .insert(original.to_string(), translated.clone());
        self.reverse
            .insert(translated.clone(), original.to_string());
        // Note: namespaces are no longer used in the new design

        debug!(
            "Mapped symbol: {} -> {} (namespace: {})",
            original, translated, namespace
        );
        translated
    }

    /// Map an object method to a flattened function name
    pub fn map_object_method(&mut self, object_name: &str, method_name: &str) -> String {
        let key = (object_name.to_string(), method_name.to_string());
        if let Some(mapped) = self.object_methods.get(&key) {
            return mapped.clone();
        }

        let flattened_name = format!("{}_{}", object_name, method_name);
        self.object_methods.insert(key, flattened_name.clone());
        self.used_symbols.insert(flattened_name.clone());

        debug!(
            "Mapped object method: {}.{} -> {}",
            object_name, method_name, flattened_name
        );
        flattened_name
    }

    /// Map a class method to a template function name
    pub fn map_class_method(&mut self, class_name: &str, method_name: &str) -> String {
        let key = (class_name.to_string(), method_name.to_string());
        if let Some(mapped) = self.class_methods.get(&key) {
            return mapped.clone();
        }

        let template_name = format!("{}_{}", class_name, method_name);
        self.class_methods.insert(key, template_name.clone());
        self.used_symbols.insert(template_name.clone());

        debug!(
            "Mapped class method: {}.{} -> {}",
            class_name, method_name, template_name
        );
        template_name
    }

    /// Get the original symbol from translated symbol
    pub fn get_original(&self, translated: &str) -> Option<&String> {
        self.reverse.get(translated)
    }

    /// Get the translated symbol from original symbol
    pub fn get_translated(&self, original: &str) -> Option<&String> {
        self.forward.get(original)
    }

    /// Check if a symbol is already mapped
    pub fn is_mapped(&self, original: &str) -> bool {
        self.forward.contains_key(original)
    }

    /// Get the flattened function name for an object method
    pub fn get_object_method(&self, object_name: &str, method_name: &str) -> Option<&String> {
        let key = (object_name.to_string(), method_name.to_string());
        self.object_methods.get(&key)
    }

    /// Get the template function name for a class method
    pub fn get_class_method(&self, class_name: &str, method_name: &str) -> Option<&String> {
        let key = (class_name.to_string(), method_name.to_string());
        self.class_methods.get(&key)
    }

    /// Check if a symbol is an object method call
    pub fn is_object_method_call(&self, func_name: &str, first_arg: &str) -> bool {
        self.get_object_method(first_arg, func_name).is_some()
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

    /// Map an object field to a flattened function name
    pub fn map_object_field(&mut self, object_name: &str, field_name: &str) -> String {
        let key = (object_name.to_string(), field_name.to_string());
        if let Some(mapped) = self.object_fields.get(&key) {
            return mapped.clone();
        }

        let flattened_name = format!("{}_{}", object_name, field_name);
        self.object_fields.insert(key, flattened_name.clone());

        debug!(
            "Mapped object field: {}.{} -> {}",
            object_name, field_name, flattened_name
        );
        flattened_name
    }

    /// Get the flattened function name for an object field
    pub fn get_object_field(&self, object_name: &str, field_name: &str) -> Option<&String> {
        let key = (object_name.to_string(), field_name.to_string());
        self.object_fields.get(&key)
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
        self.decode_cvc5_output(&stdout, &stderr)
    }

    fn decode_cvc5_output(&self, stdout: &str, stderr: &str) -> Result<SolverResult, String> {
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

    /// Collect all symbols and create mappings to avoid conflicts
    fn collect_symbols_for_mapping(&mut self) -> Result<(), String> {
        debug!("Collecting symbols for mapping...");

        // Use the interpreter's runtime information to collect all symbols
        if let Some(interpreter) = &self.interpreter {
            // Get all symbols from the global scope
            let global_scope = interpreter.global_scope();

            for (name, value) in &global_scope.bindings {
                match value {
                    crate::lang::runtime::Value::Function(_function) => {
                        // This is a function (including object methods)
                        debug!("Found function: {}", name);
                        self.context
                            .symbol_mapping
                            .forward
                            .insert(name.clone(), name.clone());
                    }
                    crate::lang::runtime::Value::Object(object_value) => {
                        // This is an object - map its methods and fields
                        debug!(
                            "Found object: {} of class {}",
                            object_value.name, object_value.class_name
                        );

                        // Map object methods
                        for (method_name, _) in &object_value.methods {
                            let flattened_name = format!("{}_{}", object_value.name, method_name);
                            self.context.symbol_mapping.object_methods.insert(
                                (object_value.name.clone(), method_name.clone()),
                                flattened_name.clone(),
                            );
                            self.context
                                .symbol_mapping
                                .forward
                                .insert(method_name.clone(), flattened_name);
                        }

                        // Map object fields
                        for (field_name, _) in &object_value.fields {
                            let flattened_name = format!("{}@{}", field_name, object_value.name);
                            self.context.object_fields.insert(
                                (object_value.name.clone(), field_name.clone()),
                                flattened_name.clone(),
                            );
                        }
                    }
                    crate::lang::runtime::Value::Class(class_value) => {
                        // This is a class - map its methods as templates
                        debug!(
                            "Found class: {} with {} methods",
                            class_value.name,
                            class_value.methods.len()
                        );

                        for (method_name, _) in &class_value.methods {
                            let template_name = format!("{}_{}", class_value.name, method_name);
                            self.context.symbol_mapping.class_methods.insert(
                                (class_value.name.clone(), method_name.clone()),
                                template_name.clone(),
                            );
                        }
                    }
                    _ => {
                        // Other values (constants, etc.)
                        debug!("Found other symbol: {}", name);
                        self.context
                            .symbol_mapping
                            .forward
                            .insert(name.clone(), name.clone());
                    }
                }
            }
        }

        debug!(
            "Symbol mapping completed with {} object methods, {} class methods",
            self.context.symbol_mapping.object_methods.len(),
            self.context.symbol_mapping.class_methods.len()
        );
        Ok(())
    }

    /// Phase 2: Dynamic translation based on complete context
    pub fn translate_dynamically(&mut self) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        if self.context.type_environment.is_none() {
            return Err(
                "Type environment not available. Run preprocess_with_interpreter first."
                    .to_string(),
            );
        }

        // First pass: collect all symbols and create mappings
        self.collect_symbols_for_mapping()?;

        // Second pass: translate with symbol mappings
        // Create separate vectors for different command types to ensure correct ordering
        let mut definitions = Vec::new(); // For declare-sort, declare-datatype, define-fun
        let mut synthesis_block = Vec::new(); // For synth-fun, synth-inv
        let mut constraints = Vec::new(); // For constraint, check-synth

        // Add set-logic at the beginning
        definitions.push(sygus_parser::SyGuSCmd::SetLogic("BV".to_string()));

        // Collect all commands first to avoid borrow checker issues
        let mut object_defs = Vec::new();
        let mut recursive_funcs = Vec::new();
        let mut enum_sorts = Vec::new();
        let mut sygus_commands = Vec::new();
        let mut class_defs = Vec::new();
        let mut func_defs = Vec::new();

        // Track which enum elements have been processed to avoid duplicates
        let mut processed_enum_elements = HashSet::new();

        for cmd in &self.original_file.cmds {
            match cmd {
                LSCmd::ObjectDef(object_def) => {
                    object_defs.push(object_def.clone());
                }
                LSCmd::DefineFuncRec(func_type) => {
                    recursive_funcs.push(func_type.clone());
                }
                LSCmd::DefineEnumSort { name, elems } => {
                    enum_sorts.push((name.clone(), elems.clone()));
                    // Mark all enum elements as processed
                    for elem in elems {
                        processed_enum_elements.insert(elem.clone());
                    }
                }
                LSCmd::As(sygus_cmd) => {
                    // Skip DefineFun commands that define enum elements
                    if let sygus_parser::SyGuSCmd::DefineFun(func_name, _, _, _) = sygus_cmd {
                        if processed_enum_elements.contains(func_name) {
                            debug!("Skipping enum element definition: {}", func_name);
                            continue;
                        }
                    }
                    sygus_commands.push(sygus_cmd.clone());
                }
                LSCmd::ClassDef(class_def) => {
                    class_defs.push(class_def.clone());
                }
                LSCmd::DefineFunc(func_type) => {
                    func_defs.push(func_type.clone());
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

        // Process enum sorts first (they need mutable self)
        for (name, elems) in enum_sorts {
            let enum_cmd = self.translate_enum_sort_dynamically(&name, &elems)?;
            definitions.extend(enum_cmd);
        }

        // Process SyGuS commands
        for sygus_cmd in sygus_commands {
            // Sort standard SyGuS commands into appropriate vectors
            match sygus_cmd {
                sygus_parser::SyGuSCmd::DeclareSort(_, _)
                | sygus_parser::SyGuSCmd::DeclareDatatype(_, _)
                | sygus_parser::SyGuSCmd::DefineFun(_, _, _, _) => {
                    let translated_cmd = self.translate_sygus_command(&sygus_cmd)?;
                    definitions.push(translated_cmd);
                }
                sygus_parser::SyGuSCmd::SynthFun(_, _, _, _) => {
                    let translated_cmd = self.translate_sygus_command(&sygus_cmd)?;
                    synthesis_block.push(translated_cmd);
                }
                sygus_parser::SyGuSCmd::Constraint(_) | sygus_parser::SyGuSCmd::CheckSynth => {
                    let translated_cmd = self.translate_sygus_command(&sygus_cmd)?;
                    constraints.push(translated_cmd);
                }
                _ => {
                    // For other commands, keep as-is
                    definitions.push(sygus_cmd);
                }
            }
        }

        // Process class definitions
        for class_def in class_defs {
            let translated_cmds = self.translate_class_definition_dynamically(&class_def)?;
            definitions.extend(translated_cmds);
        }

        // Process function definitions
        for func_type in func_defs {
            let translated_cmd = self.translate_function_definition_dynamically(&func_type)?;
            info!("Translated function definition: {:?}", translated_cmd);
            definitions.push(translated_cmd);
        }

        // Process object definitions
        for object_def in object_defs {
            let translated_cmds = self.translate_object_definition_dynamically(&object_def)?;
            definitions.extend(translated_cmds);
        }

        // Process recursive functions
        for func_type in recursive_funcs {
            let translated_cmds = self.translate_recursive_function_dynamically(&func_type)?;
            definitions.extend(translated_cmds);
        }

        // Combine vectors in the correct order
        definitions.append(&mut synthesis_block);
        definitions.append(&mut constraints);

        Ok(definitions)
    }

    /// Translate SyGuS command to handle object method calls in constraints
    fn translate_sygus_command(
        &self,
        sygus_cmd: &sygus_parser::SyGuSCmd,
    ) -> Result<sygus_parser::SyGuSCmd, String> {
        match sygus_cmd {
            sygus_parser::SyGuSCmd::Constraint(term) => {
                // Translate constraint terms to handle object method calls
                let translated_term = self.translate_sygus_term(term)?;
                Ok(sygus_parser::SyGuSCmd::Constraint(translated_term))
            }
            _ => {
                // For other commands, keep as-is for now
                Ok(sygus_cmd.clone())
            }
        }
    }

    /// Translate SyGuS term to handle object method calls
    fn translate_sygus_term(
        &self,
        term: &sygus_parser::SyGuSTerm,
    ) -> Result<sygus_parser::SyGuSTerm, String> {
        match term {
            sygus_parser::SyGuSTerm::Application(func, args) => {
                let translated_args: Result<Vec<sygus_parser::SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.translate_sygus_term(arg))
                    .collect();

                // Check if this is an object method call: (models w1 regex) -> (models@w1 regex)
                if let sygus_parser::Identifier::Symbol(func_name) = func {
                    if let Some(first_arg) = args.first() {
                        if let sygus_parser::SyGuSTerm::Identifier(
                            sygus_parser::Identifier::Symbol(obj_name),
                        ) = first_arg
                        {
                            // Check if this looks like an object method call
                            if self.is_class_method_call(func_name, obj_name) {
                                // Get the flattened function name with correct format: w1_models instead of models@w1
                                let flattened_name = format!("{}_{}", obj_name, func_name);

                                // Skip the first argument (object) and translate the rest
                                let remaining_args: Result<Vec<sygus_parser::SyGuSTerm>, String> =
                                    args[1..]
                                        .iter()
                                        .map(|arg| self.translate_sygus_term(arg))
                                        .collect();

                                return Ok(sygus_parser::SyGuSTerm::Application(
                                    sygus_parser::Identifier::Symbol(flattened_name),
                                    remaining_args?,
                                ));
                            }
                        }
                    }
                }

                Ok(sygus_parser::SyGuSTerm::Application(
                    func.clone(),
                    translated_args?,
                ))
            }
            sygus_parser::SyGuSTerm::Identifier(ident) => {
                // Handle enum value references in SyGuS terms (e.g., $0, $1, $a, $b)
                if let sygus_parser::Identifier::Symbol(sym) = ident {
                    if sym.starts_with('$') {
                        // Find the enum sort that contains this element
                        for cmd in &self.original_file.cmds {
                            if let LSCmd::DefineEnumSort { name, elems: enum_elems } = cmd {
                                if enum_elems.contains(sym) {
                                    // Calculate the bit width and value
                                    use crate::translator::utils::{calculate_bit_width, bv};
                                    let width = calculate_bit_width(enum_elems.len());
                                    let index = enum_elems.iter().position(|e| e == sym).unwrap_or(0);
                                    let bv_literal = bv(index as u64, width);
                                    
                                    return Ok(sygus_parser::SyGuSTerm::Identifier(
                                        sygus_parser::Identifier::Symbol(bv_literal),
                                    ));
                                }
                            }
                        }
                    }
                }
                Ok(sygus_parser::SyGuSTerm::Identifier(ident.clone()))
            }
            sygus_parser::SyGuSTerm::Literal(lit) => {
                Ok(sygus_parser::SyGuSTerm::Literal(lit.clone()))
            }
            _ => {
                // For other terms, keep as-is for now
                Ok(term.clone())
            }
        }
    }

    /// Translate enum sort definition dynamically
    fn translate_enum_sort_dynamically(
        &mut self,
        name: &str,
        elems: &[String],
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        debug!(
            "Dynamically translating enum sort: {} with elements: {:?}",
            name, elems
        );

        use crate::translator::utils::{bv, calculate_bit_width};

        let mut commands = Vec::new();

        // 1. Calculate the required bit vector width
        let width = calculate_bit_width(elems.len());
        let bv_sort = sygus_parser::Sort::Simple(sygus_parser::Identifier::Symbol(format!(
            "(_ BitVec {})",
            width
        )));

        // 2. Declare the sort as a bit vector
        commands.push(sygus_parser::SyGuSCmd::DefineSort(
            name.to_string(),
            vec![],
            bv_sort.clone(),
        ));

        // 3. Define each enum element as a bit vector constant
        for (i, elem) in elems.iter().enumerate() {
            let elem_name = elem.trim_start_matches('$'); // Remove the $ prefix
            let bv_literal = bv(i as u64, width);
            let bv_value =
                sygus_parser::SyGuSTerm::Identifier(sygus_parser::Identifier::Symbol(bv_literal));

            let cmd = sygus_parser::SyGuSCmd::DefineFun(
                elem_name.to_string(),
                vec![],
                bv_sort.clone(),
                bv_value,
            );
            commands.push(cmd);

            // Store the mapping for later use
            self.context
                .symbol_mapping
                .forward
                .insert(elem.clone(), elem_name.to_string());
        }

        Ok(commands)
    }

    /// Translate class definition dynamically
    fn translate_class_definition_dynamically(
        &self,
        class_def: &crate::parser::ast::ClassDef,
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        debug!(
            "Dynamically translating class definition: {}",
            class_def.name
        );

        // Class definitions are not directly translated - they serve as templates
        // for object method instantiation. Return empty vector.
        Ok(Vec::new())
    }

    /// Translate object definition dynamically using generic AST-driven logic
    fn translate_object_definition_dynamically(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        debug!(
            "Translating object using generic AST-driven logic: {}",
            object_def.name
        );

        let mut translated_cmds = Vec::new();
        let class = self.find_class_for_object(object_def)?.clone();

        // Find the semantic function definition from the class
        let semantic_func_def = class
            .methods
            .iter()
            .find(|m| {
                self.is_semantic_function(m, &class)
                    .unwrap_or(None)
                    .is_some()
            })
            .ok_or("Could not find a semantic function like 'eval' in the class definition")?;

        // Translate the semantic function (this will generate the eval and all bv_helpers)
        let semantic_cmds =
            self.translate_semantic_function(object_def, semantic_func_def, &class)?;
        translated_cmds.extend(semantic_cmds);

        // Translate ALL OTHER methods (size, toInt, charAt, models, iv-len) as simple methods
        for method_impl in &object_def.methods {
            if self.is_semantic_function(method_impl, &class)?.is_none() {
                let cmd = self.translate_simple_object_method(object_def, method_impl)?;
                translated_cmds.push(cmd);
            }
        }

        Ok(translated_cmds)
    }

    /// Translate function definition dynamically
    fn translate_function_definition_dynamically(
        &self,
        func_type: &crate::lang::types::FunctionType,
    ) -> Result<sygus_parser::SyGuSCmd, String> {
        let unknown_str = "unknown".to_string();
        let func_name = func_type.get_name().unwrap_or(&unknown_str);
        debug!("Dynamically translating function definition: {}", func_name);

        // Check finiteness before translation
        if !self.check_function_finiteness(func_type)? {
            return Err(format!(
                "Function {} cannot be translated: finiteness check failed",
                func_name
            ));
        }

        if let Some(body) = func_type.get_body() {
            let param_names = func_type.get_param_names();
            let param_sorts = func_type.get_params();
            let ret_sort = func_type.get_ret();

            // Get the mapped function name
            let translated_func_name = self
                .context
                .symbol_mapping
                .get_translated(func_name)
                .unwrap_or(func_name)
                .clone();

            // Convert function body to standard SyGuS term
            let translated_body = self.translate_term_dynamically(body)?;

            // Create sorted variables
            let sorted_vars: Vec<sygus_parser::SortedVar> = param_names
                .iter()
                .zip(param_sorts.iter())
                .map(|(name, sort)| {
                    let name_clone = name.clone();
                    let sort_clone = sort.clone();
                    sygus_parser::SortedVar {
                        name: name_clone,
                        sort: sort_clone,
                    }
                })
                .collect();

            Ok(sygus_parser::SyGuSCmd::DefineFun(
                translated_func_name,
                sorted_vars,
                ret_sort,
                translated_body,
            ))
        } else {
            Err(format!("Function {} has no body", func_name))
        }
    }

    /// Translate recursive function definition dynamically
    fn translate_recursive_function_dynamically(
        &mut self,
        func_type: &crate::lang::types::FunctionType,
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        let unknown_str = "unknown".to_string();
        let func_name = func_type.get_name().unwrap_or(&unknown_str);
        debug!(
            "Dynamically translating recursive function definition: {}",
            func_name
        );

        // Check finiteness before translation (critical for recursive functions)
        if !self.check_function_finiteness(func_type)? {
            return Err(format!(
                "Recursive function {} cannot be translated: finiteness check failed",
                func_name
            ));
        }

        // Check if the function contains a match term
        if let Some(body) = func_type.get_body() {
            if self.contains_match_term(body) {
                // Use the new match expansion method
                return self.expand_match_into_functions(func_type, None);
            }
        }

        // Fall back to simple translation for non-match recursive functions
        if let Some(body) = func_type.get_body() {
            let param_names = func_type.get_param_names();
            let param_sorts = func_type.get_params();
            let ret_sort = func_type.get_ret();

            // Get the mapped function name
            let translated_func_name = self
                .context
                .symbol_mapping
                .get_translated(func_name)
                .unwrap_or(func_name)
                .clone();

            // Convert function body to standard SyGuS term
            let translated_body = self.translate_term_dynamically(body)?;

            // Create sorted variables
            let sorted_vars: Vec<sygus_parser::SortedVar> = param_names
                .iter()
                .zip(param_sorts.iter())
                .map(|(name, sort)| {
                    let name_clone = name.clone();
                    let sort_clone = sort.clone();
                    sygus_parser::SortedVar {
                        name: name_clone,
                        sort: sort_clone,
                    }
                })
                .collect();

            // For now, use DefineFun instead of DefineFunRec since DefineFunRec might not be available
            Ok(vec![sygus_parser::SyGuSCmd::DefineFun(
                translated_func_name,
                sorted_vars,
                ret_sort,
                translated_body,
            )])
        } else {
            Err(format!("Recursive function {} has no body", func_name))
        }
    }

    /// Check if a term contains a match expression
    pub fn contains_match_term(&self, term: &LSTerm) -> bool {
        match term {
            LSTerm::MatchTerm { .. } => true,
            LSTerm::AppTerm(_, args) => {
                for arg in args {
                    if self.contains_match_term(arg) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if a function call is a class method call on an object
    fn is_class_method_call(&self, func_name: &str, obj_name: &str) -> bool {
        // Use the interpreter's runtime information to check if this is a class method call
        if let Some(interpreter) = &self.interpreter {
            // Check if the object exists in the global scope
            if let Some(crate::lang::runtime::Value::Object(_)) =
                interpreter.global_scope().lookup(obj_name)
            {
                // Check if the function exists in the global scope (as a method)
                if let Some(crate::lang::runtime::Value::Function(_)) =
                    interpreter.global_scope().lookup(func_name)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Translate an LSTerm to standard SyGuS term dynamically
    fn translate_term_dynamically(&self, term: &LSTerm) -> Result<sygus_parser::SyGuSTerm, String> {
        debug!("Dynamically translating term: {}", term);

        match term {
            LSTerm::Identifier(ident) => {
                // Handle 'self' references in class method bodies
                if let sygus_parser::Identifier::Symbol(sym) = &ident {
                    if sym == "self" {
                        // 'self' should be replaced with object-specific references
                        // For now, we'll use a placeholder that will be resolved during method instantiation
                        return Ok(sygus_parser::SyGuSTerm::Identifier(
                            sygus_parser::Identifier::Symbol("__self_placeholder__".to_string()),
                        ));
                    }

                    // Handle enum value references (e.g., $a, $b, $0, $1)
                    if sym.starts_with('$') {
                        let enum_value = &sym[1..]; // Remove the '$' prefix
                        
                        // Look up the bit vector value for this enum element
                        if let Some(translated_name) = self.context.symbol_mapping.forward.get(sym) {
                            // Find the enum sort that contains this element
                            for cmd in &self.original_file.cmds {
                                if let LSCmd::DefineEnumSort { name, elems: enum_elems } = cmd {
                                    if enum_elems.contains(sym) {
                                        // Calculate the bit width and value
                                        use crate::translator::utils::{calculate_bit_width, bv};
                                        let width = calculate_bit_width(enum_elems.len());
                                        let index = enum_elems.iter().position(|e| e == sym).unwrap_or(0);
                                        let bv_literal = bv(index as u64, width);
                                        
                                        return Ok(sygus_parser::SyGuSTerm::Identifier(
                                            sygus_parser::Identifier::Symbol(bv_literal),
                                        ));
                                    }
                                }
                            }
                        }
                        
                        // Fallback: convert enum value reference to function call
                        return Ok(sygus_parser::SyGuSTerm::Application(
                            sygus_parser::Identifier::Symbol(enum_value.to_string()),
                            vec![], // No arguments - this is a constant function
                        ));
                    }
                }
                Ok(sygus_parser::SyGuSTerm::Identifier(ident.clone()))
            }
            LSTerm::Literal(lit) => Ok(sygus_parser::SyGuSTerm::Literal(lit.clone())),
            LSTerm::AppTerm(func, args) => {
                let translated_args: Result<Vec<sygus_parser::SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.translate_term_dynamically(arg))
                    .collect();

                // Check if this is a class method call on an object
                // Pattern: (models w1 regex) -> (w1_models regex)
                if let sygus_parser::Identifier::Symbol(func_name) = &func {
                    if let Some(first_arg) = args.first() {
                        if let LSTerm::Identifier(sygus_parser::Identifier::Symbol(obj_name)) =
                            first_arg
                        {
                            // Check if this looks like a class method call
                            if self.is_class_method_call(func_name, obj_name) {
                                // Get the flattened function name for this object method
                                if let Some(flattened_name) = self
                                    .context
                                    .symbol_mapping
                                    .get_object_method(obj_name, func_name)
                                {
                                    // Skip the first argument (object) and translate the rest
                                    let remaining_args: Result<
                                        Vec<sygus_parser::SyGuSTerm>,
                                        String,
                                    > = args[1..]
                                        .iter()
                                        .map(|arg| self.translate_term_dynamically(arg))
                                        .collect();

                                    return Ok(sygus_parser::SyGuSTerm::Application(
                                        sygus_parser::Identifier::Symbol(flattened_name.clone()),
                                        remaining_args?,
                                    ));
                                }
                            }
                        }
                    }
                }

                Ok(sygus_parser::SyGuSTerm::Application(
                    func.clone(),
                    translated_args?,
                ))
            }
            LSTerm::MethodCallTerm { object, method } => {
                // Translate method call to object-specific function call
                // (models w1 regex) -> (w1_models regex)
                if let Some(flattened_name) = self
                    .context
                    .symbol_mapping
                    .get_object_method(object, method)
                {
                    Ok(sygus_parser::SyGuSTerm::Application(
                        sygus_parser::Identifier::Symbol(flattened_name.clone()),
                        vec![], // No arguments - the object context is bound in the function
                    ))
                } else {
                    // Fallback to original format if not found
                    let object_method_name = format!("{}_{}", object, method);
                    Ok(sygus_parser::SyGuSTerm::Application(
                        sygus_parser::Identifier::Symbol(object_method_name),
                        vec![],
                    ))
                }
            }
            LSTerm::MatchTerm { term, cases } => {
                // Translate pattern matching to nested if-then-else
                self.translate_match_term_dynamically(term, cases)
            }
            LSTerm::LambdaTerm { args: _, body: _ } => {
                unimplemented!()
            }
            LSTerm::MuTerm {
                name: _,
                args: _,
                ret_sort: _,
                body: _,
                params: _,
            } => {
                unimplemented!()
            }
            _ => {
                // For complex terms, try to translate based on type environment
                self.translate_complex_term_dynamically(term)
            }
        }
    }

    /// Translate term with self replacement for object methods
    pub fn translate_term_with_self_replacement(
        &self,
        term: &LSTerm,
        object_name: &str,
    ) -> Result<sygus_parser::SyGuSTerm, String> {
        match term {
            LSTerm::Identifier(ident) => {
                // Replace 'self' with object-specific field access
                if let sygus_parser::Identifier::Symbol(sym) = &ident {
                    if sym == "self" {
                        // 'self' should be replaced with object-specific field access
                        // For now, we'll use a placeholder that will be resolved during method instantiation
                        return Ok(sygus_parser::SyGuSTerm::Identifier(
                            sygus_parser::Identifier::Symbol("__self_placeholder__".to_string()),
                        ));
                    }
                }
                Ok(sygus_parser::SyGuSTerm::Identifier(ident.clone()))
            }
            LSTerm::Literal(lit) => Ok(sygus_parser::SyGuSTerm::Literal(lit.clone())),
            LSTerm::AppTerm(func, args) => {
                let translated_args: Result<Vec<sygus_parser::SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.translate_term_with_self_replacement(arg, object_name))
                    .collect();

                // Check if this is a field access on self: (field self) -> (object_field)
                if let sygus_parser::Identifier::Symbol(field_name) = &func {
                    if let Some(first_arg) = args.first() {
                        if let LSTerm::Identifier(sygus_parser::Identifier::Symbol(sym)) = first_arg
                        {
                            if sym == "self" {
                                // This is a field access: (field self) -> (object_field)
                                let flattened_field_name =
                                    format!("{}@{}", field_name, object_name);
                                return Ok(sygus_parser::SyGuSTerm::Application(
                                    sygus_parser::Identifier::Symbol(flattened_field_name),
                                    vec![],
                                ));
                            }
                        }
                    }
                }

                // Check if this is a method call that should be object-specific
                if let sygus_parser::Identifier::Symbol(func_name) = &func {
                    // Use the interpreter's runtime information to check if this is an object method
                    if let Some(interpreter) = &self.interpreter {
                        // Check if this function exists in the global scope
                        if let Some(crate::lang::runtime::Value::Function(_)) =
                            interpreter.global_scope().lookup(func_name)
                        {
                            // This is a function that should be object-specific
                            let object_method_name = format!("{}@{}", func_name, object_name);
                            return Ok(sygus_parser::SyGuSTerm::Application(
                                sygus_parser::Identifier::Symbol(object_method_name),
                                translated_args?,
                            ));
                        }
                    }
                }

                Ok(sygus_parser::SyGuSTerm::Application(
                    func.clone(),
                    translated_args?,
                ))
            }
            LSTerm::MethodCallTerm { object, method } => {
                // Translate method call to object-specific function call
                if let Some(flattened_name) = self
                    .context
                    .symbol_mapping
                    .get_object_method(object, method)
                {
                    Ok(sygus_parser::SyGuSTerm::Application(
                        sygus_parser::Identifier::Symbol(flattened_name.clone()),
                        vec![],
                    ))
                } else {
                    // Fallback to original format if not found
                    let object_method_name = format!("{}_{}", object, method);
                    Ok(sygus_parser::SyGuSTerm::Application(
                        sygus_parser::Identifier::Symbol(object_method_name),
                        vec![],
                    ))
                }
            }
            LSTerm::MatchTerm { term, cases: _ } => {
                // For match terms, we need to handle self replacement in the term and cases
                let _translated_term =
                    self.translate_term_with_self_replacement(term, object_name)?;
                // For now, return a placeholder for match terms
                Ok(sygus_parser::SyGuSTerm::Identifier(
                    sygus_parser::Identifier::Symbol("__match_placeholder__".to_string()),
                ))
            }
            LSTerm::LambdaTerm { args: _, body: _ } => {
                unimplemented!()
            }
            LSTerm::MuTerm {
                name: _,
                args: _,
                ret_sort: _,
                body: _,
                params: _,
            } => {
                unimplemented!()
            }
            _ => {
                // For complex terms, try to translate based on type environment
                self.translate_complex_term_dynamically(term)
            }
        }
    }

    /// Translate match term to nested if-then-else
    fn translate_match_term_dynamically(
        &self,
        scrutinee: &LSTerm,
        _cases: &[crate::lang::model::MatchCase],
    ) -> Result<sygus_parser::SyGuSTerm, String> {
        // For now, skip complex pattern matching translation
        debug!("Skipping complex match term translation: {}", scrutinee);
        Ok(sygus_parser::SyGuSTerm::Identifier(
            sygus_parser::Identifier::Symbol("__match_placeholder__".to_string()),
        ))
    }

    /// Translate complex terms using type environment
    fn translate_complex_term_dynamically(
        &self,
        term: &LSTerm,
    ) -> Result<sygus_parser::SyGuSTerm, String> {
        // For now, return a placeholder for complex terms
        // In a full implementation, you would use the type environment to determine the correct translation
        debug!("Complex term translation not fully implemented: {}", term);
        Ok(sygus_parser::SyGuSTerm::Identifier(
            sygus_parser::Identifier::Symbol("__complex_term_placeholder__".to_string()),
        ))
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
    pub fn translate(&mut self) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
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
                LSCmd::DefineEnumSort { name, elems } => {
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
    fn translate_enum_sort(
        &self,
        name: &str,
        elems: &[String],
    ) -> Result<sygus_parser::SyGuSCmd, String> {
        debug!("Translating enum sort: {} with elements: {:?}", name, elems);

        // For now, skip enum sort translation to avoid complex type issues
        // We'll implement this properly later
        debug!("Skipping enum sort translation for now");

        // Return a placeholder command
        Ok(sygus_parser::SyGuSCmd::DeclareSort(
            name.to_string(),
            0, // arity
        ))
    }

    /// Solve: call CVC5
    pub fn solve(&self, sygus_commands: &[sygus_parser::SyGuSCmd]) -> Result<SolverResult, String> {
        debug!("Preparing to call CVC5 solver...");

        // Convert commands to string
        let sygus_program = self.commands_to_string(sygus_commands)?;
        debug!("SyGuS program:\n{}", sygus_program);

        // Call CVC5
        let solver = CVC5Solver::new();
        solver.solve(&sygus_program)
    }

    /// Convert SyGuS commands to string
    pub fn commands_to_string(
        &self,
        commands: &[sygus_parser::SyGuSCmd],
    ) -> Result<String, String> {
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

    /// Check if a function can be safely translated based on finiteness
    fn check_function_finiteness(
        &self,
        func_type: &crate::lang::types::FunctionType,
    ) -> Result<bool, String> {
        let unknown_str = "unknown".to_string();
        let func_name = func_type.get_name().unwrap_or(&unknown_str);
        debug!("Checking finiteness for function: {}", func_name);

        // Get type environment
        let type_env = if let Some(env) = &self.context.type_environment {
            env
        } else {
            return Err("Type environment not available for finiteness check".to_string());
        };

        // Check if function body contains recursive patterns
        if let Some(body) = func_type.get_body() {
            if self.contains_recursive_patterns(body) {
                debug!(
                    "Function {} contains recursive patterns, checking finiteness",
                    func_name
                );
                return self.check_recursive_function_finiteness(func_type, type_env);
            }
        }

        // Non-recursive functions are always safe to translate
        Ok(true)
    }

    /// Check if a term contains recursive patterns that require finiteness
    fn contains_recursive_patterns(&self, term: &LSTerm) -> bool {
        match term {
            LSTerm::MuTerm { .. } | LSTerm::NuTerm { .. } => true,
            LSTerm::AppTerm(_, args) => {
                args.iter().any(|arg| self.contains_recursive_patterns(arg))
            }
            LSTerm::MatchTerm { term, cases } => {
                self.contains_recursive_patterns(term)
                    || cases
                        .iter()
                        .any(|case| self.contains_recursive_patterns(&case.value))
            }
            LSTerm::LambdaTerm { body, .. } => self.contains_recursive_patterns(body),
            LSTerm::ForallTerm { body, .. } | LSTerm::ExistsTerm { body, .. } => {
                self.contains_recursive_patterns(body)
            }
            LSTerm::LetTerm { args, body } => {
                args.iter()
                    .any(|binding| self.contains_recursive_patterns(&binding.term))
                    || self.contains_recursive_patterns(body)
            }
            _ => false,
        }
    }

    /// Check finiteness for recursive functions
    fn check_recursive_function_finiteness(
        &self,
        func_type: &crate::lang::types::FunctionType,
        type_env: &crate::lang::types::TypeEnv,
    ) -> Result<bool, String> {
        let unknown_str = "unknown".to_string();
        let func_name = func_type.get_name().unwrap_or(&unknown_str);

        // Get parameter types
        let param_sorts = func_type.get_params();

        // Check if all parameters are finite with precise cardinality information
        for (i, sort) in param_sorts.iter().enumerate() {
            if let Some(finiteness) = type_env.is_finite(sort) {
                match finiteness {
                    crate::lang::types::Finiteness::Finite(cardinality) => {
                        debug!(
                            "Parameter {} of function {} is finite with cardinality {}",
                            i, func_name, cardinality
                        );

                        // Additional check: if cardinality is 0, this is problematic
                        if cardinality == 0 {
                            return Err(format!(
                                "Function {} cannot be translated: parameter {} has empty type {}",
                                func_name, i, sort
                            ));
                        }
                    }
                    crate::lang::types::Finiteness::Infinite => {
                        return Err(format!(
                            "Function {} cannot be translated: parameter {} has infinite type {}",
                            func_name, i, sort
                        ));
                    }
                }
            } else {
                // Unknown type - this is more nuanced than just assuming infinite
                // Let's check if it's a known infinite type by name
                if let sygus_parser::Sort::Simple(identifier) = sort {
                    if let Some(is_finite) = type_env.is_finite_by_name(&identifier.to_string()) {
                        if !is_finite {
                            return Err(format!(
                                "Function {} cannot be translated: parameter {} has infinite type {}",
                                func_name, i, sort
                            ));
                        } else {
                            debug!(
                                "Parameter {} of function {} is finite (by name check)",
                                i, func_name
                            );
                        }
                    } else {
                        // Truly unknown type, be conservative
                        return Err(format!(
                            "Function {} cannot be translated: parameter {} has unknown type {}",
                            func_name, i, sort
                        ));
                    }
                } else {
                    // Complex type, assume infinite for safety
                    return Err(format!(
                        "Function {} cannot be translated: parameter {} has complex unknown type {}",
                        func_name, i, sort
                    ));
                }
            }
        }

        // Check return type with precise cardinality
        let ret_sort = func_type.get_ret();
        if let Some(finiteness) = type_env.is_finite(&ret_sort) {
            match finiteness {
                crate::lang::types::Finiteness::Finite(cardinality) => {
                    debug!(
                        "Return type of function {} is finite with cardinality {}",
                        func_name, cardinality
                    );
                }
                crate::lang::types::Finiteness::Infinite => {
                    return Err(format!(
                        "Function {} cannot be translated: return type {} is infinite",
                        func_name, ret_sort
                    ));
                }
            }
        } else {
            // Check return type by name
            if let sygus_parser::Sort::Simple(identifier) = &ret_sort {
                if let Some(is_finite) = type_env.is_finite_by_name(&identifier.to_string()) {
                    if !is_finite {
                        return Err(format!(
                            "Function {} cannot be translated: return type {} is infinite",
                            func_name, ret_sort
                        ));
                    } else {
                        debug!(
                            "Return type of function {} is finite (by name check)",
                            func_name
                        );
                    }
                } else {
                    return Err(format!(
                        "Function {} cannot be translated: return type {} is unknown",
                        func_name, ret_sort
                    ));
                }
            } else {
                return Err(format!(
                    "Function {} cannot be translated: return type {} is complex and unknown",
                    func_name, ret_sort
                ));
            }
        }

        // Check function body for specific patterns
        if let Some(body) = func_type.get_body() {
            if self.contains_mu_terms(body) {
                debug!(
                    "Function {} contains mu-terms, performing additional checks",
                    func_name
                );
                return self.check_mu_term_finiteness(body, type_env);
            }
        }

        Ok(true)
    }

    /// Check if a term contains mu-terms
    fn contains_mu_terms(&self, term: &LSTerm) -> bool {
        match term {
            LSTerm::MuTerm { .. } => true,
            LSTerm::AppTerm(_, args) => args.iter().any(|arg| self.contains_mu_terms(arg)),
            LSTerm::MatchTerm { term, cases } => {
                self.contains_mu_terms(term)
                    || cases.iter().any(|case| self.contains_mu_terms(&case.value))
            }
            LSTerm::LambdaTerm { body, .. } => self.contains_mu_terms(body),
            LSTerm::ForallTerm { body, .. } | LSTerm::ExistsTerm { body, .. } => {
                self.contains_mu_terms(body)
            }
            LSTerm::LetTerm { args, body } => {
                args.iter()
                    .any(|binding| self.contains_mu_terms(&binding.term))
                    || self.contains_mu_terms(body)
            }
            _ => false,
        }
    }

    /// Check finiteness for mu-terms specifically
    fn check_mu_term_finiteness(
        &self,
        term: &LSTerm,
        type_env: &crate::lang::types::TypeEnv,
    ) -> Result<bool, String> {
        match term {
            LSTerm::MuTerm {
                name,
                args,
                ret_sort,
                body,
                params,
            } => {
                debug!("Checking mu-term: {}", name);

                // Check if the mu-term's parameter type is finite
                for arg in args {
                    if let Some(finiteness) = type_env.is_finite(&arg.sort) {
                        match finiteness {
                            crate::lang::types::Finiteness::Finite(_) => {
                                debug!("Mu-term parameter {} is finite", arg.name);
                            }
                            crate::lang::types::Finiteness::Infinite => {
                                return Err(format!(
                                    "Mu-term {} cannot be translated: parameter {} has infinite type {}",
                                    name, arg.name, arg.sort
                                ));
                            }
                        }
                    } else {
                        return Err(format!(
                            "Mu-term {} cannot be translated: parameter {} has unknown type {}",
                            name, arg.name, arg.sort
                        ));
                    }
                }

                // Check return type
                if let Some(finiteness) = type_env.is_finite(ret_sort) {
                    match finiteness {
                        crate::lang::types::Finiteness::Finite(_) => {
                            debug!("Mu-term return type is finite");
                        }
                        crate::lang::types::Finiteness::Infinite => {
                            return Err(format!(
                                "Mu-term {} cannot be translated: return type {} is infinite",
                                name, ret_sort
                            ));
                        }
                    }
                } else {
                    return Err(format!(
                        "Mu-term {} cannot be translated: return type {} is unknown",
                        name, ret_sort
                    ));
                }

                // Recursively check body and param
                self.check_mu_term_finiteness(body, type_env)?;
                for param in params {
                    self.check_mu_term_finiteness(&param, type_env)?;
                }

                Ok(true)
            }
            LSTerm::AppTerm(_, args) => {
                for arg in args {
                    self.check_mu_term_finiteness(arg, type_env)?;
                }
                Ok(true)
            }
            LSTerm::MatchTerm { term, cases } => {
                self.check_mu_term_finiteness(term, type_env)?;
                for case in cases {
                    self.check_mu_term_finiteness(&case.value, type_env)?;
                }
                Ok(true)
            }
            LSTerm::LambdaTerm { body, .. } => self.check_mu_term_finiteness(body, type_env),
            LSTerm::ForallTerm { body, .. } | LSTerm::ExistsTerm { body, .. } => {
                self.check_mu_term_finiteness(body, type_env)
            }
            LSTerm::LetTerm { args, body } => {
                for binding in args {
                    self.check_mu_term_finiteness(&binding.term, type_env)?;
                }
                self.check_mu_term_finiteness(body, type_env)
            }
            _ => Ok(true),
        }
    }

    /// Safe wrapper for object definition translation to avoid borrow checker issues
    fn _translate_object_definition_safe(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        // Clone the object definition to avoid borrow checker issues
        let object_def_clone = object_def.clone();
        self.translate_object_definition_dynamically(&object_def_clone)
    }

    /// Safe wrapper for recursive function translation to avoid borrow checker issues
    fn _translate_recursive_function_safe(
        &mut self,
        func_type: &crate::lang::types::FunctionType,
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        // Clone the function type to avoid borrow checker issues
        let func_type_clone = func_type.clone();
        self.translate_recursive_function_dynamically(&func_type_clone)
    }

    /// Expands a recursive function with a `match` term into a set of flat, mutually
    /// recursive functions.
    fn expand_match_into_functions(
        &mut self,
        func_def: &crate::lang::types::FunctionType,
        object_context: Option<&str>, // e.g., Some("w1") for an object method, None for a global function
    ) -> Result<Vec<sygus_parser::SyGuSCmd>, String> {
        let unknown_str = "unknown".to_string();
        let func_name = func_def.get_name().unwrap_or(&unknown_str);
        debug!(
            "Expanding match function: {} with object context: {:?}",
            func_name, object_context
        );

        // For now, return a placeholder implementation
        // This will be implemented in the next iteration
        debug!(
            "Match expansion not yet implemented for function: {}",
            func_name
        );

        // Return the original function as a simple DefineFun for now
        if let Some(body) = func_def.get_body() {
            let param_names = func_def.get_param_names();
            let param_sorts = func_def.get_params();
            let ret_sort = func_def.get_ret();

            let translated_func_name = if let Some(obj_name) = object_context {
                format!("{}_{}", obj_name, func_name)
            } else {
                func_name.clone()
            };

            let sorted_vars: Vec<sygus_parser::SortedVar> = param_names
                .iter()
                .zip(param_sorts.iter())
                .map(|(name, sort)| {
                    let name_clone = name.clone();
                    let sort_clone = sort.clone();
                    sygus_parser::SortedVar {
                        name: name_clone,
                        sort: sort_clone,
                    }
                })
                .collect();

            let translated_body = self.translate_term_dynamically(body)?;

            Ok(vec![sygus_parser::SyGuSCmd::DefineFun(
                translated_func_name,
                sorted_vars,
                ret_sort,
                translated_body,
            )])
        } else {
            Err(format!("Function {} has no body", func_name))
        }
    }
}
