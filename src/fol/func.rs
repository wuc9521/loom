use crate::lang::{expr::*, symbol::*};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Function {
    pub expr: Term,
    pub mappings: HashMap<Vec<i32>, i32>,
}

impl Function {
    /// Create a new function with the given name and arity
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            expr: Term::Func(Symbol::SFunc(name.into()), vec![Term::Var(0); arity]),
            mappings: HashMap::new(),
        }
    }

    /// Add a new entry to the function
    pub fn extend(&mut self, args: Vec<i32>, value: i32) -> bool {
        self.mappings.insert(args, value).is_none()
    }

    /// Apply the function to the given arguments
    pub fn apply(&self, args: &[i32]) -> Option<i32> {
        self.mappings.get(args).copied()
    }

    #[inline]
    pub fn arity(&self) -> usize {
        self.mappings.keys().next().map_or(0, |args| args.len())
    }
}
