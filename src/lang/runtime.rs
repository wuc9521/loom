use crate::fxmap::HashMap;
use crate::lang::model::{InterpSort, Model};
use crate::lang::signature::FuncSig;
use crate::parser::ast::LoomCmd;
use sygus_parser::SyGuSCmd;

#[derive(Debug, Clone)]
pub enum Symbol {
    Func(FuncSig),
    InterpSort(InterpSort),
    Model(Model),
}

#[derive(Debug, Default, Clone)]
pub struct Runtime {
    table: HashMap<String, Symbol>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            table: HashMap::default(),
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
    InterpSort(InterpSort),
}

impl ModelSymbolTable {
    pub fn new() -> Self {
        Self {
            table: std::collections::HashMap::default(),
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
    pub fn handle_command(&mut self, cmd: &LoomCmd) -> Option<LoomCmd> {
        match cmd {
            LoomCmd::DeclareFunc(sig) => {
                self.runtime
                    .insert(sig.name.clone(), Symbol::Func(sig.clone()));
            }
            LoomCmd::DefineModel(model) => {
                self.runtime
                    .insert(model.name.clone(), Symbol::Model(model.clone()));
                for sort_interp in &model.sort_interps {
                    self.runtime.insert(
                        sort_interp.sort.to_string(),
                        Symbol::InterpSort(sort_interp.clone()),
                    );
                }
            }
            LoomCmd::As(SyGuSCmd::SetLogic(_))
            | LoomCmd::TermTypes(_)
            | LoomCmd::DefineAspects(_)
            | LoomCmd::Semantics(_)
            | LoomCmd::DeclareAxiom(_)
            | LoomCmd::As(_) => {}
        }
        Some(cmd.clone())
    }
}
