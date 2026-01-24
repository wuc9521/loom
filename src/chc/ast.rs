use sygus_parser::{DTConsDecl, Sort, SortDecl, SortedVar, SyGuSTerm};

#[derive(Debug, Clone)]
pub struct Atom {
    pub pred: String,
    pub args: Vec<SyGuSTerm>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub vars: Vec<SortedVar>,
    pub body_atoms: Vec<Atom>,
    pub constraints: Vec<SyGuSTerm>,
    pub head: Atom,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub vars: Vec<SortedVar>,
    pub body_atoms: Vec<Atom>,
    pub constraints: Vec<SyGuSTerm>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    SetLogic(String),
    SetOption {
        key: String,
        value: SyGuSTerm,
    },
    DeclareRel {
        name: String,
        args: Vec<Sort>,
    },
    DeclareVar {
        name: String,
        sort: Sort,
    },
    DeclareConst {
        name: String,
        sort: Sort,
    },
    DeclareDatatypes {
        sorts: Vec<SortDecl>,
        decls: Vec<Vec<DTConsDecl>>,
    },
    GetProof,
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub decls: Vec<Decl>,
    pub rules: Vec<Rule>,
    pub queries: Vec<Query>,
    pub tail: Vec<Decl>,
}
