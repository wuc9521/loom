// src/logic/model.rs

use crate::parser::ast::LSTerm;
use crate::parser::{LSParseError, Rule};
use core::str;
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::{Identifier, SortedVar};

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ UnivDef* ~ FuncInterp* ~ ")" }
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
    fmt = "(define-model {} {} {} {})",
    name,
    "univ_defs.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(\" \")",
    "func_interps.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(\" \")",
    "ext_cmds.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct Model {
    pub name: String,
    pub univ_defs: Vec<DefineUniv>,
    pub func_interps: Vec<FuncInterp>,
    pub ext_cmds: Vec<ThSpecCmd>,
    pub ext_ctx: Option<crate::theory::ModelCtx>,
}

impl Model {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            univ_defs: Vec::new(),
            func_interps: Vec::new(),
            ext_cmds: Vec::new(),
            ext_ctx: None,
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs.next().unwrap().as_str();
        let mut model = Self::new(name);
        for item_pair in inner_pairs {
            match item_pair.as_rule() {
                Rule::UnivDef => {
                    let univ_def = DefineUniv::parse(item_pair)?;
                    model.add_univ_def(univ_def);
                }
                Rule::FuncInterp => {
                    let func_interp = FuncInterp::parse(item_pair)?;
                    model.add_func_interp(func_interp);
                }
                Rule::ThSpecCmd => {
                    let th_spec_cmd = ThSpecCmd::parse(item_pair)?;
                    model.ext_cmds.push(th_spec_cmd);
                }
                _ => {
                    eprintln!(
                        "[DEBUG] Unmatched rule: {:?}, content: {}",
                        item_pair.as_rule(),
                        item_pair.as_str()
                    );
                    Err(LSParseError::UnsupportedFeature(format!(
                        "Unsupported feature in model definition: {}",
                        item_pair.as_str()
                    )))?;
                }
            }
        }
        Ok(model)
    }
    pub fn add_univ_def(&mut self, univ_def: DefineUniv) {
        self.univ_defs.push(univ_def);
    }
    pub fn add_func_interp(&mut self, func_interp: FuncInterp) {
        self.func_interps.push(func_interp);
    }
    pub fn fill_interp_table(&self, table: &mut crate::lang::runtime::ModelSymbolTable) {
        for interp in &self.func_interps {
            match interp {
                FuncInterp::SimpleInterp(name, _, _, term) => {
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
                FuncInterp::MatchInterp(name, _, _, _, _) => {
                    // 对于match表达式，我们创建一个简单的函数应用
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
#[display(
    fmt = "(define-univ {} ({}))",
    sort,
    "elems.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct DefineUniv {
    pub sort: Sort,
    pub elems: Vec<UnivElem>,
}

// UnivDef = { "(" ~ "define-univ" ~ Sort ~ "(" ~ UnivElem* ~ ")" ~ ")" }
impl DefineUniv {
    pub fn new(sort: Sort, elems: Vec<UnivElem>) -> Self {
        Self { sort, elems }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        let sort = Sort::from_str(inner_pairs.next().unwrap().as_str()).unwrap();
        let mut elems = Vec::new();
        while let Some(elem_pair) = inner_pairs.next() {
            if elem_pair.as_rule() == Rule::UnivElem {
                let elem = UnivElem::parse(elem_pair)?;
                elems.push(elem);
            } else {
                Err(LSParseError::InvalidSyntax(format!(
                    "Invalid element in universe definition: {}",
                    elem_pair.as_str()
                )))?;
            }
        }
        Ok(Self::new(sort, elems))
    }
    pub fn define_univ(name: &str, elems: Vec<UnivElem>) -> Self {
        Self {
            sort: Sort::Simple(Identifier::Symbol(name.to_string())),
            elems,
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(fmt = "{}", name)]
pub struct UnivElem {
    pub name: String,
}

impl UnivElem {
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

// FuncInterp = { "(" ~ "interp-fun" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ InterpBody ~ ")" }
#[derive(Clone, Debug, Display)]
pub enum FuncInterp {
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

// FuncInterp = { "(" ~ #LSTkInterpFun="interp-fun" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ InterpBody ~ ")" }
impl FuncInterp {
    pub fn name(&self) -> &str {
        match self {
            FuncInterp::MatchInterp(name, _, _, _, _) => name,
            FuncInterp::SimpleInterp(name, _, _, _) => name,
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            FuncInterp::MatchInterp(_, args, _, _, _) => args.len(),
            FuncInterp::SimpleInterp(_, args, _, _) => args.len(),
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
    UnivElem(UnivElem),
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
        // 检查是否是括号包围的pattern
        if pair.as_str().starts_with('(') && pair.as_str().ends_with(')') {
            let content = &pair.as_str()[1..pair.as_str().len() - 1];
            // 尝试解析为UnivElem
            if content.starts_with('$') {
                let elem = UnivElem::from_str(content)?;
                return Ok(Self::new(vec![PatternElem::UnivElem(elem)]));
            }
            // 尝试解析为Symbol
            if content.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Ok(Self::new(vec![PatternElem::Var(content.to_string())]));
            }
            // 尝试解析为wildcard
            if content == "_" {
                return Ok(Self::new(vec![PatternElem::Wildcard]));
            }
        }

        // 原有的解析逻辑
        let elements = pair
            .into_inner()
            .map(|elem| match elem.as_rule() {
                Rule::UnivElem => PatternElem::UnivElem(UnivElem::parse(elem).unwrap()),
                Rule::Symbol => PatternElem::Var(elem.as_str().to_string()),
                _ => PatternElem::Wildcard,
            })
            .collect();

        Ok(Self::new(elements))
    }
}
