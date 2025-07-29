// src/logic/model.rs

use crate::parser::ast::{LSCmd, LSTerm};
use crate::parser::{LSParseError, Rule};
use core::str;
use derive_more::Display;
use log::debug;

use pest::iterators::Pair;
use sygus_parser::Identifier;
use sygus_parser::Sort;

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ SetUniv* ~ ")" }
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
    fmt = "(model {} ({}) {} {})",
    name,
    "type_params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" \")",
    "univ_defs.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(\" \")",
    "ext_cmds.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct Model {
    pub name: String,
    pub type_params: Vec<String>,
    pub univ_defs: Vec<Univ>,
    pub ext_cmds: Vec<ThSpecCmd>,
    pub ext_ctx: Option<crate::theory::ModelCtx>,
}

impl Model {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_params: Vec::new(),
            univ_defs: Vec::new(),
            ext_cmds: Vec::new(),
            ext_ctx: None,
        }
    }
    ///ModelDef = { "(" ~ #LSTkDefModel="define-model" ~ Symbol ~ "(" ~ Symbol* ~ ")" ~ (SetUniv | LSCmd)* ~ ")" }
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
                    let univ_def = Univ::parse(item_pair)?;
                    model.add_univ_def(univ_def);
                }
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
    pub fn add_univ_def(&mut self, univ_def: Univ) {
        self.univ_defs.push(univ_def);
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(fmt = "(set-univ {} {})", name, sort)]
pub struct Univ {
    pub name: String,
    pub sort: Sort,
}

// SetUniv = { "(" ~ "set-univ" ~ Sort ~ "(" ~ SortElem* ~ ")" ~ ")" }
impl Univ {
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
        debug!("Parsing MatchCase: {}", pair.as_str());

        let mut inner = pair.into_inner();

        // First element should be the pattern
        let pattern_pair = inner.next().ok_or_else(|| {
            LSParseError::InvalidSyntax("Expected pattern in match case".to_string())
        })?;

        debug!(
            "Pattern pair rule: {:?}, content: {}",
            pattern_pair.as_rule(),
            pattern_pair.as_str()
        );

        let pattern = Pattern::parse(pattern_pair)?;

        // Second element should be the value
        let value_pair = inner.next().ok_or_else(|| {
            LSParseError::InvalidSyntax("Expected value in match case".to_string())
        })?;

        let value = crate::parser::ast::LSTerm::parse(value_pair)?;

        Ok(MatchCase {
            pattern: Some(pattern),
            value,
        })
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
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, elem: PatternElem) {
        self.elements.push(elem);
    }

    /// Parse pattern from pest pair
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        debug!(
            "Parsing pattern: {} (rule: {:?})",
            pair.as_str(),
            pair.as_rule()
        );

        let mut pattern = Pattern::new();
        let pattern_str = pair.as_str().trim();
        if pattern_str == "_" {
            pattern.push(PatternElem::Wildcard);
            Ok(pattern)
        } else if pattern_str.starts_with('(') && pattern_str.ends_with(')') {
            debug!("Parsing constructor pattern: {}", pattern_str);

            // remove outer parentheses and parse inner elements
            let inner_str = &pattern_str[1..pattern_str.len() - 1];
            let tokens: Vec<&str> = inner_str.split_whitespace().collect();

            if tokens.is_empty() {
                return Err(LSParseError::InvalidSyntax(
                    "Empty constructor pattern".to_string(),
                ));
            }

            // the first is the constructor name
            pattern.push(PatternElem::Var(tokens[0].to_string()));

            // the rest are parameters
            for token in tokens.iter().skip(1) {
                if *token == "_" {
                    pattern.push(PatternElem::Wildcard);
                } else {
                    pattern.push(PatternElem::Var(token.to_string()));
                }
            }

            Ok(pattern)
        } else {
            // case 3: simple variable or constructor (like nil, x, etc.)
            pattern.push(PatternElem::Var(pattern_str.to_string()));
            Ok(pattern)
        }
    }
}
