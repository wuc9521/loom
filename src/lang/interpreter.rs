use crate::lang::runtime::{Function, Scope, Value};
use crate::lang::types::{FunctionType, TypeInferer, ConstructorInfo};
use crate::parser::ast::{parse, LSCmd, LSTerm};
use crate::theory::{Logic, ModelCtx};
use log::debug;
use sygus_parser::{Sort, SyGuSCmd};

#[derive(Debug, Clone)]
pub enum SymbolLookupResult {
    Variable(Value),
    Constructor(ConstructorInfo),
    EnumElement { name: String, enum_type: String },
    NotFound,
}

#[derive(Debug)]
pub struct TheoryInterpreter {
    pub interpreter: Interpreter,
    pub name: String,
}

#[derive(Debug)]
pub struct Interpreter {
    pub logic: Option<Logic>,
    pub ctx: Option<ModelCtx>,
    pub last_model: Option<(crate::lang::model::Model, ModelCtx)>,
    pub stack: Vec<Scope>,
    pub typing: TypeInferer,
    pub theories: std::collections::HashMap<String, TheoryInterpreter>,
}

impl Interpreter {
    pub fn new() -> Self {
        debug!("interpreter created");
        let mut interpreter = Self {
            logic: None,
            ctx: None,
            last_model: None,
            stack: Vec::new(),
            typing: TypeInferer::new(),
            theories: std::collections::HashMap::new(),
        };

        interpreter.stack.push(Scope::new()); // initialize global scope

        // Register builtin constants in runtime scope
        interpreter.register_builtin_constants();

        interpreter
    }

    /// Register builtin constants in runtime scope
    fn register_builtin_constants(&mut self) {
        debug!("register built-in constants in runtime scope");
        self.global_scope_mut()
            .define("true".to_string(), Value::Bool(true));
        self.global_scope_mut()
            .define("false".to_string(), Value::Bool(false));
    }

    /// load theory file
    pub fn load_theory_file(&mut self, file_path: &str) -> Result<(), String> {
        debug!("Loading theory file: {}", file_path);
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read theory file: {}", e))?;

        debug!("Theory file content length: {}", content.len());
        let ls_file = parse(&content).map_err(|e| format!("Failed to parse theory file: {}", e))?;

        debug!("Parsed theory file with {} commands", ls_file.cmds.len());

        // find define-theory command and load to independent theory interpreter
        for cmd in ls_file.cmds.iter() {
            if let LSCmd::DefineTheory { name, cmds } = cmd {
                debug!("Loading theory: {} with {} commands", name, cmds.len());
                self.load_theory(name.clone(), cmds.clone())?;
            }
        }

        debug!("Loaded {} theories", self.theories.len());
        for theory_name in self.theories.keys() {
            debug!("Available theory: {}", theory_name);
        }

        Ok(())
    }

    /// load a theory.  
    pub fn load_theory(&mut self, name: String, cmds: Vec<LSCmd>) -> Result<(), String> {
        debug!("Creating theory interpreter for: {}", name);
        let mut theory_interpreter = TheoryInterpreter {
            interpreter: Interpreter::new(),
            name: name.clone(),
        };

        // process theory commands, but don't activate them immediately
        for cmd in cmds.iter() {
            debug!("{}", cmd);
            theory_interpreter.interpreter.proc(cmd)?;
        }

        let symbol_count = theory_interpreter.interpreter.global_scope().bindings.len();
        debug!(
            "Theory {} has {} symbols in global scope",
            name, symbol_count
        );

        // print all functions defined in the theory
        debug!("Theory {} functions in type environment:", name);
        for scope in &theory_interpreter.interpreter.typing.stack {
            for (_, func_type) in &scope.function_types {
                debug!("    {}", func_type);
            }
        }

        let theory_name = name.clone();
        self.theories.insert(name, theory_interpreter);
        debug!("Successfully loaded theory: {}", theory_name);

        Ok(())
    }

    /// activate a theory.
    pub fn activate_theory(&mut self, theory_name: &str) -> Result<(), String> {
        // first check if the theory exists, and collect the data we need
        let (bindings, functions_to_merge, variables_to_merge, datatypes_to_merge) = {
            if let Some(theory) = self.theories.get(theory_name) {
                debug!("Activating theory: {}", theory_name);

                // collect runtime bindings
                let bindings: Vec<(String, Value)> = theory
                    .interpreter
                    .global_scope()
                    .bindings
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                // collect type environment information
                let mut functions_to_merge = Vec::new();
                let mut variables_to_merge = Vec::new();
                let mut datatypes_to_merge = Vec::new();

                for scope in &theory.interpreter.typing.stack {
                    // collect function type definitions
                    for (func_name, func_type) in &scope.function_types {
                        functions_to_merge.push((func_name.clone(), func_type.clone()));
                    }

                    // collect variable type definitions
                    for (var_name, var_type) in &scope.variables {
                        variables_to_merge.push((var_name.clone(), var_type.clone()));
                    }

                    // collect datatype definitions
                    for (dt_name, dt_def) in &scope.datatypes {
                        datatypes_to_merge.push((dt_name.clone(), dt_def.clone()));
                    }
                }

                (
                    bindings,
                    functions_to_merge,
                    variables_to_merge,
                    datatypes_to_merge,
                )
            } else {
                return Err(format!("Theory '{}' not found", theory_name));
            }
        };

        // now merge to main interpreter safely
        // 1. merge runtime bindings
        for (name, value) in bindings {
            debug!("Merging binding from theory: {} = {}", name, value);
            // check if the symbol is already defined in global scope
            if !self.global_scope().bindings.contains_key(&name) {
                self.global_scope_mut().define(name, value);
            } else {
                debug!(
                    "Skipping theory binding '{}' as it already exists in global scope",
                    name
                );
            }
        }

        // 2. merge type environment
        for (func_name, func_type) in functions_to_merge {
            debug!("Merging function type from theory: {}", func_type);
            // check if the function already exists, if so, skip
            if !self.typing.global_scope().has_function(&func_name) {
                self.typing
                    .global_scope_mut()
                    .register_function_type(func_name, func_type);
            } else {
                debug!(
                    "Skipping theory function '{}' as it already exists in global scope",
                    func_name
                );
            }
        }

        for (var_name, var_type) in variables_to_merge {
            debug!(
                "Merging variable type from theory: {} :: {}",
                var_name, var_type
            );
            // check if the variable already exists, if so, skip
            if !self.typing.global_scope().has_variable(&var_name) {
                self.typing
                    .global_scope_mut()
                    .bind_variable(var_name, var_type);
            } else {
                debug!(
                    "Skipping theory variable '{}' as it already exists in global scope",
                    var_name
                );
            }
        }

        for (dt_name, dt_def) in datatypes_to_merge {
            debug!("Merging datatype from theory: {}", dt_name);
            // check if the datatype already exists, if so, skip
            if !self.typing.global_scope().has_datatype(&dt_name) {
                self.typing
                    .global_scope_mut()
                    .register_datatype(dt_name, dt_def);
            } else {
                debug!(
                    "Skipping theory datatype '{}' as it already exists in global scope",
                    dt_name
                );
            }
        }

        Ok(())
    }

    /// get global scope as mutable reference
    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.stack.first_mut().expect("Stack should never be empty")
    }

    /// handle constraint command
    fn handle_constraint(&mut self, term: &sygus_parser::SyGuSTerm) -> Result<(), String> {
        debug!("handling constraint term: {}", term);

        // recursively evaluate the constraint term
        self.eval_constraint_term(term)?;

        Ok(())
    }

    /// evaluate constraint term recursively
    fn eval_constraint_term(&mut self, term: &sygus_parser::SyGuSTerm) -> Result<Value, String> {
        match term {
            sygus_parser::SyGuSTerm::Application(func_name, args) => {
                debug!(
                    "evaluating constraint function call: {} with {} args",
                    func_name,
                    args.len()
                );

                // recursively evaluate arguments
                let mut evaluated_args = Vec::new();
                for arg in args {
                    let arg_value = self.eval_constraint_term(arg)?;
                    evaluated_args.push(arg_value);
                }

                // find and execute function
                self.execute_constraint_function(&func_name.to_string(), &evaluated_args)
            }
            sygus_parser::SyGuSTerm::Identifier(ident) => {
                debug!("evaluating constraint identifier: {}", ident);
                // find the value of the identifier
                if let Some(value) = self.lookup_symbol(&ident.to_string()) {
                    Ok(value)
                } else {
                    Err(format!("Undefined identifier in constraint: {}", ident))
                }
            }
            sygus_parser::SyGuSTerm::Literal(lit) => {
                debug!("evaluating constraint literal: {}", lit);
                // convert the literal to Value
                match lit {
                    sygus_parser::Literal::Bool(b) => Ok(Value::Bool(*b)),
                    sygus_parser::Literal::Numeral(n) => Ok(Value::Int(*n as i64)),
                    sygus_parser::Literal::Decimal(d) => Ok(Value::String(d.to_string())),
                    _ => Ok(Value::Symbol(lit.to_string())),
                }
            }
            _ => Err(format!("Unsupported constraint term type: {}", term)),
        }
    }

    /// execute constraint function
    fn execute_constraint_function(
        &mut self,
        func_name: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        // first try to find builtin functions
        match func_name {
            "not" => {
                if args.len() == 1 {
                    match &args[0] {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        Value::Symbol(sym) => {
                            debug!("not function got symbol argument: {}", sym);
                            // return true temporarily to avoid panic
                            Ok(Value::Bool(true))
                        }
                        _ => Err(format!(
                            "not function expects boolean argument, got: {}",
                            args[0]
                        )),
                    }
                } else {
                    Err(format!(
                        "not function expects 1 argument, got: {}",
                        args.len()
                    ))
                }
            }
            _ => {
                // then try to find user-defined functions in current scope
                if let Some(value) = self.lookup_symbol(func_name) {
                    match value {
                        Value::Function(func) => {
                            debug!("found function: {}", func.name);
                            // TODO: implement function call logic
                            Ok(Value::Symbol(func.name.clone()))
                        }
                        Value::Symbol(sym) => {
                            debug!("found function symbol: {}", sym);
                            Ok(Value::Symbol(sym.clone()))
                        }
                        _ => Err(format!("Symbol {} is not a function", func_name)),
                    }
                } else {
                    // try to find in theories and execute there
                    for theory in self.theories.values() {
                        if let Some(value) = theory.interpreter.lookup_symbol(func_name) {
                            match value {
                                Value::Function(func) => {
                                    // TODO: implement cross-theory function call
                                    return Ok(Value::Symbol(func.name.clone()));
                                }
                                Value::Symbol(sym) => {
                                    return Ok(Value::Symbol(sym.clone()));
                                }
                                _ => continue,
                            }
                        }
                    }
                    Err(format!("Undefined function in constraint: {}", func_name))
                }
            }
        }
    }

    /// get current scope as immutable reference
    pub fn current_scope(&self) -> &Scope {
        self.stack.last().expect("Stack should never be empty")
    }

    /// get global scope as immutable reference
    pub fn global_scope(&self) -> &Scope {
        self.stack.first().expect("Stack should never be empty")
    }

    /// get current scope as mutable reference
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.stack.last_mut().expect("Stack should never be empty")
    }

    /// interpret a file content.
    pub fn interp(&mut self, content: &str) -> Result<(), String> {
        let ls_file = parse(content).map_err(|e| format!("Parse error: {}", e))?;
        for (i, cmd) in ls_file.cmds.iter().enumerate() {
            debug!("[cmd {}] {}", i + 1, cmd);
            self.proc(cmd)?;
        }
        Ok(())
    }

    /// process a command, including type inference.
    pub fn proc(&mut self, cmd: &LSCmd) -> Result<(), String> {
        debug!("{}", cmd);
        match cmd {
            LSCmd::Echo(term) => {
                // evaluate the term
                let value = self.eval(term)?;
                println!("{}", value);
            }
            LSCmd::DefineTheory { name, cmds } => self.load_theory(name.clone(), cmds.clone())?,
            LSCmd::As(sygus_cmd) => self.as_sygus_cmd(sygus_cmd)?,
            LSCmd::DeclareFunc(func_type) => {
                // check conflicts
                let func_name = func_type
                    .get_name()
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                self.typing.check_all_conflicts(&func_name, "function")?;

                self.typing
                    .current_scope_mut()
                    .register_function_type(func_name.clone(), func_type.clone());
                self.current_scope_mut()
                    .define(func_name.clone(), Value::Symbol(func_name.clone()));
            }
            LSCmd::DefineFunc(func_type) => {
                let func_name = func_type
                    .get_name()
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                debug!("defining function: {}", func_name);
                // check conflicts
                self.typing.check_all_conflicts(&func_name, "function")?;

                // Type check the function definition
                self.type_check_function_definition_unified(&func_type)?;
                if let Some(body) = func_type.get_body() {
                    self.define_function(
                        func_name.clone(),
                        func_type.get_param_names().to_vec(), // use actual parameter names
                        body.clone(),
                    )?;
                }
            }
            LSCmd::DefineFuncRec(func_type) => {
                let func_name = func_type
                    .get_name()
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                // check conflicts
                self.typing.check_all_conflicts(&func_name, "function")?;

                self.process_recursive_function_unified(&func_type)?;
            }
            LSCmd::ModelDef(model) => {
                debug!("defining model: {}", model.name);
                // check conflicts
                self.typing.check_all_conflicts(&model.name, "model")?;

                // Type check the model before defining it
                self.type_check_model(model.clone())?;
                self.define_model(model.clone())?;
            }
            LSCmd::ThSpec(th_cmd) => {
                debug!("handling theory-specific command: {}", th_cmd.name);
                let _ = self.handle_thspec_cmd(th_cmd);
            }
            LSCmd::DefineEnumSort { name, elems } => {
                debug!(
                    "defining enum sort: {} with elements: {}",
                    name,
                    elems
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                // check conflicts
                self.typing.check_all_conflicts(name, "enum")?;

                self.typing
                    .current_scope_mut()
                    .register_enum(name.clone(), elems.clone());

                // register enum values to runtime environment
                for elem in elems {
                    debug!("Registering enum element: {}", elem);
                    self.current_scope_mut()
                        .define(elem.clone(), Value::Symbol(elem.clone()));
                }
            }
        }
        Ok(())
    }

    /// update type environment from sygus command.
    fn as_sygus_cmd(&mut self, cmd: &SyGuSCmd) -> Result<(), String> {
        match cmd {
            SyGuSCmd::SetLogic(logic) => {
                debug!("set logic: {}", logic);
                // check if the logic is a loaded theory
                if self.theories.contains_key(logic) {
                    debug!("Activating theory for logic: {}", logic);
                    self.activate_theory(logic)?;
                }
                // set logic type
                self.logic = match logic.as_str() {
                    "FOL" => Some(Logic::FOL),
                    "PML" => Some(Logic::PML),
                    "RE" => Some(Logic::RE),
                    "BV" => Some(Logic::BV),
                    "CFE" => Some(Logic::CFE),
                    "SLIA" => Some(Logic::SLIA),
                    _ => {
                        // if it's a custom theory, try to activate it
                        if self.theories.contains_key(logic) {
                            debug!("Using custom theory as logic: {}", logic);
                            Some(Logic::RE) // temporarily use RE as default
                        } else {
                            debug!("Unknown logic: {}", logic);
                            None
                        }
                    }
                };
            }
            SyGuSCmd::DeclareDatatype(name, dtdecl) => {
                debug!("Declaring datatype: {}", name);
                self.typing.check_all_conflicts(name, "datatype")?;

                let data_type = crate::lang::types::adapter::from_dtdecl(name, dtdecl);
                debug!(
                    "Registering datatype: {} with {} constructors",
                    name,
                    data_type.constructors.len()
                );
                
                // register type information only, do not create runtime values
                self.typing
                    .current_scope_mut()
                    .register_datatype(name.clone(), data_type);
            }

            SyGuSCmd::DeclareSort(name, arity) => {
                debug!("declare sort: {}", name);
                // check conflicts
                self.typing.check_all_conflicts(name, "sort")?;

                // register the sort as a simple sort with the given arity
                let sort = Sort::from_name(name.as_str());
                self.typing
                    .current_scope_mut()
                    .bind_variable(name.clone(), sort);
                debug!("{} :: sort with arity {}", name, arity);
            }
            SyGuSCmd::DefineSort(name, _params, sort) => {
                debug!("define sort: {}", name);
                // check conflicts
                self.typing.check_all_conflicts(name, "sort")?;

                self.typing
                    .current_scope_mut()
                    .bind_variable(name.clone(), sort.clone());
                debug!("{} :: {}", name, sort);
            }
            SyGuSCmd::Constraint(term) => {
                debug!("handling constraint: {}", term);
                // parse function calls in constraint
                self.handle_constraint(term)?;
            }
            SyGuSCmd::SynthFun(name, args, ret_sort, _grammar) => {
                debug!("synth-fun: {}", name);
                // check conflicts
                self.typing.check_all_conflicts(name, "function")?;

                // register function type using FunctionType
                let param_sorts: Vec<sygus_parser::Sort> =
                    args.iter().map(|sv| sv.sort.clone()).collect();
                let func_type =
                    FunctionType::new_signature(name.clone(), param_sorts, ret_sort.clone());
                self.typing
                    .current_scope_mut()
                    .register_function_type(name.clone(), func_type);
                // store the function into the scope
                self.current_scope_mut()
                    .define(name.clone(), Value::Symbol(name.clone()));
            }
            SyGuSCmd::DefineFun(name, args, ret_sort, body) => {
                debug!("define-fun: {} with {} args", name, args.len());
                // check conflicts
                self.typing.check_all_conflicts(name, "function")?;

                // convert SyGuSTerm to LSTerm
                let body_str = body.to_string();
                let body_term = {
                    use crate::parser::{LSParser, Rule};
                    use pest::Parser;

                    let mut pairs = LSParser::parse(Rule::LSTerm, &body_str)
                        .map_err(|e| format!("Failed to parse function body as LSTerm: {}", e))?;

                    if let Some(pair) = pairs.next() {
                        crate::parser::ast::LSTerm::parse(pair)
                            .map_err(|e| format!("Failed to parse LSTerm: {}", e))?
                    } else {
                        return Err("No LSTerm found".to_string());
                    }
                };

                // create FunctionType
                let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
                let param_sorts: Vec<sygus_parser::Sort> =
                    args.iter().map(|sv| sv.sort.clone()).collect();
                let func_type = FunctionType::new_definition(
                    name.clone(),
                    param_names.clone(),
                    param_sorts,
                    ret_sort.clone(),
                    body_term.clone(),
                );

                // type check
                self.type_check_function_definition_unified(&func_type)?;

                // define function
                self.define_function(name.clone(), param_names, body_term)?;
            }
            SyGuSCmd::CheckSynth => {}
            _ => unimplemented!("unsupported sygus command: {:?}", cmd),
        }
        Ok(())
    }

    /// process recursive function definition (unified version).
    fn process_recursive_function_unified(
        &mut self,
        func_type: &FunctionType,
    ) -> Result<(), String> {
        let func_name = func_type
            .get_name()
            .unwrap_or(&"unknown".to_string())
            .clone();

        // 1. register function type (recursion)
        self.typing
            .current_scope_mut()
            .register_function_type(func_name.clone(), func_type.clone());
        self.typing.enter_scope();

        // bind parameter types with actual names
        let param_names = func_type.get_param_names();
        for (i, param_type) in func_type.get_params().iter().enumerate() {
            let param_name = if i < param_names.len() {
                param_names[i].clone()
            } else {
                format!("param{}", i)
            };
            self.typing
                .current_scope_mut()
                .bind_variable(param_name, param_type.clone());
        }

        // infer function body type
        if let Some(body) = func_type.get_body() {
            let body_type = self.typing.infer(body)?;

            // check return type
            if body_type != *func_type.get_ret() {
                return Err(format!(
                    "Function body type {} does not match declared return type {}",
                    body_type,
                    func_type.get_ret()
                ));
            }

            self.define_function(
                func_name.clone(),
                func_type.get_param_names().to_vec(), // convert to Vec<String>
                body.clone(),
            )?;
        }

        self.typing.exit_scope();
        Ok(())
    }

    /// lookup a symbol in the scope stack
    pub fn lookup_symbol(&self, name: &str) -> Option<Value> {
        // lookup in the scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.lookup(name) {
                return Some(value);
            }
        }

        // if not found in current scopes, try to find in theories
        for theory in self.theories.values() {
            if let Some(value) = theory.interpreter.lookup_symbol(name) {
                return Some(value);
            }
        }

        None
    }

    /// lookup a symbol in the scope stack, returning a more detailed result.
    pub fn lookup_symbol_enhanced(&self, name: &str) -> SymbolLookupResult {
        // first check if it is a constructor
        for scope in self.typing.stack.iter().rev() {
            if let Some(constructor_info) = scope.get_constructor(name) {
                return SymbolLookupResult::Constructor(constructor_info.clone());
            }
        }
        
        // then check normal variables
        if let Some(value) = self.lookup_symbol(name) {
            return SymbolLookupResult::Variable(value);
        }
        
        // finally check enum elements
        for scope in self.typing.stack.iter().rev() {
            for (_, enum_type) in &scope.enums {
                if enum_type.elements.contains(&name.to_string()) {
                    return SymbolLookupResult::EnumElement {
                        name: name.to_string(),
                        enum_type: enum_type.name.clone(),
                    };
                }
            }
        }
        
        SymbolLookupResult::NotFound
    }

    /// define a symbol in the current scope
    pub fn define_symbol(&mut self, name: String, value: Value) {
        self.current_scope_mut().define(name, value);
    }

    /// evaluate a constructor call.
    pub fn eval_constructor_call(&mut self, info: &ConstructorInfo, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != info.field_types.len() {
            return Err(format!(
                "Constructor {} expects {} arguments, got {}",
                info.name,
                info.field_types.len(),
                args.len()
            ));
        }
        
        let mut field_values = Vec::new();
        for arg in args {
            field_values.push(self.interp_expr(arg)?);
        }
        
        Ok(Value::DataValue {
            constructor: info.name.clone(),
            datatype: info.datatype.clone(),
            fields: field_values,
        })
    }

    /// interpret an expression, including type checking
    pub fn interp_expr(&mut self, expr: &LSTerm) -> Result<Value, String> {
        // first do type checking
        self.typing.infer(expr)?;
        // then evaluate
        self.eval(expr)
    }

    /// enter a new scope
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new();
        self.stack.push(new_scope);
    }

    /// exit the current scope
    pub fn exit_scope(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            debug!("exit scope, current stack depth: {}", self.stack.len());
        } else {
            debug!("try to exit root scope");
        }
    }

    /// define a function.
    pub fn define_function(
        &mut self,
        name: String,
        params: Vec<String>,
        body: LSTerm,
    ) -> Result<(), String> {
        debug!(
            "Defining function '{}' with {} parameters",
            name,
            params.len()
        );

        // create function closure
        let function = Function {
            name: name.clone(),
            params,
            body,
            closure: self.current_scope().clone(),
        };

        // register to current scope
        self.define_symbol(name.clone(), Value::Function(function));
        debug!("Function '{}' defined successfully", name);
        Ok(())
    }

    /// Type check a function definition (unified version)
    fn type_check_function_definition_unified(
        &mut self,
        func_type: &FunctionType,
    ) -> Result<(), String> {
        let func_name = func_type
            .get_name()
            .unwrap_or(&"unknown".to_string())
            .clone();
        debug!("type checking function: {}", func_name);

        // Register function type
        self.typing
            .current_scope_mut()
            .register_function_type(func_name.clone(), func_type.clone());

        // Enter function scope for type checking
        self.typing.enter_scope();

        // Bind parameter types using actual parameter names
        let param_names = func_type.get_param_names();
        for (i, param_type) in func_type.get_params().iter().enumerate() {
            let param_name = if i < param_names.len() {
                param_names[i].clone()
            } else {
                format!("param{}", i)
            };
            debug!("Binding parameter: {} :: {}", param_name, param_type);
            self.typing
                .current_scope_mut()
                .bind_variable(param_name, param_type.clone());
        }

        // Type check function body
        if let Some(body) = func_type.get_body() {
            let body_type = self.typing.infer(body)?;

            // Check return type
            if body_type != *func_type.get_ret() {
                return Err(format!(
                    "Function {} body type {} does not match declared return type {}",
                    func_name,
                    body_type,
                    func_type.get_ret()
                ));
            }
        }

        // Exit function scope
        self.typing.exit_scope();

        debug!("function type checking completed successfully");
        Ok(())
    }

    /// Type check a model definition
    fn type_check_model(&mut self, model: crate::lang::model::Model) -> Result<(), String> {
        debug!("type checking model: {}", model.name);

        // Enter model scope for type checking
        self.typing.enter_scope();

        // Register type parameters in the type environment
        for type_param in &model.type_params {
            debug!("registering type parameter: {}", type_param);
            self.typing.current_scope_mut().bind_variable(
                type_param.clone(),
                sygus_parser::Sort::Simple(sygus_parser::Identifier::Symbol(type_param.clone())),
            );
        }

        // Type check enum sort definitions and other LSCmd in model
        for ext_cmd in &model.ext_cmds {
            if ext_cmd.name == "define-sort-enum" {
                debug!("enum sort definition found in model: {}", ext_cmd.args);
                // Parse the enum definition from the command
                // The format should be: (define-sort-enum Name ($0 $1 $2 ...))
                // We need to extract the name and elements from ext_cmd.args
                // For now, we'll assume the enum is already registered by the proc method
            }
            // Add type checking for other theory-specific commands if needed
        }

        // Type check set-univ definitions
        for univ_def in &model.univ_defs {
            debug!(
                "type checking set-univ: {} :: {}",
                univ_def.name, univ_def.sort
            );
            // Verify that the sort is well-formed
            self.typing
                .current_scope_mut()
                .bind_variable(univ_def.name.clone(), univ_def.sort.clone());
        }

        // Exit model scope
        self.typing.exit_scope();

        debug!("model type checking completed successfully");
        Ok(())
    }

    /// define a model.
    pub fn define_model(&mut self, model: crate::lang::model::Model) -> Result<(), String> {
        self.enter_scope();
        for type_param in &model.type_params {
            // register (univ) type parameters
            // Bind type parameter as a sort in the type environment
            self.typing
                .current_scope_mut()
                .bind_variable(type_param.clone(), Sort::from_str(type_param).unwrap());
        }
        self.define_symbol(model.name.clone(), Value::Model(model.clone()));

        // handle extension commands
        for ext_cmd in &model.ext_cmds {
            let _ = self.handle_thspec_cmd(ext_cmd);
        }

        // note: here we don't exit scope, because model scope should persist
        // we only exit scope when processing next model

        Ok(())
    }

    /// handle theory specific command.
    pub fn handle_thspec_cmd(
        &mut self,
        cmd: &crate::lang::model::ThSpecCmd,
    ) -> Result<Value, String> {
        debug!("handle theory command: {}", cmd);
        // here we can add theory specific handling logic
        Ok(Value::Symbol("theory_command".to_string()))
    }
}
