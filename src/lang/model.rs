// src/logic/model.rs

use crate::parser::ast::{LSCmd, LSTerm};
use crate::parser::{LSParseError, Rule};
use core::str;
use derive_more::Display;

use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::Identifier;

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
    pub univ_defs: Vec<SetUniv>,
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
                    let univ_def = SetUniv::parse(item_pair)?;
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
    pub fn add_univ_def(&mut self, univ_def: SetUniv) {
        self.univ_defs.push(univ_def);
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
