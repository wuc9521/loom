use crate::lang::model::{DefineUniv, Model};
use crate::lang::signature::FuncSig;
use crate::parser::ast::{LSCmd, LSTerm};
use crate::theory::{dispatch_extcmd, Logic, ModelCtx};
use std::collections::HashMap;
use derive_more::Display;

// 值类型枚举
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
    #[display(fmt = "{}", _0)]
    Symbol(String),
}

// 函数类型
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: LSTerm,
    pub closure: Scope,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "λ({}) {}", self.params.join(", "), self.body)
    }
}

// 作用域
#[derive(Debug, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, Value>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
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
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Func(FuncSig),
    Univ(DefineUniv),
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

#[derive(Debug, Default)]
pub struct Session {
    pub logic: Option<Logic>,
    pub runtime: Runtime,
    pub ctx: Option<ModelCtx>,
    pub last_model: Option<(crate::lang::model::Model, ModelCtx)>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            logic: None,
            runtime: Runtime::new(),
            ctx: None,
            last_model: None,
        }
    }
    pub fn handle_command(&mut self, cmd: &LSCmd) -> Option<LSCmd> {
        use crate::parser::ast::LSCmd;
        use sygus_parser::SyGuSCmd;
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
                        println!("unsupported logic: {}", logic_str);
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
                self.runtime
                    .insert(sig.name.clone(), Symbol::Func(sig.clone()));
                return Some(cmd.clone());
            }
            LSCmd::DefineModel(model) => {
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
                println!("dispatching extcmd: {}", cmd);
                dispatch_extcmd(self.logic.unwrap(), self.ctx.as_mut().unwrap(), &cmd);
                return Some(LSCmd::ThSpec(cmd.clone()));
            }
            _ => {
                return Some(cmd.clone());
            }
        }
    }
}
