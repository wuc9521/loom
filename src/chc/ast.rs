use sygus_parser::{DTConsDecl, Sort, SortDecl, SortedVar, SyGuSTerm};

pub type HornTerm = SyGuSTerm;

#[derive(Debug, Clone)]
pub struct HornAtom {
    pub pred: String,
    pub args: Vec<HornTerm>,
}

#[derive(Debug, Clone)]
pub struct HornRule {
    pub vars: Vec<SortedVar>,
    pub body_atoms: Vec<HornAtom>,
    pub head: HornAtom,
}

#[derive(Debug, Clone)]
pub struct HornQuery {
    pub vars: Vec<SortedVar>,
    pub body_atoms: Vec<HornAtom>,
}

#[derive(Debug, Clone)]
pub enum HornDecl {
    SetLogic(String),
    SetOption(String, HornTerm),
    DeclareRel(String, Vec<Sort>),
    DeclareVar(String, Sort),
    DeclareConst(String, Sort),
    DeclareDatatypes(Vec<SortDecl>, Vec<Vec<DTConsDecl>>),
    GetProof,
}

#[derive(Debug, Clone, Default)]
pub struct HornProg {
    pub decls: Vec<HornDecl>,
    pub rules: Vec<HornRule>,
    pub queries: Vec<HornQuery>,
    pub tail: Vec<HornDecl>,
}
