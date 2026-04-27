use crate::backends::smt2::ast::{HornAtom, HornTerm};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Polarity {
    Pos,
    Neg,
}

impl Polarity {
    pub fn dual(self) -> Self {
        match self {
            Polarity::Pos => Polarity::Neg,
            Polarity::Neg => Polarity::Pos,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pred {
    positive: String,
    negative: String,
    polarity: Polarity,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PredAtom {
    pub pred: Pred,
    pub arg: HornTerm,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredFormula {
    True,
    False,
    Atom(PredAtom),
    And(Vec<PredFormula>),
    Or(Vec<PredFormula>),
}

impl PredAtom {
    pub fn new(pred: Pred, arg: HornTerm) -> Self {
        Self { pred, arg }
    }

    pub fn dual(&self) -> Self {
        Self {
            pred: self.pred.dual(),
            arg: self.arg.clone(),
        }
    }

    pub fn to_horn(&self) -> HornAtom {
        HornAtom {
            pred: self.pred.name(),
            arg: self.arg.clone(),
        }
    }
}

impl Pred {
    pub fn new(positive: String, negative: String) -> Self {
        Self {
            positive,
            negative,
            polarity: Polarity::Pos,
        }
    }

    pub fn dual(&self) -> Self {
        Self {
            positive: self.positive.clone(),
            negative: self.negative.clone(),
            polarity: self.polarity.dual(),
        }
    }

    pub fn name(&self) -> String {
        match self.polarity {
            Polarity::Pos => self.positive.clone(),
            Polarity::Neg => self.negative.clone(),
        }
    }
}

pub fn formula_true() -> PredFormula {
    PredFormula::True
}

pub fn formula_false() -> PredFormula {
    PredFormula::False
}

pub fn formula_atom(atom: PredAtom) -> PredFormula {
    PredFormula::Atom(atom)
}

pub fn formula_and(lhs: PredFormula, rhs: PredFormula) -> PredFormula {
    match (lhs, rhs) {
        (PredFormula::False, _) | (_, PredFormula::False) => PredFormula::False,
        (PredFormula::True, other) | (other, PredFormula::True) => other,
        (PredFormula::And(mut left), PredFormula::And(right)) => {
            left.extend(right);
            PredFormula::And(left)
        }
        (PredFormula::And(mut left), right) => {
            left.push(right);
            PredFormula::And(left)
        }
        (left, PredFormula::And(mut right)) => {
            let mut parts = vec![left];
            parts.append(&mut right);
            PredFormula::And(parts)
        }
        (left, right) => PredFormula::And(vec![left, right]),
    }
}

pub fn formula_or(lhs: PredFormula, rhs: PredFormula) -> PredFormula {
    match (lhs, rhs) {
        (PredFormula::True, _) | (_, PredFormula::True) => PredFormula::True,
        (PredFormula::False, other) | (other, PredFormula::False) => other,
        (PredFormula::Or(mut left), PredFormula::Or(right)) => {
            left.extend(right);
            PredFormula::Or(left)
        }
        (PredFormula::Or(mut left), right) => {
            left.push(right);
            PredFormula::Or(left)
        }
        (left, PredFormula::Or(mut right)) => {
            let mut parts = vec![left];
            parts.append(&mut right);
            PredFormula::Or(parts)
        }
        (left, right) => PredFormula::Or(vec![left, right]),
    }
}
