use crate::chc::ast::{HornAtom, HornRule, HornTerm};
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
    pub arg: HornTerm,
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
            Polarity::Neg => format!("_NOT_{}", self.base),
        }
    }
}

pub type PredConj = Vec<PredAtom>;
pub type PredDnf = Vec<PredConj>;

pub fn emit_dnf_rules(rules: &mut Vec<HornRule>, head: PredAtom, dnf: PredDnf) {
    for conj in &dnf {
        let body_atoms = conj.iter().map(PredAtom::to_horn).collect();
        rules.push(HornRule {
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
            arg: self.arg.clone(),
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
