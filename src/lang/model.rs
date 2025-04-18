// src/logic/model.rs

use core::str;

use crate::parser::{LSParseError, Rule};
use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::SyGuSTerm;

#[derive(Clone, Debug)]
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
        inner_pairs.next();
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

#[derive(Clone, Debug)]
pub struct UnivDef {
    pub sort: Sort,
    pub elems: Vec<UnivElem>,
}

impl UnivDef {
    pub fn new(sort: Sort, elems: Vec<UnivElem>) -> Self {
        Self { sort, elems }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        inner_pairs.next();
        let sort = Sort::from_str(inner_pairs.next().unwrap().as_str()).unwrap();
        let elems_pair = inner_pairs.next().unwrap();
        let elems = elems_pair
            .into_inner()
            .map(|elem| UnivElem::parse(elem).unwrap())
            .collect();
        Ok(Self::new(sort, elems))
    }
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct FuncInterp {
    pub name: String,
    pub mapping: Vec<InterpedCase>,
}

// FuncInterp = { "(" ~ "interp-fun" ~ Symbol ~ (SimpleInterp | MatchInterp) ~ ")" }
// SimpleInterp = { "(" ~ InterpedValue ~ ")" }  // For simple 0-ary functions
// MatchInterp = { "(" ~ "match" ~ "(" ~ MatchCase+ ~ ")" ~ ")" }
// MatchCase = { "(" ~ Pattern ~ InterpedValue ~ ")" }

impl FuncInterp {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        inner_pairs.next(); // skip the opening parenthesis
        let name = inner_pairs.next().unwrap().as_str();
        let mapping = inner_pairs
            .map(|case| InterpedCase::parse(case).unwrap())
            .collect();

        Ok(Self {
            name: name.to_string(),
            mapping,
        })
    }
}
#[derive(Clone, Debug)]
pub struct InterpedCase {
    pub pattern: Option<Pattern>,
    pub interped_value: InterpedValue,
}

#[derive(Clone, Debug)]
pub enum InterpedValue {
    UnivElem(UnivElem),
    Bool(bool),
    FuncApp(String, Vec<SyGuSTerm>),
}

impl InterpedValue {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::UnivElem => Ok(Self::UnivElem(UnivElem::parse(inner)?)),
            Rule::BoolConst => {
                let value = inner.as_str() == "true";
                Ok(Self::Bool(value))
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
impl InterpedCase {
    pub fn new(pattern: Pattern, interped_value: InterpedValue) -> Self {
        Self {
            pattern: Some(pattern),
            interped_value,
        }
    }

    // InterpedCase =    "(" ~ Pattern? ~ InterpedValue ~ ")"
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner_pairs = pair.into_inner();
        inner_pairs.next(); // skip the opening parenthesis
        let pattern = inner_pairs.next().map(|p| Pattern::parse(p).unwrap());
        let interped_value = InterpedValue::parse(inner_pairs.next().unwrap())?;
        Ok(Self {
            pattern,
            interped_value,
        })
    }
}

#[derive(Clone, Debug)]
pub enum PatternElem {
    UnivElem(UnivElem),
    Var(String),
    Wildcard,
}

#[derive(Clone, Debug)]
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
