// src/logic/model.rs

use crate::parser::ast::{LSCmd, LSTerm};
use crate::parser::{LSParseError, Rule};
use core::str;
use derive_more::Display;

use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::{Identifier, SortedVar};

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ SetUniv* ~ InterpFun* ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(fmt = "({} {})", name, args)]
pub struct ThSpecCmd {
    pub name: String,
    pub args: String, // may be refined to Vec<String>
    pub sexpr: Option<lexpr::Value>,
}

impl ThSpecCmd {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let args = inner.map(|p| p.as_str()).collect::<Vec<_>>().join(" ");
        let sexpr = lexpr::from_str(&format!("({} {})", name, args)).ok();
        Ok(Self { name, args, sexpr })
    }
}

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-model {} ({}) {} {} {})",
    name,
    "type_params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" \")",
    "univ_defs.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(\" \")",
    "func_interps.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(\" \")",
    "ext_cmds.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct Model {
    pub name: String,
    pub type_params: Vec<String>, 
    pub univ_defs: Vec<SetUniv>,
    pub func_interps: Vec<InterpFun>,
    pub ext_cmds: Vec<ThSpecCmd>,
    pub ext_ctx: Option<crate::theory::ModelCtx>,
}

impl Model {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_params: Vec::new(),
            univ_defs: Vec::new(),
            func_interps: Vec::new(),
            ext_cmds: Vec::new(),
            ext_ctx: None,
        }
    }
    ///ModelDef = { "(" ~ #LSTkDefModel="define-model" ~ Symbol ~ "(" ~ Symbol* ~ ")" ~ (SetUniv | InterpFun | LSCmd)* ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs.next().unwrap().as_str();
        let mut model = Self::new(name);
        // parse type parameters (list of symbols)
        while let Some(symbol_pair) = inner_pairs.peek() {
            if symbol_pair.as_rule() == Rule::Symbol {
                model.type_params.push(symbol_pair.as_str().to_string());
                inner_pairs.next();
            } else {
                break;
            }
        }
        for item_pair in inner_pairs {
            match item_pair.as_rule() {
                Rule::SetUniv => {
                    let univ_def = SetUniv::parse(item_pair)?;
                    model.add_univ_def(univ_def);
                }
                // Rule::InterpFun => {
                //     let func_interp = InterpFun::parse(item_pair)?;
                //     model.add_func_interp(func_interp);
                // }
                Rule::LSCmd => {
                    let cmd = crate::parser::ast::parse_cmd(item_pair)?;
                    if let LSCmd::ThSpec(th_cmd) = cmd {
                        model.ext_cmds.push(th_cmd);
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Ok(model)
    }
    pub fn add_univ_def(&mut self, univ_def: SetUniv) {
        self.univ_defs.push(univ_def);
    }
    pub fn add_func_interp(&mut self, func_interp: InterpFun) {
        self.func_interps.push(func_interp);
    }
    pub fn fill_interp_table(&self, table: &mut crate::lang::runtime::ModelSymbolTable) {
        for interp in &self.func_interps {
            match interp {
                InterpFun::SimpleInterp(name, _, _, term) => {
                    table.insert(
                        name.clone(),
                        crate::lang::runtime::SymbolInterp::InterpFun(
                            name.clone(),
                            match &term {
                                LSTerm::AppTerm(sym, _) => LSTerm::AppTerm(sym.clone(), vec![]),
                                LSTerm::AttrTerm(term, _) => LSTerm::AttrTerm(term.clone(), vec![]),
                                _ => unimplemented!(),
                            },
                        ),
                    );
                }
                InterpFun::MatchInterp(name, _, _, _, _) => {
                    // TODO:?
                    let func_app = LSTerm::AppTerm(Identifier::Symbol(name.clone()), vec![]);
                    table.insert(
                        name.clone(),
                        crate::lang::runtime::SymbolInterp::InterpFun(name.clone(), func_app),
                    );
                }
            }
        }
        for univ in &self.univ_defs {
            table.insert(
                univ.sort.to_string(),
                crate::lang::runtime::SymbolInterp::Univ(univ.clone()),
            );
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(fmt = "(set-univ {} {})", name, sort)]
pub struct SetUniv {
    pub name: String,
    pub sort: Sort,
}

// SetUniv = { "(" ~ "set-univ" ~ Sort ~ "(" ~ SortElem* ~ ")" ~ ")" }
impl SetUniv {
    pub fn new(name: String, sort: Sort) -> Self {
        Self { name, sort }
    }
    /// SetUniv = { "(" ~ #LSTkSetUniv="set-univ" ~ Symbol ~ Sort ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs.next().unwrap().as_str().to_string();
        let sort = Sort::from_str(inner_pairs.next().unwrap().as_str()).unwrap();
        Ok(Self::new(name, sort))
    }

    pub fn define_univ(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sort: Sort::Simple(Identifier::Symbol(name.to_string())),
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(fmt = "{}", name)]
pub struct EnumSortElem {
    pub name: String,
}

impl EnumSortElem {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        Ok(Self::new(pair.as_str()))
    }
    pub fn from_str(s: &str) -> Result<Self, LSParseError> {
        Ok(Self::new(s))
    }
}

// InterpFun = { "(" ~ "interp-fun" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ InterpBody ~ ")" }
#[derive(Clone, Debug, Display)]
pub enum InterpFun {
    #[display(
        fmt = "(interp-fun {} ({}) {} (match {} ({})))",
        _0,
        "_1.iter().map(|v| format!(\"({} {})\", v.0, v.1)).collect::<Vec<_>>().join(\" \")",
        _2,
        _3,
        "_4.iter().map(|c| format!(\"{}\", c)).collect::<Vec<_>>().join(\" \")"
    )]
    MatchInterp(String, Vec<(String, Sort)>, Sort, LSTerm, Vec<MatchCase>),
    #[display(
        fmt = "(interp-fun {} ({}) {} {})",
        _0,
        "_1.iter().map(|v| format!(\"({} {})\", v.0, v.1)).collect::<Vec<_>>().join(\" \")",
        _2,
        _3
    )]
    SimpleInterp(String, Vec<(String, Sort)>, Sort, LSTerm),
}

// InterpFun = { "(" ~ #LSTkInterpFun="interp-fun" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ InterpBody ~ ")" }
impl InterpFun {
    pub fn name(&self) -> &str {
        match self {
            InterpFun::MatchInterp(name, _, _, _, _) => name,
            InterpFun::SimpleInterp(name, _, _, _) => name,
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            InterpFun::MatchInterp(_, args, _, _, _) => args.len(),
            InterpFun::SimpleInterp(_, args, _, _) => args.len(),
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.clone().into_inner();
        let name = inner_pairs.next().unwrap().as_str();

        // Parse argument list
        let mut args = Vec::new();
        let args_pair = inner_pairs.next().unwrap();
        for arg_pair in args_pair.into_inner() {
            if arg_pair.as_rule() == Rule::SortedVar {
                let sorted_var = SortedVar::from_str(arg_pair.as_str()).unwrap();
                args.push((sorted_var.name.clone(), sorted_var.sort.clone()));
            }
        }

        // Parse return sort
        let ret_sort = Sort::from_str(inner_pairs.next().unwrap().as_str()).unwrap();

        // Parse body
        let body_pair = inner_pairs.next().unwrap().into_inner().next().unwrap();
        match body_pair.as_rule() {
            Rule::MatchTerm => {
                let mut match_inner = body_pair.into_inner();
                let match_term = LSTerm::parse(match_inner.next().unwrap())?;
                let mut cases = Vec::new();

                while let Some(case_pair) = match_inner.next() {
                    if case_pair.as_rule() == Rule::MatchCase {
                        let case = MatchCase::parse(case_pair)?;
                        cases.push(case);
                    }
                }
                Ok(Self::MatchInterp(
                    name.to_string(),
                    args,
                    ret_sort,
                    match_term,
                    cases,
                ))
            }
            _ => {
                let term = LSTerm::parse(body_pair)?;
                Ok(Self::SimpleInterp(name.to_string(), args, ret_sort, term))
            }
        }
    }
}

// MatchCase = { "(" ~ Pattern ~ LSTerm ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "({} {})",
    "pattern.as_ref().map_or_else(|| \"_\".to_string(), |p| format!(\"{}\", p))",
    value
)]
pub struct MatchCase {
    pub pattern: Option<Pattern>,
    pub value: LSTerm,
}

// MatchCase = { "(" ~ Pattern ~ SyGuSTerm ~ ")" }
impl MatchCase {
    pub fn new(pattern: Pattern, value: LSTerm) -> Self {
        Self {
            pattern: Some(pattern),
            value,
        }
    }
    pub fn pattern(&self) -> &Option<Pattern> {
        &self.pattern
    }

    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        // inner_pairs.next(); // skip the opening parenthesis
        let pattern = inner_pairs.next().map(|p| Pattern::parse(p).unwrap());
        let value = LSTerm::parse(inner_pairs.next().unwrap())?;
        Ok(Self { pattern, value })
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum PatternElem {
    #[display(fmt = "{}", _0)]
    SortElem(EnumSortElem),
    #[display(fmt = "{}", _0)]
    Var(String),
    #[display(fmt = "_")]
    Wildcard,
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(
    fmt = "({})",
    "elements.iter().map(|e| format!(\"{}\", e)).collect::<Vec<_>>().join(\" \")"
)]
pub struct Pattern {
    pub elements: Vec<PatternElem>,
}

impl Pattern {
    pub fn new(elements: Vec<PatternElem>) -> Self {
        Self { elements }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        // TODO: unimplemented!()
        if pair.as_str().starts_with('(') && pair.as_str().ends_with(')') {
            let content = &pair.as_str()[1..pair.as_str().len() - 1];
            // try to parse as SortElem
            if content.starts_with('$') {
                let elem = EnumSortElem::from_str(content)?;
                return Ok(Self::new(vec![PatternElem::SortElem(elem)]));
            }
            // try to parse as Symbol
            if content.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Ok(Self::new(vec![PatternElem::Var(content.to_string())]));
            }
            // try to parse as wildcard
            if content == "_" {
                return Ok(Self::new(vec![PatternElem::Wildcard]));
            }

            // handle constructor pattern, e.g. (re.sym c)
            // split content and parse each element
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() > 1 {
                let mut elements = Vec::new();
                for part in parts {
                    if part.starts_with('$') {
                        // SortElem
                        let elem = EnumSortElem::from_str(part)?;
                        elements.push(PatternElem::SortElem(elem));
                    } else if part == "_" {
                        // Wildcard
                        elements.push(PatternElem::Wildcard);
                    } else {
                        // Symbol (variable)
                        elements.push(PatternElem::Var(part.to_string()));
                    }
                }
                return Ok(Self::new(elements));
            }
        }
        let elements = pair
            .into_inner()
            .map(|elem| match elem.as_rule() {
                Rule::SortElem => PatternElem::SortElem(EnumSortElem::parse(elem).unwrap()),
                Rule::Symbol => PatternElem::Var(elem.as_str().to_string()),
                _ => PatternElem::Wildcard,
            })
            .collect();

        Ok(Self::new(elements))
    }
}
