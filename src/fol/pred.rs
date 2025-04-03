use crate::lang::{expr::*, symbol::*};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Predicate {
    pub expr: Form,
    tuples: HashSet<Vec<i32>>,
}

impl Predicate {
    /// constructor
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            expr: Form::Pred(Symbol::SPred(name.into()), vec![Term::Var(0); arity]),
            tuples: HashSet::new(),
        }
    }

    /// add a tuple to the predicate
    pub fn extend(&mut self, tuple: Vec<i32>) -> bool {
        return self.tuples.insert(tuple);
    }

    /// return true if the predicate holds for the given arguments
    pub fn holds(&self, args: &[i32]) -> bool {
        return self.tuples.contains(args);
    }
}
