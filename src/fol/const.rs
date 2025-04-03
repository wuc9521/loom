use crate::lang::expr::*;

#[derive(Debug, Clone)]
pub struct Constant {
    pub expr: Term,
    pub value: i32,
}

impl Constant {
    /// constructor
    pub fn new(name: impl Into<String>, value: i32) -> Self {
        Self {
            expr: Term::Const(name.into()),
            value,
        }
    }
}
