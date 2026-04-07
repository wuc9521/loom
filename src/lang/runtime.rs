use crate::lang::model::{DefineUniv, Model};
use crate::lang::signature::FuncSig;
use crate::parser::ast::DelisCmd;
use std::collections::HashMap;

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
    InterpFun(String, crate::lang::model::InterpedValue),
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
    pub runtime: Runtime,
}

impl Session {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }
    pub fn handle_command(&mut self, cmd: &DelisCmd) -> Option<DelisCmd> {
        use crate::parser::ast::DelisCmd;
        use sygus_parser::SyGuSCmd;
        match cmd {
            DelisCmd::As(SyGuSCmd::SetLogic(_)) => {
                return Some(cmd.clone());
            }
            DelisCmd::DeclareFunc(sig) => {
                self.runtime
                    .insert(sig.name.clone(), Symbol::Func(sig.clone()));
                return Some(cmd.clone());
            }
            DelisCmd::DeclareTermTypes(_) => {
                return Some(cmd.clone());
            }
            DelisCmd::DefineFinite(_) | DelisCmd::DefineSemantics(_) => {
                return Some(cmd.clone());
            }
            DelisCmd::DefineModel(model) => {
                self.runtime
                    .insert(model.name.clone(), Symbol::Model(model.clone()));
                for univ in &model.univ_defs {
                    self.runtime
                        .insert(univ.sort.to_string(), Symbol::Univ(univ.clone()));
                }
                return Some(cmd.clone());
            }
            DelisCmd::ThSpec(cmd) => {
                return Some(DelisCmd::ThSpec(cmd.clone()));
            }
            _ => {
                return Some(cmd.clone());
            }
        }
    }
}
