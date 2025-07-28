use crate::lang::pattern::{DataTypeProcessor, MatchEvaluator};
use crate::lang::runtime::{Function, Scope, Value};
use crate::lang::types::{TypeEnv, TypeInferer};
use crate::parser::ast::{parse_commands, LSCmd, LSTerm};
use crate::theory::{Logic, ModelCtx};
use log::{debug, error};

#[derive(Debug)]
pub struct Interpreter {
    pub logic: Option<Logic>,
    pub ctx: Option<ModelCtx>,
    pub last_model: Option<(crate::lang::model::Model, ModelCtx)>,
    pub stack: Vec<Scope>,
    pub typing: TypeInferer,
    pub match_evaluator: MatchEvaluator,
    pub data_type_processor: DataTypeProcessor,
}

impl Interpreter {
    pub fn new() -> Self {
        let type_env = TypeEnv::new();
        debug!("interpreter created");
        let mut interpreter = Self {
            logic: None,
            ctx: None,
            last_model: None,
            stack: Vec::new(),
            typing: TypeInferer::new(),
            match_evaluator: MatchEvaluator::new(type_env.clone()),
            data_type_processor: DataTypeProcessor::new(),
        };

        interpreter.stack.push(Scope::new()); // initialize global scope

        interpreter
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
    pub fn interp(&mut self, content: &str) -> Result<Vec<Value>, String> {
        let cmds = parse_commands(content);
        let mut results = Vec::new();
        for (i, cmd_result) in cmds.iter().enumerate() {
            match cmd_result {
                Ok(cmd) => {
                    debug!("[cmd {}] {}", i + 1, cmd);
                    self.proc(cmd)?;
                    results.push(Value::Symbol("command_executed".to_string()));
                }
                Err(e) => {
                    error!("parse error: {}", e);
                    return Err(format!("Parse error: {}", e));
                }
            }
        }
        Ok(results)
    }

    /// process a command, including type inference.
    fn proc(&mut self, cmd: &LSCmd) -> Result<(), String> {
        match cmd {
            LSCmd::As(sygus_cmd) => {
                self.as_sygus_cmd(sygus_cmd)?;
                let _ = self.handle_command(cmd);
            }
            LSCmd::DeclareFunc(func_sig) => {
                debug!("declare function: {}", func_sig.name);
                let func_type = crate::lang::types::FuncType {
                    params: func_sig.arg_sorts.clone(),
                    ret: func_sig.ret_sort.clone(),
                };
                self.typing
                    .current_scope_mut()
                    .register_function(func_sig.name.clone(), func_type);
                self.current_scope_mut()
                    .define(func_sig.name.clone(), Value::Symbol(func_sig.name.clone()));
            }
            LSCmd::DefineFuncRec(func_def) => {
                debug!("define recursive function: {}", func_def.name);
                self.process_recursive_function(
                    &func_def.name,
                    &func_def.args,
                    &func_def.ret_sort,
                    &func_def.body,
                )?;
            }
            LSCmd::DefineModel(model) => {
                debug!("defining model: {}", model.name);
                self.define_model(model.clone())?;
            }
            LSCmd::ThSpec(th_cmd) => {
                debug!("handling theory-specific command: {}", th_cmd.name);
                let _ = self.handle_theory_command(th_cmd);
            }
        }
        Ok(())
    }

    /// update type environment from sygus command.
    fn as_sygus_cmd(&mut self, cmd: &sygus_parser::SyGuSCmd) -> Result<(), String> {
        match cmd {
            sygus_parser::SyGuSCmd::DeclareDatatype(name, dtdecl) => {
                debug!("declare data type: {}", name);
                let data_type = crate::lang::types::adapter::from_dtdecl(name, dtdecl);
                self.typing
                    .current_scope_mut()
                    .register_datatype(name.clone(), data_type);
            }
            sygus_parser::SyGuSCmd::DefineSort(name, _params, sort) => {
                debug!("define sort: {}", name);
                self.typing
                    .current_scope_mut()
                    .bind_variable(name.clone(), sort.clone());
                debug!("{} := {}", name, sort);
            }
            sygus_parser::SyGuSCmd::DefineFun(name, arg_sorts, ret_sort, _body) => {
                debug!("define (non-recursive) function: {}", name);
                let param_sorts: Vec<sygus_parser::Sort> =
                    arg_sorts.iter().map(|sv| sv.sort.clone()).collect();
                let func_type = crate::lang::types::FuncType {
                    params: param_sorts,
                    ret: ret_sort.clone(),
                };
                self.typing
                    .current_scope_mut()
                    .register_function(name.clone(), func_type);
                self.typing
                    .current_scope_mut()
                    .bind_variable(name.clone(), ret_sort.clone());
                self.typing.enter_scope();
                for arg in arg_sorts {
                    debug!("    {} :: {}", arg.name, arg.sort);
                    self.typing
                        .current_scope_mut()
                        .bind_variable(arg.name.clone(), arg.sort.clone());
                }
                self.typing.exit_scope();
            }
            _ => {
                debug!("handling other sygus command types");
                // other commands are skipped for now
            }
        }
        Ok(())
    }

    /// process recursive function definition.
    fn process_recursive_function(
        &mut self,
        name: &str,
        args: &[sygus_parser::SortedVar],
        ret_sort: &sygus_parser::Sort,
        body: &LSTerm,
    ) -> Result<(), String> {
        debug!("start processing recursive function: {}", name);

        // 1. register function type (recursion)
        let param_sorts: Vec<sygus_parser::Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
        let func_type = crate::lang::types::FuncType {
            params: param_sorts,
            ret: ret_sort.clone(),
        };
        self.typing
            .current_scope_mut()
            .register_function(name.to_string(), func_type);
        debug!("function type registered");

        // 2. type check function body
        debug!("enter function type checking scope");
        self.typing.enter_scope();

        // bind parameter types
        for arg in args {
            debug!("bind function parameter: {} -> {}", arg.name, arg.sort);
            self.typing
                .current_scope_mut()
                .bind_variable(arg.name.clone(), arg.sort.clone());
        }

        // infer function body type
        let body_type = self.typing.infer(body)?;

        // check return type
        if body_type != *ret_sort {
            return Err(format!(
                "Function body type {} does not match declared return type {}",
                body_type, ret_sort
            ));
        }
        self.typing.exit_scope();
        self.define_function(
            name.to_string(),
            args.iter().map(|sv| sv.name.clone()).collect(),
            body.clone(),
        )?;
        Ok(())
    }

    /// evaluate an expression
    pub fn eval(&mut self, term: &LSTerm) -> Result<Value, String> {
        debug!("start evaluating expression: {}", term);
        match term {
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                // use unified scope lookup
                if let Some(value) = self.lookup_symbol(&sym) {
                    debug!("find symbol: {} = {}", sym, value);
                    Ok(value)
                } else {
                    error!("undefined symbol: {}", sym);
                    Err(format!("Undefined symbol: {}", sym))
                }
            }
            LSTerm::Literal(lit) => {
                debug!("evaluate literal: {:?}", lit);
                match lit {
                    sygus_parser::Literal::Numeral(n) => {
                        let value = Value::Int(*n as i64);
                        debug!("numeric literal result: {:?}", value);
                        Ok(value)
                    }
                    sygus_parser::Literal::Decimal(d) => {
                        let value = Value::String(d.to_string());
                        debug!("string literal result: {:?}", value);
                        Ok(value)
                    }
                    _ => {
                        debug!("other literal type: {:?}", lit);
                        Ok(Value::Symbol(lit.to_string()))
                    }
                }
            }
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
                debug!("evaluate match expression (cases number: {})", cases.len());
                let value = self.eval(term)?;
                debug!("match expression matched value: {:?}", value);
                self.eval_match(&value, cases)
            }
            LSTerm::LambdaTerm { args, body } => {
                debug!(
                    "evaluate lambda expression (parameters number: {})",
                    args.len()
                );
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
                args,
                body,
                param: _,
                ret_sort: _,
            } => {
                debug!(
                    "evaluate mu expression: {} (parameters number: {})",
                    name,
                    args.len()
                );
                self.eval_mu(name, body)
            }
            LSTerm::NuTerm {
                name,
                args,
                body: _body,
                param: _,
                ret_sort: _,
            } => {
                debug!(
                    "evaluate nu expression: {} (parameters number: {})",
                    name,
                    args.len()
                );
                // TODO: implement nu expression
                unimplemented!()
            }
            LSTerm::ForallTerm { args: _, body } => {
                debug!("evaluate forall expression");
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
            LSTerm::ExistsTerm { args: _, body } => {
                debug!("evaluate exists expression");
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
        debug!("start interpreting expression: {}", expr);

        // first do type checking
        let expr_type = self.typing.infer(expr)?;
        debug!("expression type: {}", expr_type);

        // then evaluate
        let result = self.eval(expr);
        debug!("expression evaluation result: {:?}", result);
        result
    }

    /// enter a new scope
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new();
        self.stack.push(new_scope);
        debug!("enter scope, current stack depth: {}", self.stack.len());
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

        // then try to find user-defined functions
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
            _ => None,
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

    /// handle a command
    pub fn handle_command(&mut self, cmd: &LSCmd) -> Option<LSCmd> {
        match cmd {
            LSCmd::DefineModel(model) => {
                debug!("handling model definition: {}", model.name);
                self.define_symbol(model.name.clone(), Value::Model(model.clone()));
                None
            }
            _ => Some(cmd.clone()),
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
            "define function: {} (parameters: {})",
            name,
            params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
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
        Ok(())
    }

    /// define a model.
    pub fn define_model(&mut self, model: crate::lang::model::Model) -> Result<(), String> {
        debug!("defining model: {}", model.name);
        self.enter_scope();
        self.define_symbol(model.name.clone(), Value::Model(model.clone()));
        for interp in &model.func_interps {
            match interp {
                crate::lang::model::FuncInterp::SimpleInterp(name, _, _, term) => {
                    debug!("redefining function {} in model scope", name);
                    let func_value = Value::Function(Function {
                        name: name.clone(),
                        params: vec![], // TODO:? 
                        body: term.clone(),
                        closure: self.current_scope().clone(),
                    });
                    self.define_symbol(name.clone(), func_value);
                }
                crate::lang::model::FuncInterp::MatchInterp(name, args, _, _, cases) => {
                    // redefine function in model scope
                    debug!("redefining function {} in model scope", name);
                    // add function definition to current scope
                    let param_names: Vec<String> =
                        args.iter().map(|(name, _)| name.clone()).collect();
                    let func_value = Value::Function(Function {
                        name: name.clone(),
                        params: param_names,
                        body: LSTerm::MatchTerm {
                            term: Box::new(LSTerm::Identifier(sygus_parser::Identifier::Symbol(
                                "_".to_string(),
                            ))),
                            cases: cases.clone(),
                        },
                        closure: self.current_scope().clone(),
                    });
                    self.define_symbol(name.clone(), func_value);
                }
            }
        }

        // handle extension commands
        for ext_cmd in &model.ext_cmds {
            let _ = self.handle_theory_command(ext_cmd);
        }

        // note: here we don't exit scope, because model scope should persist
        // we only exit scope when processing next model

        Ok(())
    }

    /// handle theory specific command.
    pub fn handle_theory_command(
        &mut self,
        cmd: &crate::lang::model::ThSpecCmd,
    ) -> Result<Value, String> {
        debug!("handle theory command: {}", cmd);
        // here we can add theory specific handling logic
        Ok(Value::Symbol("theory_command".to_string()))
    }
}
