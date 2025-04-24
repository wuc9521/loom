// src/logic/model.rs

use core::str;

use crate::parser::{LSParseError, Rule};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::SyGuSTerm;

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ UnivDef* ~ FuncInterp* ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-model {} {} {})",
    name,
    "univ_defs.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(\" \")",
    "func_interps.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct Model {
    pub name: String,
    pub univ_defs: Vec<UnivDef>,
    pub func_interps: Vec<FuncInterp>,
}

impl Model {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            univ_defs: Vec::new(),
            func_interps: Vec::new(),
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs.next().unwrap().as_str();
        let mut model = Self::new(name);
        for item_pair in inner_pairs {
            match item_pair.as_rule() {
                Rule::UnivDef => {
                    let univ_def = UnivDef::parse(item_pair)?;
                    model.add_univ_def(univ_def);
                }
                Rule::FuncInterp => {
                    let func_interp = FuncInterp::parse(item_pair)?;
                    model.add_func_interp(func_interp);
                }
                _ => {
                    Err(LSParseError::UnsupportedFeature(format!(
                        "Unsupported feature in model definition: {}",
                        item_pair.as_str()
                    )))?;
                }
            }
        }

        Ok(model)
    }
    pub fn add_univ_def(&mut self, univ_def: UnivDef) {
        self.univ_defs.push(univ_def);
    }
    pub fn add_func_interp(&mut self, func_interp: FuncInterp) {
        self.func_interps.push(func_interp);
    }
}

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-universe {} ({}))",
    sort,
    "elems.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct UnivDef {
    pub sort: Sort,
    pub elems: Vec<UnivElem>,
}

// UnivDef = { "(" ~ "define-universe" ~ Sort ~ "(" ~ UnivElem* ~ ")" ~ ")" }
impl UnivDef {
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
}

#[derive(Clone, Debug, Display)]
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
}

// FuncInterp = {  "(" ~ "interp-fun" ~ Symbol ~  "(" ~ InterpedCase+ ~ ")" ~ ")" }
#[derive(Clone, Debug, Display)]
pub enum FuncInterp {
    #[display(
        fmt = "(interp-fun {} (match ({})))",
        _0,
        "_1.iter().map(|c| format!(\"{}\", c)).collect::<Vec<_>>().join(\" \")"
    )]
    Match(String, Vec<MatchCase>),
    #[display(fmt = "(interp-fun {} {})", _0, _1)]
    Simple(String, InterpedValue),
}
// FuncInterp = { "(" ~ "interp-fun" ~ Symbol ~ (SimpleInterp | MatchInterp) ~ ")" }
// SimpleInterp = { "(" ~ InterpedValue ~ ")" }  // For simple 0-ary functions
// MatchInterp = { "(" ~ "match" ~ "(" ~ MatchCase+ ~ ")" ~ ")" }
// MatchCase = { "(" ~ Pattern ~ InterpedValue ~ ")" }

impl FuncInterp {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.clone().into_inner();
        let name = inner_pairs.next().unwrap().as_str();
        let mut cases = Vec::new();
        let interp_pair = inner_pairs.next().unwrap();
        match interp_pair.as_rule() {
            Rule::SimpleInterp => {
                let interped_value = InterpedValue::parse(interp_pair)?;
                return Ok(Self::Simple(name.to_string(), interped_value));
            }
            Rule::MatchInterp => {
                // Get all match cases directly from the inner elements of MatchInterp
                let match_cases = interp_pair.into_inner();

                for case_pair in match_cases {
                    if case_pair.as_rule() == Rule::MatchCase {
                        let match_case = MatchCase::parse(case_pair)?;
                        cases.push(match_case);
                    } else {
                        return Err(LSParseError::InvalidSyntax(format!(
                            "Invalid element in function interpretation: {}",
                            case_pair.as_str()
                        )));
                    }
                }
                return Ok(Self::Match(name.to_string(), cases));
            }
            _ => {
                return Err(LSParseError::UnsupportedFeature(format!(
                    "Unsupported feature in function interpretation: {}",
                    pair.as_str()
                )));
            }
        }
    }
}

// MatchCase = { "(" ~ Pattern ~ InterpedValue ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "({} {})",
    "pattern.as_ref().map_or_else(|| \"_\".to_string(), |p| format!(\"{}\", p))",
    interped_value
)]
pub struct MatchCase {
    pub pattern: Option<Pattern>,
    pub interped_value: InterpedValue,
}

// InterpedValue = { UnivElem   | BoolConst  | "(" ~ Identifier ~ SyGuSTerm+ ~ ")"   }

#[derive(Clone, Debug, Display)]
#[display(fmt = )]
pub enum InterpedValue {
    #[display(fmt = "{}", _0)]
    UnivElem(UnivElem),
    #[display(fmt = "{}", _0)]
    BoolConst(bool),
    #[display(
        fmt = "({} {})",
        _0,
        "_1.iter().map(|e| format!(\"{}\", e)).collect::<Vec<_>>().join(\" \")"
    )]
    FuncApp(String, Vec<SyGuSTerm>),
}

// InterpedValue = {
//   UnivElem
//   | BoolConst
//   | "(" ~ Identifier ~ SyGuSTerm+ ~ ")"
//   }
impl InterpedValue {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::UnivElem => Ok(Self::UnivElem(UnivElem::parse(inner)?)),
            Rule::BoolConst => {
                let value = inner.as_str() == "true";
                Ok(Self::BoolConst(value))
            }
            _ => {
                let mut inner_pairs = inner.into_inner();
                let func_name = inner_pairs.next().unwrap().as_str();
                let args = inner_pairs
                    .map(|arg| SyGuSTerm::from_str(arg.as_str()).unwrap())
                    .collect();

                Ok(Self::FuncApp(func_name.to_string(), args))
            }
        }
    }
}

//   InterpedCase = { "(" ~ Pattern ~ InterpedValue ~ ")" // ! same as SMT-lib
// a SINGLE case
impl MatchCase {
    pub fn new(pattern: Pattern, interped_value: InterpedValue) -> Self {
        Self {
            pattern: Some(pattern),
            interped_value,
        }
    }

    // InterpedCase =    "(" ~ Pattern? ~ InterpedValue ~ ")"
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        // inner_pairs.next(); // skip the opening parenthesis
        let pattern = inner_pairs.next().map(|p| Pattern::parse(p).unwrap());
        let interped_value = InterpedValue::parse(inner_pairs.next().unwrap())?;
        Ok(Self {
            pattern,
            interped_value,
        })
    }
}

#[derive(Clone, Debug, Display)]
pub enum PatternElem {
    #[display(fmt = "{}", _0)]
    UnivElem(UnivElem),
    #[display(fmt = "{}", _0)]
    Var(String),
    #[display(fmt = "_")]
    Wildcard,
}

#[derive(Clone, Debug, Display)]
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
