use crate::parser::{LSParseError, LSTerm, Rule};
use derive_more::Display;
use log::debug;
use pest::iterators::Pair;

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

        let pattern = Pattern::parse(pattern_pair)?;

        // Second element should be the value
        let value_pair = inner.next().ok_or_else(|| {
            LSParseError::InvalidSyntax("Expected value in match case".to_string())
        })?;

        let value = LSTerm::parse(value_pair)?;

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
    fmt = "{}",
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
        let mut pattern = Pattern::new();
        let pattern_str = pair.as_str().trim();
        if pattern_str == "_" {
            pattern.push(PatternElem::Wildcard);
            Ok(pattern)
        } else if pattern_str.starts_with('(') && pattern_str.ends_with(')') {
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