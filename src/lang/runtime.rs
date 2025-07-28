use crate::lang::model::{DefineUniv, Model};
use crate::lang::signature::FuncSig;
use crate::parser::ast::{LSCmd, LSTerm};
use crate::theory::{dispatch_extcmd, Logic, ModelCtx};
use std::collections::HashMap;
use log::{debug, warn, error};
use derive_more::Display;

#[derive(Debug, Clone, Display)]
pub enum Symbol {
    #[display(fmt = "func[{}]", _0.name)]
    Func(FuncSig),
    #[display(fmt = "univ[{}]", _0.sort)]
    Univ(DefineUniv),
    #[display(fmt = "model[{}]", _0.name)]
    Model(Model),
}

#[derive(Debug, Default, Clone)]
pub struct Runtime {
    table: HashMap<String, Symbol>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, sym: Symbol) {
        self.table.insert(name, sym);
    }
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }
    pub fn remove(&mut self, name: &str) {
        self.table.remove(name);
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Symbol)> {
        self.table.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModelSymbolTable {
    table: std::collections::HashMap<String, SymbolInterp>,
}

#[derive(Debug, Clone)]
pub enum SymbolInterp {
    InterpFun(String, LSTerm),
    Univ(crate::lang::model::DefineUniv),
}

impl ModelSymbolTable {
    pub fn new() -> Self {
        Self {
            table: std::collections::HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, interp: SymbolInterp) {
        self.table.insert(name, interp);
    }
    pub fn get(&self, name: &str) -> Option<&SymbolInterp> {
        self.table.get(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolInterp)> {
        self.table.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, Value>,
    pub parent: Option<Box<Scope>>,
}

#[derive(Debug, Clone, Display)]
pub enum Value {
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    String(String),
    #[display(fmt = "{}", _0)]
    Function(Function),
    #[display(fmt = "{}", _0)]
    Model(crate::lang::model::Model),
    #[display(fmt = "{}", _0)]
    Term(LSTerm),
    #[display(fmt = "({})", 
        "_0.iter().map(|v| format!(\"{}\", v)).collect::<Vec<_>>().join(\" \")")]
    List(Vec<Value>),
    #[display(fmt = "{}", _0)]
    Symbol(String),
}

#[derive(Debug, Clone, Display)]
#[display(fmt = "{}", name)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: LSTerm,
    pub closure: Scope,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.bindings
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else {
            self.parent
                .as_mut()
                .map(|p| p.set(name, value))
                .unwrap_or(false)
        }
    }
}

#[derive(Debug, Default)]
pub struct Session {
    pub logic: Option<Logic>,
    pub runtime: Runtime,
    pub ctx: Option<ModelCtx>,
    pub last_model: Option<(crate::lang::model::Model, ModelCtx)>,
    pub env_stack: Vec<Scope>, // stack of scopes.
    pub current_scope: Scope,  // current scope.
}

impl Session {
    pub fn new() -> Self {
        debug!("create new Session instance");
        Self {
            logic: None,
            runtime: Runtime::new(),
            ctx: None,
            last_model: None,
            env_stack: Vec::new(),
            current_scope: Scope::new(),
        }
    }

    /// enter a new scope.
    pub fn enter_scope(&mut self) {
        debug!("enter new scope");
        let new_scope = Scope::with_parent(self.current_scope.clone());
        self.env_stack.push(self.current_scope.clone());
        self.current_scope = new_scope;
        debug!("current scope stack depth: {}", self.env_stack.len() + 1);
    }

    /// exit the current scope.
    pub fn exit_scope(&mut self) {
        debug!("exit current scope");
        if let Some(parent_scope) = self.env_stack.pop() {
            self.current_scope = parent_scope;
            debug!("current scope stack depth: {}", self.env_stack.len() + 1);
        } else {
            warn!("try to exit root scope");
        }
    }

    /// evaluate an expression.
    pub fn eval(&mut self, term: &LSTerm) -> Result<Value, String> {
        debug!("start evaluating expression: {}", term);
        match term {
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                if let Some(value) = self.current_scope.lookup(&sym) {
                    debug!("find symbol in current scope: {} = {}", sym, value);
                    Ok(value)
                } else if let Some(symbol) = self.runtime.get(&sym) {
                    debug!("find symbol in runtime: {} = {}", sym, symbol);
                    match symbol {
                        Symbol::Func(func_sig) => {
                            debug!("find function definition: {}", func_sig.name);
                            Ok(Value::Symbol(func_sig.name.clone()))
                        }
                        Symbol::Model(model) => {
                            debug!("find model definition: {}", model.name);
                            Ok(Value::Model(model.clone()))
                        }
                        Symbol::Univ(univ) => {
                            debug!("find universum definition: {}", univ.sort);
                            Ok(Value::Symbol(univ.sort.to_string()))
                        }
                    }
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
                debug!("evaluate function call: {} (parameters number: {})", func.to_string(), args.len());
                self.eval_function_call(&func.to_string(), args)
            }
            LSTerm::AttrTerm(term, _attrs) => self.eval(term),
            LSTerm::MatchTerm { term, cases } => {
                debug!("evaluate match expression (cases number: {})", cases.len());
                let value = self.eval(term)?;
                debug!("match expression matched value: {:?}", value);
                self.eval_match(&value, cases)
            }
            LSTerm::LambdaTerm { args, body } => {
                debug!("evaluate lambda expression (parameters number: {})", args.len());
                let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
                let function = Function {
                    name: "lambda".to_string(),
                    params: param_names,
                    body: *body.clone(),
                    closure: self.current_scope.clone(),
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
                debug!("evaluate mu expression: {} (parameters number: {})", name, args.len());
                self.eval_mu(name, body)
            }
            LSTerm::NuTerm {
                name,
                args,
                body: _body,
                param: _,
                ret_sort: _,
            } => {
                debug!("evaluate nu expression: {} (parameters number: {})", name, args.len());
                // TODO: implement nu expression
                Ok(Value::Symbol("nu_expr".to_string()))
            }
            LSTerm::ForallTerm { args: _, body } => {
                debug!("evaluate forall expression");
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                debug!("forall expression evaluation completed: {:?}", result);
                Ok(result)
            }
            LSTerm::ExistsTerm { args: _, body } => {
                debug!("evaluate exists expression");
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                debug!("exists expression evaluation completed: {:?}", result);
                Ok(result)
            }
            LSTerm::LetTerm { args, body } => {
                debug!("evaluate let expression (bindings number: {})", args.len());
                self.enter_scope();

                for binding in args {
                    debug!("bind variable: {} = {:?}", binding.name, binding.term);
                    let value = self.eval(&binding.term)?;
                    self.current_scope.define(binding.name.clone(), value);
                }

                // evaluate body
                let result = self.eval(body)?;
                self.exit_scope();
                debug!("let expression evaluation completed: {:?}", result);
                Ok(result)
            }
        }
    }

    /// evaluate a function call.
    fn eval_function_call(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Value, String> {
        debug!("evaluate function call: {} (parameters number: {})", func_name, args.len());
        
        // evaluate arguments
        let mut arg_values = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            debug!("evaluate parameter {}: {:?}", i + 1, arg);
            let value = self.eval(arg)?;
            arg_values.push(value);
        }
        debug!("function parameters evaluation completed: {:?}", arg_values);
        
        // find function
        if let Some(symbol) = self.runtime.get(func_name) {
            match symbol {
                Symbol::Func(func_sig) => {
                    debug!("find function definition: {}", func_sig.name);
                    // TODO: implement function call logic
                    Ok(Value::Symbol(format!("called_{}", func_sig.name)))
                }
                _ => {
                    error!("symbol is not a function: {}", func_name);
                    Err(format!("{} is not a function", func_name))
                }
            }
        } else {
            // check if it is a builtin function
            match func_name {
                "and" => {
                    let mut result = true;
                    for arg in &arg_values {
                        if let Value::Bool(b) = arg {
                            result = result && *b;
                        } else {
                            return Err("and: expected boolean arguments".to_string());
                        }
                    }
                    Ok(Value::Bool(result))
                }
                "or" => {
                    let mut result = false;
                    for arg in &arg_values {
                        if let Value::Bool(b) = arg {
                            result = result || *b;
                        } else {
                            return Err("or: expected boolean arguments".to_string());
                        }
                    }
                    Ok(Value::Bool(result))
                }
                "not" => {
                    if arg_values.len() != 1 {
                        return Err("not: expected exactly one argument".to_string());
                    }
                    if let Value::Bool(b) = arg_values[0] {
                        Ok(Value::Bool(!b))
                    } else {
                        Err("not: expected boolean argument".to_string())
                    }
                }
                "=" => {
                    if arg_values.len() != 2 {
                        return Err("=: expected exactly two arguments".to_string());
                    }
                    let val1 = &arg_values[0];
                    let val2 = &arg_values[1];
                    Ok(Value::Bool(self.values_equal(val1, val2)))
                }
                _ => {
                    error!("undefined function: {}", func_name);
                    Err(format!("Undefined function: {}", func_name))
                }
            }
        }
    }

    /// evaluate a match expression.
    fn eval_match(
        &mut self,
        value: &Value,
        cases: &[crate::lang::model::MatchCase],
    ) -> Result<Value, String> {
        debug!("start pattern matching, value: {:?}", value);
        debug!("matching branch number: {}", cases.len());
        
        for (i, case) in cases.iter().enumerate() {
            debug!("check matching branch {}: {:?}", i + 1, case);
            
            if let Some(pattern) = &case.pattern {
                debug!("branch has pattern: {:?}", pattern);
                // TODO: implement pattern matching logic
                // TODO: return the value of the first branch for now
                let result = self.eval(&case.value)?;
                debug!("pattern matching success, result: {:?}", result);
                return Ok(result);
            } else {
                debug!("branch is default branch");
                let result = self.eval(&case.value)?;
                debug!("default branch execution, result: {:?}", result);
                return Ok(result);
            }
        }
        
        error!("no matching branch");
        Err("No matching pattern found".to_string())
    }

    /// check if a value matches a pattern.
    fn _pattern_matches(
        &self,
        _value: &Value,
        _pattern: &crate::lang::model::Pattern,
    ) -> Result<bool, String> {
        debug!("check pattern matching");
        // TODO: implement pattern matching logic
        Ok(true)
    }

    /// evaluate a mu expression.
    fn eval_mu(&mut self, _binder: &str, _body: &Box<LSTerm>) -> Result<Value, String> {
        debug!("evaluate mu expression");
        // TODO: implement mu expression evaluation
        Ok(Value::Symbol("mu_expr".to_string()))
    }

    /// check if two values are equal.
    fn values_equal(&self, val1: &Value, val2: &Value) -> bool {
        debug!("compare values: {:?} == {:?}", val1, val2);
        match (val1, val2) {
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
            _ => false,
        }
    }

    pub fn handle_command(&mut self, cmd: &LSCmd) -> Option<LSCmd> {
        debug!("handle command: {}", cmd.to_string());
        use sygus_parser::SyGuSCmd;
        use LSCmd;
        match cmd {
            LSCmd::As(SyGuSCmd::SetLogic(logic_str)) => {
                if self.logic.is_some() {
                    // println!("logic is already set");
                    return None;
                }
                let logic = match logic_str.as_str() {
                    "FOL" => Logic::FOL,
                    "PML" => Logic::PML,
                    "RE" => Logic::RE,
                    _ => {
                        error!("unsupported logic: {}", logic_str);
                        return None;
                    }
                };
                self.logic = Some(logic);
                self.ctx = Some(match logic {
                    Logic::FOL => ModelCtx::FOL(Default::default()),
                    Logic::PML => ModelCtx::PML(Default::default()),
                    Logic::RE => ModelCtx::RE(Default::default()),
                    _ => unreachable!(),
                });
                return Some(cmd.clone());
            }
            LSCmd::DeclareFunc(sig) => {
                debug!("declare function: {}", sig.name);
                self.runtime
                    .insert(sig.name.clone(), Symbol::Func(sig.clone()));
                return Some(cmd.clone());
            }
            LSCmd::DefineModel(model) => {
                debug!("define model: {}", model.name);
                self.runtime
                    .insert(model.name.clone(), Symbol::Model(model.clone()));
                for univ in &model.univ_defs {
                    self.runtime
                        .insert(univ.sort.to_string(), Symbol::Univ(univ.clone()));
                }
                if let Some(ctx) = self.ctx.clone() {
                    self.last_model = Some((model.clone(), ctx));
                }
                return Some(cmd.clone());
            }
            LSCmd::ThSpec(cmd) => {
                debug!("dispatching extcmd: {}", cmd);
                dispatch_extcmd(self.logic.unwrap(), self.ctx.as_mut().unwrap(), &cmd);
                return Some(LSCmd::ThSpec(cmd.clone()));
            }
            _ => {
                return Some(cmd.clone());
            }
        }
    }
}
