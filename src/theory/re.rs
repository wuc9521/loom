use crate::api::{Finite, Syntax};
use derive_more::Display;

#[derive(Debug, Syntax, Clone)]
pub enum RE {
    Empty,
    Eps,
    Sym(char),
    Union(Box<RE>, Box<RE>),
    Inter(Box<RE>, Box<RE>),
    Concat(Box<RE>, Box<RE>),
    Star(Box<RE>),
}

#[derive(Debug, Finite, Display, Clone)]
pub enum Interval<T: Finite> {
    L(T),
    R(T),
}

#[derive(Debug, Finite, Display, Clone)]
pub enum Bool {
    False,
    True,
}

// Example implementation of eval function
impl RE {
    pub fn eval(&self, _a: &Bool) -> Bool {
        Bool::False
    }
    pub fn models(&self) -> Bool {
        Bool::False
    }
}
