// src/logic/symbol.rs

use std::fmt::Display;

use crate::lang::expr::*;

/// S_ means the symbol of _.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    SConst(String),
    SFunc(String),
    SPred(String),
    SVar(usize),
    SITE,
    SEqual,
    SAnd,
    SOr,
    SNeg,
    SForall,
    SExists,
    STrue,
    SFalse,
    SLet,
    SIn,
}

impl Symbol {
    /// Get the name of a constant, function, or predicate symbol
    #[inline]
    pub fn as_str(&self) -> Option<&String> {
        match self {
            Symbol::SConst(ref s) | Symbol::SFunc(ref s) | Symbol::SPred(ref s) => Some(s),
            _ => None,
        }
    }

    /// Get the index of a variable symbol
    #[inline]
    pub fn idx(&self) -> Option<usize> {
        match self {
            Symbol::SVar(i) => Some(*i),
            _ => None,
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::SConst(name) | Symbol::SFunc(name) | Symbol::SPred(name) => write!(f, "{}", name),
            Symbol::SVar(id) => write!(f, "x{}", id),
            Symbol::SEqual => write!(f, "="),
            Symbol::SAnd => write!(f, "∧"),
            Symbol::SOr => write!(f, "∨"),
            Symbol::SNeg => write!(f, "¬"),
            Symbol::SForall => write!(f, "∀"),
            Symbol::SExists => write!(f, "∃"),
            Symbol::STrue => write!(f, "⊤"),
            Symbol::SFalse => write!(f, "⊥"),
            Symbol::SITE => write!(f, "ite"),
            Symbol::SLet => write!(f, "let"),
            Symbol::SIn => write!(f, "in"),
        }
    }
}
pub struct LogicSymbols {
    pub foralls: Vec<Expr>,
    pub existss: Vec<Expr>,
    pub and: Expr,
    pub or: Expr,
    pub not: Expr,
    pub r#true: Expr,
    pub r#false: Expr,
}

impl LogicSymbols {
    /// Create a new set of logic symbols with the given number of variables
    pub fn new(k: usize) -> Self {
        let mut forall_symbols = Vec::new();
        let mut exists_symbols = Vec::new();
        for i in 0..k {
            let var = Symbol::SVar(i);
            forall_symbols.push(Expr::SubExpr(vec![Symbol::SForall, var.clone()]));
            exists_symbols.push(Expr::SubExpr(vec![Symbol::SExists, var.clone()]));
        }
        Self {
            and: Expr::SubExpr(vec![Symbol::SAnd]),
            or: Expr::SubExpr(vec![Symbol::SOr]),
            not: Expr::SubExpr(vec![Symbol::SNeg]),
            r#true: Expr::SubExpr(vec![Symbol::STrue]),
            r#false: Expr::SubExpr(vec![Symbol::SFalse]),
            foralls: forall_symbols,
            existss: exists_symbols,
        }
    }
}
