use crate::lang::utils::{Function, Scope, Value};
use crate::lang::types::adapter;
use crate::lang::types::{Constructor, FunctionType, TypeInferer};
use crate::parser::{LSCmd, LSTerm, ThSpecCmd};
use log::{debug, error};
use sygus_parser::{Sort, SyGuSCmd};

#[derive(Debug, Clone)]
pub enum SymbolLookupResult {
    Variable(Value),
    Constructor(Constructor),
    EnumElement {
        name: String,
        enum_type: String,
    },
    ToBeSynthed {
        name: String,
        return_type: sygus_parser::Sort,
    },
    NotFound,
}

#[derive(Debug)]
pub struct TheoryInterpreter {
    pub interpreter: Runtime,
    pub name: String,
}

#[derive(Debug)]
pub struct Runtime {
    pub stack: Vec<Scope>,
    pub typing: TypeInferer,
    pub theories: std::collections::HashMap<String, TheoryInterpreter>,
    pub current_object: Option<String>, // Track current object for "self" resolution
}

impl Runtime {
    pub fn new() -> Self {
        debug!("interpreter created");
        let mut interpreter = Self {
            stack: Vec::new(),
            typing: TypeInferer::new(),
            theories: std::collections::HashMap::new(),
            current_object: None,
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

    /// get global scope as mutable reference
    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.stack.first_mut().expect("Stack should never be empty")
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

    /// process a command, including type inference.
    pub fn run(&mut self, cmd: &LSCmd) -> Result<(), String> {
        debug!("{}", cmd);
        match cmd {
            LSCmd::Echo(term) => {
                // evaluate the term
                let value = self.eval(term)?;
                println!("{}", value);
            }
            // LSCmd::DefineTheory { name, cmds } => self.load_theory(name.clone(), cmds.clone())?,
            LSCmd::As(sygus_cmd) => self.run_sygus(sygus_cmd)?,
            LSCmd::DeclareFunc(func_type) => {
                // check conflicts
                let func_name = func_type
                    .get_name()
                    .unwrap_or(&"<#unknown>".to_string())
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
                    .unwrap_or(&"<#unknown>".to_string())
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
                    .unwrap_or(&"<#unknown>".to_string())
                    .clone();
                // check conflicts
                self.typing.check_all_conflicts(&func_name, "function")?;

                self.process_recursive_function_unified(&func_type)?;
            }
            LSCmd::DefineSortEnum { name, elems } => {
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
            LSCmd::DefineClass(class_def) => {
                self.define_class(class_def.clone())?;
            }
            LSCmd::DefineObj(object_def) => {
                self.define_object(object_def.clone())?;
            }
            LSCmd::SynthFunExt(func_type) => {
                error!("SynthFunExt is not supported yet: {}", func_type);
            }
            _ => unimplemented!("unsupported command: {}", cmd),
        }
        Ok(())
    }

    /// update type environment from sygus command.
    fn run_sygus(&mut self, cmd: &SyGuSCmd) -> Result<(), String> {
        match cmd {
            SyGuSCmd::SetLogic(logic) => {
                error!("SetLogic is not supported yet: {}", logic);
            }
            SyGuSCmd::DeclareDatatype(name, dtdecl) => {
                debug!("Declaring datatype: {}", name);
                self.typing.check_all_conflicts(name, "datatype")?;

                let data_type = adapter::from_dtdecl(name, dtdecl);
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
            SyGuSCmd::DeclareDatatypes(sort_decls, dt_decls) => {
                debug!(
                    "Declaring datatypes: {} sort declarations, {} datatype declarations",
                    sort_decls.len(),
                    dt_decls.len()
                );

                // Process each datatype declaration
                for (i, (sort_decl, dt_decl)) in sort_decls.iter().zip(dt_decls.iter()).enumerate()
                {
                    let name = &sort_decl.symbol;
                    debug!("Processing datatype {}: {}", i, name);

                    self.typing.check_all_conflicts(name, "datatype")?;

                    let data_type = crate::lang::types::adapter::from_dtdecl(name, dt_decl);
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
                error!("Constraint is not supported yet: {}", term);
            }
            SyGuSCmd::SynthFun(name, _, _, _) => {
                error!("SynthFun is not supported yet: {}", name);
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
                        LSTerm::parse(pair)
                            .map_err(|e| format!("Failed to parse LSTerm: {}", e))?
                    } else {
                        return Err("No LSTerm found".to_string());
                    }
                };

                // create FunctionType
                let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
                let param_sorts: Vec<Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
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
            SyGuSCmd::CheckSynth => error!("CheckSynth is not supported yet"),
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
            if body_type != func_type.get_ret() {
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

    /// lookup a symbol in the scope stack, returning detailed information about what was found
    pub fn lookup_symbol(&self, name: &str) -> SymbolLookupResult {
        debug!("Looking up symbol: '{}'", name);
        debug!("Runtime scope stack size: {}", self.stack.len());
        debug!("Type scope stack size: {}", self.typing.stack.len());

        // first check if it is a constructor
        for (i, scope) in self.typing.stack.iter().rev().enumerate() {
            if let Some(constructor_info) = scope.get_constructor(name) {
                debug!("Found constructor '{}' in type scope {}", name, i);
                return SymbolLookupResult::Constructor(constructor_info.clone());
            }
        }

        // then check normal variables in runtime scopes
        for (i, scope) in self.stack.iter().rev().enumerate() {
            if let Some(value) = scope.lookup(name) {
                debug!("Found variable '{}' in runtime scope {}", name, i);
                return SymbolLookupResult::Variable(value);
            }
        }

        // Check if it's a synthesis function (function with placeholder body)
        for scope in &self.stack {
            if let Some(value) = scope.lookup(name) {
                // Use pattern matching to identify synthesis placeholders
                if let Value::SynthesisPlaceholder {
                    name: synth_name,
                    return_type,
                } = &value
                {
                    debug!("Found synthesis function: {}", synth_name);
                    return SymbolLookupResult::ToBeSynthed {
                        name: synth_name.clone(),
                        return_type: return_type.clone(),
                    };
                }
            }
        }

        // check in theories
        for theory in self.theories.values() {
            for (i, scope) in theory.interpreter.stack.iter().rev().enumerate() {
                if let Some(value) = scope.lookup(name) {
                    debug!("Found variable '{}' in theory scope {}", name, i);
                    return SymbolLookupResult::Variable(value);
                }
            }
        }

        // finally check enum elements
        for (i, scope) in self.typing.stack.iter().rev().enumerate() {
            for (_, enum_type) in &scope.enums {
                if enum_type.elements.contains(&name.to_string()) {
                    debug!("Found enum element '{}' in type scope {}", name, i);
                    return SymbolLookupResult::EnumElement {
                        name: name.to_string(),
                        enum_type: enum_type.name.clone(),
                    };
                }
            }
        }

        debug!("Symbol '{}' not found in any scope", name);
        SymbolLookupResult::NotFound
    }

    /// lookup a symbol and return only the value (for backward compatibility)
    pub fn lookup_symbol_value(&self, name: &str) -> Option<Value> {
        // directly lookup runtime variables, avoid recursive call of lookup_symbol
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.lookup(name) {
                return Some(value);
            }
        }
        None
    }

    /// define a symbol in the current scope
    pub fn define_symbol(&mut self, name: String, value: Value) {
        debug!(
            "Defining symbol: {} with value type: {:?}",
            name,
            std::mem::discriminant(&value)
        );
        let name_clone = name.clone();
        self.current_scope_mut().define(name, value);
        debug!("Successfully defined symbol: {}", name_clone);
    }

    /// evaluate a constructor call.
    pub fn eval_constructor_call(
        &mut self,
        info: &Constructor,
        args: &[LSTerm],
    ) -> Result<Value, String> {
        if args.len() != info.field_types().len() {
            return Err(format!(
                "Constructor {} expects {} arguments, got {}",
                info.name,
                info.field_types().len(),
                args.len()
            ));
        }

        let mut field_values = Vec::new();
        for arg in args {
            let field_value = self.interp_expr(arg)?;
            field_values.push(field_value);
        }

        let result = Value::DataValue {
            constructor: info.name.clone(),
            datatype: info.datatype.clone(),
            fields: field_values,
        };

        Ok(result)
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
            if body_type != func_type.get_ret() {
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


    /// handle theory specific command.
    pub fn handle_thspec_cmd(
        &mut self,
        cmd: &ThSpecCmd,
    ) -> Result<Value, String> {
        debug!("handle theory command: {}", cmd);
        // here we can add theory specific handling logic
        Ok(Value::Symbol("theory_command".to_string()))
    }
}
