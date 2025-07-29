use crate::lang::runtime::{Function, Scope, Value};
use crate::lang::types::{FunctionType, TypeInferer};
use crate::parser::ast::{parse, LSCmd, LSTerm};
use crate::theory::{Logic, ModelCtx};
use log::{debug, error};
use std::collections::HashMap;
use sygus_parser::{Sort, SyGuSCmd};

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
    pub theories: HashMap<String, TheoryInterpreter>,
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
            theories: HashMap::new(),
        };

        interpreter.stack.push(Scope::new()); // initialize global scope

        interpreter
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
            debug!("Processing theory command: {}", cmd);
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
        debug!(
            "executing constraint function: {} with {} args",
            func_name,
            args.len()
        );

        // first try to find the function in the current scope
        if let Some(value) = self.lookup_symbol(func_name) {
            debug!("found function {} in current scope", func_name);
            match value {
                Value::Function(func) => {
                    // TODO: actually execute the function call
                    debug!("found function definition: {}", func.name);
                    // return true temporarily to avoid panic
                    Ok(Value::Bool(true))
                }
                Value::Symbol(sym) => {
                    debug!("found function symbol: {}", sym);
                    // if it's just a symbol, try to find the actual function definition
                    if let Some(func_value) = self.lookup_symbol(&sym) {
                        match func_value {
                            Value::Function(func) => {
                                debug!("found actual function definition: {}", func.name);
                                // TODO: actually execute function call
                                Ok(Value::Bool(true))
                            }
                            _ => Ok(Value::Symbol(sym.clone())),
                        }
                    } else {
                        Ok(Value::Symbol(sym.clone()))
                    }
                }
                _ => Err(format!("Symbol {} is not a function", func_name)),
            }
        } else {
            // try to find the function in all theories
            debug!(
                "function {} not found in current scope, checking theories",
                func_name
            );
            for theory_name in self.theories.keys() {
                debug!(
                    "checking theory: {} for function: {}",
                    theory_name, func_name
                );
                if let Some(theory) = self.theories.get(theory_name) {
                    if let Some(value) = theory.interpreter.lookup_symbol(func_name) {
                        debug!("found function {} in theory {}", func_name, theory_name);
                        match value {
                            Value::Function(func) => {
                                // TODO: actually execute the theory function call
                                debug!("found theory function definition: {}", func.name);
                                // return true temporarily to avoid panic
                                return Ok(Value::Bool(true));
                            }
                            Value::Symbol(sym) => {
                                debug!("found theory function symbol: {}", sym);
                                // try to find the actual function definition in the theory
                                if let Some(func_value) = theory.interpreter.lookup_symbol(&sym) {
                                    match func_value {
                                        Value::Function(func) => {
                                            debug!(
                                                "found actual theory function definition: {}",
                                                func.name
                                            );
                                            // TODO: actually execute function call
                                            return Ok(Value::Bool(true));
                                        }
                                        _ => return Ok(Value::Symbol(sym.clone())),
                                    }
                                } else {
                                    return Ok(Value::Symbol(sym.clone()));
                                }
                            }
                            _ => continue,
                        }
                    } else {
                        debug!("function {} not found in theory {}", func_name, theory_name);
                    }
                }
            }

            // if the function is a builtin function, try to execute it
            match func_name {
                "not" => {
                    if args.len() == 1 {
                        match &args[0] {
                            Value::Bool(b) => Ok(Value::Bool(!b)),
                            Value::Symbol(sym) => {
                                // if the argument is a symbol, try to parse it as a boolean value
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
                _ => Err(format!("Undefined function in constraint: {}", func_name)),
            }
        }
    }

    /// get current scope as immutable reference
    fn current_scope(&self) -> &Scope {
        self.stack.last().expect("Stack should never be empty")
    }

    /// get global scope as immutable reference
    pub fn global_scope(&self) -> &Scope {
        self.stack.first().expect("Stack should never be empty")
    }

    /// get current scope as mutable reference
    fn current_scope_mut(&mut self) -> &mut Scope {
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
        debug!("processing command: {}", cmd);
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
                        func_type
                            .get_params()
                            .iter()
                            .map(|_| "param".to_string())
                            .collect(),
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
                debug!("declare data type: {}", name);
                // check conflicts
                self.typing.check_all_conflicts(name, "datatype")?;

                let data_type = crate::lang::types::adapter::from_dtdecl(name, dtdecl);
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
            SyGuSCmd::DefineFun(name, args, ret_sort, _body) => {
                debug!("define-fun: {} with {} args", name, args.len());
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
                func_type
                    .get_params()
                    .iter()
                    .map(|_| "param".to_string())
                    .collect(),
                body.clone(),
            )?;
        }

        self.typing.exit_scope();
        Ok(())
    }

    /// evaluate an expression
    pub fn eval(&mut self, term: &LSTerm) -> Result<Value, String> {
        match term {
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                // use unified scope lookup
                if let Some(value) = self.lookup_symbol(&sym) {
                    Ok(value)
                } else {
                    error!("undefined symbol: {}", sym);
                    Err(format!("Undefined symbol: {}", sym))
                }
            }
            LSTerm::Literal(lit) => match lit {
                sygus_parser::Literal::Numeral(n) => {
                    let value = Value::Int(*n as i64);
                    Ok(value)
                }
                sygus_parser::Literal::Decimal(d) => {
                    let value = Value::String(d.to_string());
                    Ok(value)
                }
                _ => Ok(Value::Symbol(lit.to_string())),
            },
            LSTerm::AppTerm(func, args) => {
                debug!(
                    "evaluate function call: {} (parameters number: {})",
                    func.to_string(),
                    args.len()
                );
                self.eval_func_app(&func.to_string(), args)
            }
            LSTerm::AttrTerm(term, _attrs) => self.eval(term),
            LSTerm::MatchTerm { term, cases } => {
                let value = self.eval(term)?;
                self.eval_match(&value, cases)
            }
            LSTerm::LambdaTerm { args, body } => {
                let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
                let function = Function {
                    name: "lambda".to_string(),
                    params: param_names,
                    body: *body.clone(),
                    closure: self.current_scope().clone(),
                };
                Ok(Value::Function(function))
            }
            LSTerm::MuTerm {
                name,
                args: _args,
                body,
                param: _,
                ret_sort: _,
            } => self.eval_mu(name, body),
            LSTerm::NuTerm {
                name: _name,
                args: _args,
                body: _body,
                param: _,
                ret_sort: _,
            } => {
                // TODO: implement nu expression
                unimplemented!()
            }
            LSTerm::ForallTerm { args: _, body } => {
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
            LSTerm::ExistsTerm { args: _, body } => {
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
            LSTerm::LetTerm { args, body } => {
                debug!("evaluate let expression (bindings number: {})", args.len());
                self.enter_scope();
                for binding in args {
                    let value = self.eval(&binding.term)?;
                    self.current_scope_mut().define(binding.name.clone(), value);
                }
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
        }
    }

    /// lookup a symbol in the scope stack
    fn lookup_symbol(&self, name: &str) -> Option<Value> {
        // lookup in the scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.lookup(name) {
                debug!("find symbol: {} = {}", name, value);
                return Some(value);
            }
        }

        // if not found in current scopes, try to find in theories
        for theory in self.theories.values() {
            if let Some(value) = theory.interpreter.lookup_symbol(name) {
                debug!(
                    "find symbol in theory {}: {} = {}",
                    theory.name, name, value
                );
                return Some(value);
            }
        }

        None
    }

    /// define a symbol in the current scope
    fn define_symbol(&mut self, name: String, value: Value) {
        self.current_scope_mut().define(name, value);
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

    /// evaluate a function call
    fn eval_func_app(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Value, String> {
        debug!(
            "evaluating function call: {} with {} arguments",
            func_name,
            args.len()
        );

        // first try to find builtin functions
        if let Some(builtin_result) = self.eval_builtin_function(func_name, args) {
            return builtin_result;
        }

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
                    debug!("found function in theory {}: {}", theory.name, func_name);
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
            Err(format!("Undefined function: {}", func_name))
        }
    }

    /// evaluate a builtin function
    fn eval_builtin_function(
        &mut self,
        func_name: &str,
        args: &[LSTerm],
    ) -> Option<Result<Value, String>> {
        match func_name {
            "+" => Some(self.add(args)),
            "-" => Some(self.subtract(args)),
            "*" => Some(self.multiply(args)),
            "/" => Some(self.divide(args)),
            "if" => Some(self.if_expr(args)),
            "cond" => Some(self.cond(args)),
            _ => {
                // try to find function in theories
                for theory in self.theories.values() {
                    if let Some(value) = theory.interpreter.lookup_symbol(func_name) {
                        debug!("Found function {} in theory {}", func_name, theory.name);
                        match value {
                            Value::Function(func) => {
                                // TODO: actually execute function call
                                return Some(Ok(Value::Symbol(func.name.clone())));
                            }
                            Value::Symbol(sym) => {
                                return Some(Ok(Value::Symbol(sym.clone())));
                            }
                            _ => continue,
                        }
                    }
                }
                None
            }
        }
    }

    /// builtin function implementation
    fn add(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("add requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            _ => Err("add requires integer arguments".to_string()),
        }
    }

    fn subtract(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("subtract requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            _ => Err("subtract requires integer arguments".to_string()),
        }
    }

    fn multiply(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("multiply requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            _ => Err("multiply requires integer arguments".to_string()),
        }
    }

    fn divide(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("divide requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Int(x / y))
                }
            }
            _ => Err("divide requires integer arguments".to_string()),
        }
    }

    fn if_expr(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("if requires exactly 3 arguments".to_string());
        }
        let condition = self.eval(&args[0])?;
        match condition {
            Value::Bool(true) => self.eval(&args[1]),
            Value::Bool(false) => self.eval(&args[2]),
            _ => Err("if condition must be boolean".to_string()),
        }
    }

    fn cond(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        for arg in args {
            if let LSTerm::AppTerm(func, cond_args) = arg {
                if func.to_string() == "cond" && cond_args.len() == 2 {
                    let condition = self.eval(&cond_args[0])?;
                    match condition {
                        Value::Bool(true) => return self.eval(&cond_args[1]),
                        Value::Bool(false) => continue,
                        _ => return Err("cond condition must be boolean".to_string()),
                    }
                }
            }
        }
        Err("no matching condition found in cond".to_string())
    }

    /// evaluate a match expression
    fn eval_match(
        &mut self,
        value: &Value,
        cases: &[crate::lang::model::MatchCase],
    ) -> Result<Value, String> {
        for case in cases {
            if let Some(pattern) = &case.pattern {
                if self.pattern_matches(value, pattern)? {
                    return self.eval(&case.value);
                }
            } else {
                // default case
                return self.eval(&case.value);
            }
        }
        Err("no matching pattern found".to_string())
    }

    fn pattern_matches(
        &self,
        _value: &Value,
        _pattern: &crate::lang::model::Pattern,
    ) -> Result<bool, String> {
        // TODO: implement pattern matching
        unimplemented!()
    }

    fn eval_mu(&mut self, _binder: &str, _body: &Box<LSTerm>) -> Result<Value, String> {
        // TODO: implement mu expression
        unimplemented!()
    }

    /// define a function.
    pub fn define_function(
        &mut self,
        name: String,
        params: Vec<String>,
        body: LSTerm,
    ) -> Result<(), String> {
        // create function closure
        let function = Function {
            name: name.clone(),
            params,
            body,
            closure: self.current_scope().clone(),
        };

        // register to current scope
        self.define_symbol(name.clone(), Value::Function(function));
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

        // Bind parameter types (simplified - we don't have parameter names)
        for (i, param_type) in func_type.get_params().iter().enumerate() {
            self.typing
                .current_scope_mut()
                .bind_variable(format!("param{}", i), param_type.clone());
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
