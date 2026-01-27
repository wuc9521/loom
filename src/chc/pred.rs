use crate::chc::ast::{HornAtom, HornRule, HornTerm};
use sygus_parser::SortedVar;
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
    base: String,
    polarity: Polarity,
}

#[derive(Clone, Debug)]
pub struct PredAtom {
    pub pred: Pred,
    pub args: Vec<HornTerm>,
}

impl PredAtom {
    pub fn new(pred: Pred, args: Vec<HornTerm>) -> Self {
        Self { pred, args }
    }

    pub fn dual(&self) -> Self {
        Self {
            pred: self.pred.dual(),
            args: self.args.clone(),
        }
    }
}

impl Pred {
    pub fn new(base: String) -> Self {
        Self {
            base,
            polarity: Polarity::Pos,
        }
    }

    pub fn dual(&self) -> Self {
        Self {
            base: self.base.clone(),
            polarity: self.polarity.dual(),
        }
    }

    pub fn name(&self) -> String {
        match self.polarity {
            Polarity::Pos => self.base.clone(),
            Polarity::Neg => format!("NOT_{}", self.base),
        }
    }
}

pub type PredConj = Vec<PredAtom>;
pub type PredDnf = Vec<PredConj>;

pub fn emit_dnf_rules(
    rules: &mut Vec<HornRule>,
    vars: Vec<SortedVar>,
    head: PredAtom,
    dnf: PredDnf,
) {
    for conj in &dnf {
        let body_atoms = conj.iter().map(PredAtom::to_horn).collect();
        rules.push(HornRule {
            vars: vars.clone(),
            body_atoms,
            head: head.to_horn(),
        });
    }
}

pub fn dualize_dnf(head: PredAtom, dnf: &PredDnf) -> Option<(PredAtom, PredDnf)> {
    let neg_dnf = negate_dnf(dnf);
    if neg_dnf.is_empty() {
        None
    } else {
        Some((head.dual(), neg_dnf))
    }
}

impl PredAtom {
    fn to_horn(&self) -> HornAtom {
        HornAtom {
            pred: self.pred.name(),
            args: self.args.clone(),
        }
    }
}

fn negate_dnf(dnf: &PredDnf) -> PredDnf {
    if dnf.is_empty() {
        return vec![vec![]];
    }
    let mut clauses: Vec<Vec<PredAtom>> = Vec::new();
    for conj in dnf {
        if conj.is_empty() {
            return Vec::new();
        }
        clauses.push(conj.iter().map(|atom| atom.dual()).collect());
    }
    let mut out: PredDnf = vec![Vec::new()];
    for clause in clauses {
        let mut next = Vec::new();
        for base in &out {
            for atom in &clause {
                let mut conj = base.clone();
                conj.push(atom.clone());
                next.push(conj);
            }
        }
        out = next;
    }
    out
}
