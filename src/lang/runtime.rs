use crate::parser::ast::LSTerm;
use derive_more::Display;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ModelSymbolTable {
    table: std::collections::HashMap<String, SymbolInterp>,
}

#[derive(Debug, Clone)]
pub enum SymbolInterp {
    Univ(crate::lang::model::Univ),
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
    #[display(
        fmt = "({})",
        "_0.iter().map(|v| format!(\"{}\", v)).collect::<Vec<_>>().join(\" \")"
    )]
    List(Vec<Value>),
    #[display(fmt = "{}", _0)]
    Symbol(String),
    #[display(fmt = "<constructor:{}>", name)]
    Constructor {
        name: String,
        datatype: String,
        arity: usize,
    },
    #[display(fmt = "{}", constructor)]
    DataValue {
        constructor: String,
        datatype: String,
        fields: Vec<Value>,
    },
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
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }
}
