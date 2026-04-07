use crate::lang::model::{DefineUniv, Model};
use crate::lang::signature::FuncSig;
use crate::parser::ast::DelisCmd;
use crate::theory::{dispatch_extcmd, Logic, ModelCtx};
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
    pub fn handle_command(&mut self, cmd: &DelisCmd) -> Option<DelisCmd> {
        use crate::parser::ast::DelisCmd;
        use sygus_parser::SyGuSCmd;
        match cmd {
            DelisCmd::As(SyGuSCmd::SetLogic(logic_str)) => {
                if self.logic.is_some() {
                    // println!("logic is already set");
                    return None;
                }
                let logic = match logic_str.as_str() {
                    "ALL" => Logic::ALL,
                    "FOL" => Logic::FOL,
                    "PML" => Logic::PML,
                    "RE" => Logic::RE,
                    "CFE" | "CFG" | "CFL" => Logic::CFE,
                    _ => {
                        println!("unsupported logic: {}", logic_str);
                        return None;
                    }
                };
                self.logic = Some(logic);
                self.ctx = match logic {
                    Logic::FOL => Some(ModelCtx::FOL(Default::default())),
                    Logic::PML => Some(ModelCtx::PML(Default::default())),
                    Logic::RE => Some(ModelCtx::RE(Default::default())),
                    Logic::CFE | Logic::CFL => None,
                    Logic::ALL => None,
                    _ => unreachable!(),
                };
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
            DelisCmd::DefineModel(model) => {
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
            DelisCmd::ThSpec(cmd) => {
                if let (Some(logic), Some(ctx)) = (self.logic, self.ctx.as_mut()) {
                    if logic != Logic::ALL {
                        println!("dispatching extcmd: {}", cmd);
                        dispatch_extcmd(logic, ctx, &cmd);
                    }
                }
                return Some(DelisCmd::ThSpec(cmd.clone()));
            }
            _ => {
                return Some(cmd.clone());
            }
        }
    }
}
