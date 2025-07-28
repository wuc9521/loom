use crate::lang::pattern::{DataTypeProcessor, MatchEvaluator};
use crate::lang::runtime::{Function, Scope, Value};
use crate::lang::types::{TypeEnv, TypeInferer};
use crate::parser::ast::{parse_commands, LSCmd, LSTerm};
use crate::theory::{Logic, ModelCtx};
use log::{debug, error};

/// 统一解释器状态
#[derive(Debug)]
pub struct Interpreter {
    pub logic: Option<Logic>,
    pub ctx: Option<ModelCtx>,
    pub last_model: Option<(crate::lang::model::Model, ModelCtx)>,
    pub stack: Vec<Scope>, // 统一的作用域栈，管理所有符号
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
        
        // 初始化全局作用域
        interpreter.stack.push(Scope::new());
        
        interpreter
    }

    /// 获取当前作用域（栈顶）
    fn current_scope(&self) -> &Scope {
        self.stack.last().expect("Stack should never be empty")
    }

    /// 获取当前作用域的可变引用
    fn current_scope_mut(&mut self) -> &mut Scope {
        self.stack.last_mut().expect("Stack should never be empty")
    }

    /// 获取全局作用域（栈底）
    fn global_scope(&self) -> &Scope {
        &self.stack[0]
    }

    /// 获取全局作用域的可变引用
    fn global_scope_mut(&mut self) -> &mut Scope {
        &mut self.stack[0]
    }

    /// interpret a file content.
    pub fn interpret(&mut self, content: &str) -> Result<Vec<Value>, String> {
        debug!("start interpreting file content");
        debug!("file content length: {} characters", content.len());

        let commands = parse_commands(content, None); // 移除 runtime 参数
        debug!("parsed {} commands", commands.len());

        let mut results = Vec::new();

        for (i, cmd_result) in commands.iter().enumerate() {
            debug!("processing the command {}", i + 1);
            match cmd_result {
                Ok(cmd) => {
                    self.process_command(&cmd)?;
                    results.push(Value::Symbol("command_executed".to_string()));
                }
                Err(e) => {
                    error!("parse error: {}", e);
                    return Err(format!("Parse error: {}", e));
                }
            }
        }

        debug!("all commands processed, total {} commands", results.len());
        Ok(results)
    }

    /// process a command, including type inference.
    pub fn process_command(&mut self, cmd: &LSCmd) -> Result<(), String> {
        debug!("handling command: {}", cmd.to_string());
        debug!("current scope number: {}", self.stack.len());

        match cmd {
            LSCmd::As(sygus_cmd) => {
                self.update_type_env_from_sygus(sygus_cmd)?;
                debug!("sygus command handled");
            }
            LSCmd::DeclareFunc(func_sig) => {
                debug!("declaring function: {}", func_sig.name);

                // 注册函数类型
                let func_type = crate::lang::types::FuncType {
                    params: func_sig.arg_sorts.clone(),
                    ret: func_sig.ret_sort.clone(),
                };
                self.typing
                    .globenv
                    .register_function(func_sig.name.clone(), func_type);

                // 注册到全局作用域
                self.global_scope_mut().define(
                    func_sig.name.clone(), 
                    Value::Symbol(func_sig.name.clone())
                );
                debug!("function registered to global scope");
            }
            LSCmd::DefineFuncRec(func_def) => {
                debug!("defining recursive function: {}", func_def.name);
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

    /// 统一的表达式求值方法
    pub fn eval(&mut self, term: &LSTerm) -> Result<Value, String> {
        debug!("start evaluating expression: {}", term);
        match term {
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                // 使用统一的作用域查找
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
            _ => {
                debug!("unhandled term type");
                Ok(Value::Symbol("unhandled".to_string()))
            }
        }
    }

    /// 统一的作用域查找符号
    fn lookup_symbol(&self, name: &str) -> Option<Value> {
        // 在作用域栈中查找（从栈顶到栈底，即从最近到最远）
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.lookup(name) {
                return Some(value);
            }
        }
        
        None
    }

    /// 在当前作用域中定义符号
    fn define_symbol(&mut self, name: String, value: Value) {
        self.current_scope_mut().define(name, value);
    }

    /// 在全局作用域中定义符号
    fn define_global_symbol(&mut self, name: String, value: Value) {
        self.global_scope_mut().define(name, value);
    }

    // 作用域管理方法
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new();
        self.stack.push(new_scope);
        debug!("enter scope, current stack depth: {}", self.stack.len());
    }

    pub fn exit_scope(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            debug!("exit scope, current stack depth: {}", self.stack.len());
        } else {
            debug!("try to exit root scope");
        }
    }

    // 函数调用求值
    fn eval_func_app(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Value, String> {
        debug!(
            "evaluating function call: {} with {} arguments",
            func_name,
            args.len()
        );

        // 首先尝试查找内置函数
        if let Some(builtin_result) = self.eval_builtin_function(func_name, args) {
            return builtin_result;
        }

        // 然后尝试查找用户定义的函数
        if let Some(value) = self.lookup_symbol(func_name) {
            match value {
                Value::Function(func) => {
                    debug!("found function: {}", func.name);
                    // 这里需要实现函数调用逻辑
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

    // 内置函数求值
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
            _ => None,
        }
    }

    // 内置函数实现
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

    // 模式匹配求值
    fn eval_match(
        &mut self,
        _value: &Value,
        _cases: &[crate::lang::model::MatchCase],
    ) -> Result<Value, String> {
        // 简化实现
        Ok(Value::Symbol("match_result".to_string()))
    }

    // 其他方法
    fn update_type_env_from_sygus(&mut self, _cmd: &sygus_parser::SyGuSCmd) -> Result<(), String> {
        // 暂时空实现
        Ok(())
    }

    fn process_recursive_function(
        &mut self,
        name: &str,
        args: &[sygus_parser::SortedVar],
        ret_sort: &sygus_parser::Sort,
        body: &LSTerm,
    ) -> Result<(), String> {
        debug!("start processing recursive function: {}", name);

        // 1. 注册函数类型
        let param_sorts: Vec<sygus_parser::Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
        let func_type = crate::lang::types::FuncType {
            params: param_sorts,
            ret: ret_sort.clone(),
        };
        self.typing
            .globenv
            .register_function(name.to_string(), func_type);
        debug!("function type registered");

        // 2. 创建函数值
        let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
        let function = Function {
            name: name.to_string(),
            params: param_names,
            body: body.clone(),
            closure: self.current_scope().clone(),
        };

        // 3. 注册到当前作用域
        self.define_symbol(name.to_string(), Value::Function(function));
        debug!("function registered to current scope");

        Ok(())
    }

    pub fn define_model(&mut self, model: crate::lang::model::Model) -> Result<(), String> {
        debug!("defining model: {}", model.name);

        // 为模型创建新的作用域
        self.enter_scope();

        // 注册模型到全局作用域
        self.define_global_symbol(model.name.clone(), Value::Model(model.clone()));

        // 处理模型内部的函数重新定义
        for interp in &model.func_interps {
            match interp {
                crate::lang::model::FuncInterp::SimpleInterp(name, _, _, term) => {
                    debug!("redefining function {} in model scope", name);
                    let func_value = Value::Function(Function {
                        name: name.clone(),
                        params: vec![],
                        body: term.clone(),
                        closure: self.current_scope().clone(),
                    });
                    self.define_symbol(name.clone(), func_value);
                }
                crate::lang::model::FuncInterp::MatchInterp(name, args, _, _, cases) => {
                    debug!("redefining function {} in model scope", name);
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

        Ok(())
    }

    pub fn handle_theory_command(
        &mut self,
        _cmd: &crate::lang::model::ThSpecCmd,
    ) -> Result<Value, String> {
        Ok(Value::Symbol("theory_command".to_string()))
    }

    // 添加迭代器方法以兼容现有代码
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        // 返回全局作用域中的所有绑定
        self.global_scope().bindings.iter()
    }
}