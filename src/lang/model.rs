// src/logic/model.rs

use crate::{
    lang::runtime::{ModelSymbolTable, SymbolInterp},
    parser::{DelisParseError, Rule},
};
use core::str;
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::{Identifier, Literal, Sort, SyGuSTerm};

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ UnivDef* ~ FuncInterp* ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(
    "(define-model {} {} {})",
    name,
    univ_defs.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(" "),
    func_interps.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(" ")
)]
pub struct Model {
    pub name: String,
    pub univ_defs: Vec<DefineUniv>,
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

    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs
            .next()
            .ok_or_else(|| {
                DelisParseError::InvalidSyntax("define-model expects a model name".to_string())
            })?
            .as_str();
        let mut model = Self::new(name);
        for item_pair in inner_pairs {
            match item_pair.as_rule() {
                Rule::UnivDef => {
                    let univ_def = DefineUniv::parse(item_pair)?;
                    model.univ_defs.push(univ_def);
                }
                Rule::FuncInterp => {
                    let func_interp = FuncInterp::parse(item_pair)?;
                    model.func_interps.push(func_interp);
                }
                _ => {
                    return Err(DelisParseError::UnsupportedFeature(format!(
                        "Unsupported feature in model definition: {}",
                        item_pair.as_str()
                    )));
                }
            }
        }
        Ok(model)
    }

    pub fn fill_interp_table(&self, table: &mut ModelSymbolTable) {
        for interp in &self.func_interps {
            match interp {
                FuncInterp::SimpleInterp(name, val) => {
                    table.insert(
                        name.clone(),
                        SymbolInterp::InterpFun(name.clone(), val.clone()),
                    );
                }
                FuncInterp::MatchInterp(name, _cases) => {
                    table.insert(
                        name.clone(),
                        SymbolInterp::InterpFun(
                            name.clone(),
                            InterpedValue::FuncApp(name.clone(), vec![]),
                        ),
                    );
                }
            }
        }
        for univ in &self.univ_defs {
            table.insert(univ.sort.to_string(), SymbolInterp::Univ(univ.clone()));
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(
    "(define-univ {} ({}))",
    sort,
    elems.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ")
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
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner_pairs = pair.into_inner();
        let sort = inner_pairs
            .next()
            .ok_or_else(|| DelisParseError::InvalidSyntax("define-univ expects a sort".to_string()))
            .and_then(|p| {
                Sort::from_str(p.as_str())
                    .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))
            })?;
        let mut elems = Vec::new();
        for elem_pair in inner_pairs {
            if elem_pair.as_rule() == Rule::UnivElem {
                let elem = UnivElem::parse(elem_pair)?;
                elems.push(elem);
            } else {
                return Err(DelisParseError::InvalidSyntax(format!(
                    "Invalid element in universe definition: {}",
                    elem_pair.as_str()
                )));
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

#[derive(Clone, Debug, Display, PartialEq, Eq, Hash)]
#[display("{}", name)]
pub struct UnivElem {
    pub name: String,
}

impl UnivElem {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        Ok(Self::new(pair.as_str()))
    }
    pub fn from_str(s: &str) -> Result<Self, DelisParseError> {
        Ok(Self::new(s))
    }
}

// FuncInterp = {  "(" ~ "interp-fun" ~ Symbol ~  "(" ~ InterpedCase+ ~ ")" ~ ")" }
#[derive(Clone, Debug, Display)]
pub enum FuncInterp {
    #[display(
        "(interp-fun {} (match ({})))",
        _0,
        _1.iter().map(|c| format!("{}", c)).collect::<Vec<_>>().join(" ")
    )]
    MatchInterp(String, Vec<MatchCase>),
    #[display("(interp-fun {} ({}))", _0, _1)]
    SimpleInterp(String, InterpedValue),
}
// FuncInterp = { "(" ~ "interp-fun" ~ Symbol ~ (SimpleInterp | MatchInterp) ~ ")" }
// SimpleInterp = { "(" ~ InterpedValue ~ ")" }  // For simple 0-ary functions
// MatchInterp = { "(" ~ "match" ~ "(" ~ MatchCase+ ~ ")" ~ ")" }
// MatchCase = { "(" ~ Pattern ~ InterpedValue ~ ")" }

impl FuncInterp {
    pub fn name(&self) -> &str {
        match self {
            FuncInterp::MatchInterp(name, _) => name,
            FuncInterp::SimpleInterp(name, _) => name,
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            FuncInterp::MatchInterp(_, cases) => cases.first().map_or(0, |case| {
                case.pattern.as_ref().map_or(0, |p| p.elements.len())
            }),
            FuncInterp::SimpleInterp(_, _) => 0,
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner_pairs = pair.into_inner();
        let name = inner_pairs
            .next()
            .ok_or_else(|| {
                DelisParseError::InvalidSyntax("interp-fun expects a function name".to_string())
            })?
            .as_str()
            .to_string();
        let mut cases = Vec::new();
        let interp_pair = inner_pairs.next().ok_or_else(|| {
            DelisParseError::InvalidSyntax(
                "interp-fun expects a simple value or a match block".to_string(),
            )
        })?;
        match interp_pair.as_rule() {
            Rule::SimpleInterp => {
                let interped_value = InterpedValue::parse(interp_pair)?;
                Ok(Self::SimpleInterp(name, interped_value))
            }
            Rule::MatchInterp => {
                let match_cases = interp_pair.into_inner();

                for case_pair in match_cases {
                    if case_pair.as_rule() == Rule::MatchCase {
                        let match_case = MatchCase::parse(case_pair)?;
                        cases.push(match_case);
                    } else {
                        return Err(DelisParseError::InvalidSyntax(format!(
                            "Invalid element in function interpretation: {}",
                            case_pair.as_str()
                        )));
                    }
                }
                Ok(Self::MatchInterp(name, cases))
            }
            _ => Err(DelisParseError::UnsupportedFeature(format!(
                "Unsupported feature in function interpretation: {}",
                interp_pair.as_str()
            ))),
        }
    }
}

// MatchCase = { "(" ~ Pattern ~ InterpedValue ~ ")" }
#[derive(Clone, Debug, Display, PartialEq)]
#[display(
    "({} {})",
    pattern.as_ref().map_or_else(|| "_".to_string(), |p| format!("{}", p)),
    value
)]
pub struct MatchCase {
    pub pattern: Option<Pattern>,
    pub value: InterpedValue,
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum InterpedValue {
    #[display("{}", _0)]
    UnivElem(UnivElem), // elements from new-defined universes.

    #[display("{}", _0)]
    Literal(Literal), // elements from existing sorts.

    #[display(
        "({} {})",
        _0,
        _1.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join(" ")
    )]
    FuncApp(String, Vec<SyGuSTerm>),
}

impl InterpedValue {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut pair = pair.into_inner().next().ok_or_else(|| {
            DelisParseError::InvalidSyntax("missing interpreted value".to_string())
        })?;
        if pair.as_rule() == Rule::InterpedValue {
            pair = pair.into_inner().next().ok_or_else(|| {
                DelisParseError::InvalidSyntax("missing interpreted inner value".to_string())
            })?;
        }
        match pair.as_rule() {
            Rule::UnivElem => Ok(Self::UnivElem(UnivElem::from_str(pair.as_str())?)),
            Rule::Literal => Ok(Self::Literal(Literal::from_str(pair.as_str()))),
            _ => {
                let mut children = pair.into_inner();
                let func_name = children
                    .next()
                    .ok_or_else(|| {
                        DelisParseError::InvalidSyntax(
                            "function application expects a function symbol".to_string(),
                        )
                    })?
                    .as_str()
                    .to_string();
                let args = children
                    .map(|p| SyGuSTerm::from_str(p.as_str()))
                    .collect::<Result<_, _>>()
                    .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?;
                Ok(Self::FuncApp(func_name, args))
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
            value: interped_value,
        }
    }

    pub fn pattern(&self) -> &Option<Pattern> {
        &self.pattern
    }

    // InterpedCase =    "(" ~ Pattern? ~ InterpedValue ~ ")"
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner_pairs = pair.into_inner();
        let interped_pair = inner_pairs.next_back().ok_or_else(|| {
            DelisParseError::InvalidSyntax("match case expects an interpreted value".to_string())
        })?;
        let pattern = inner_pairs.next().map(Pattern::parse).transpose()?;
        let interped_value = InterpedValue::parse(interped_pair)?;
        Ok(Self {
            pattern,
            value: interped_value,
        })
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum PatternElem {
    #[display("{}", _0)]
    UnivElem(UnivElem),
    #[display("{}", _0)]
    Var(String),
    #[display("_")]
    Wildcard,
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(
    "({})",
    elements.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join(" ")
)]
pub struct Pattern {
    pub elements: Vec<PatternElem>,
}

impl Pattern {
    pub fn new(elements: Vec<PatternElem>) -> Self {
        Self { elements }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let elements = pair
            .into_inner()
            .map(|elem| match elem.as_rule() {
                Rule::UnivElem => UnivElem::parse(elem).map(PatternElem::UnivElem),
                Rule::Symbol => Ok(PatternElem::Var(elem.as_str().to_string())),
                _ => Ok(PatternElem::Wildcard),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::new(elements))
    }
}
