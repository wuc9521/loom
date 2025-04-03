// src/logic/model.rs

use crate::fol::{r#const::*, func::*, pred::*};

#[derive(Debug, Clone)]
pub struct Model {
    pub pos: bool,
    // Changed from HashSet to Vec to maintain order
    pub domain: Vec<i32>,
    pub predicates: Vec<Predicate>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
}

impl Model {
    /// create a new model with positive or negative flag
    pub fn new(pos: bool) -> Self {
        Self {
            pos,
            domain: Vec::new(),
            predicates: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// set the domain size
    pub fn set_domain_size(&mut self, size: usize) {
        self.domain = (0..size as i32).collect();
    }

    /// get a predicate by name
    pub fn predicate(&self, name: &str) -> Option<&Predicate> {
        self.predicates
            .iter()
            .find(|p| p.expr.name() == name)
    }

    /// get a function by name
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions
            .iter()
            .find(|f| f.expr.name() == name)
    }

    /// get a constant by name
    pub fn constant(&self, name: &str) -> Option<&Constant> {
        self.constants
            .iter()
            .find(|c| c.expr.name() == name)
    }
}
