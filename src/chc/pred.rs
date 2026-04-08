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
    positive: String,
    negative: String,
    polarity: Polarity,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

pub type PredConj = Vec<PredAtom>;
pub type PredDnf = Vec<PredConj>;

pub fn dnf_true() -> PredDnf {
    vec![vec![]]
}

pub fn dnf_false() -> PredDnf {
    Vec::new()
}

pub fn dnf_and(lhs: PredDnf, rhs: PredDnf) -> PredDnf {
    if lhs.is_empty() || rhs.is_empty() {
        return dnf_false();
    }
    if lhs.iter().any(Vec::is_empty) {
        return rhs;
    }
    if rhs.iter().any(Vec::is_empty) {
        return lhs;
    }
    let mut out = Vec::new();
    for left in &lhs {
        for right in &rhs {
            let mut conj = left.clone();
            for atom in right {
                if !conj.contains(atom) {
                    conj.push(atom.clone());
                }
            }
            out.push(conj);
        }
    }
    dedup_dnf(out)
}

pub fn dnf_or(mut lhs: PredDnf, rhs: PredDnf) -> PredDnf {
    if lhs.iter().any(Vec::is_empty) || rhs.iter().any(Vec::is_empty) {
        return dnf_true();
    }
    lhs.extend(rhs);
    dedup_dnf(lhs)
}

pub fn negate_dnf(dnf: &PredDnf) -> PredDnf {
    if dnf.is_empty() {
        return dnf_true();
    }
    let mut clauses: Vec<Vec<PredAtom>> = Vec::new();
    for conj in dnf {
        if conj.is_empty() {
            return dnf_false();
        }
        clauses.push(conj.iter().map(PredAtom::dual).collect());
    }
    let mut out = dnf_true();
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

fn dedup_dnf(dnf: PredDnf) -> PredDnf {
    let mut out: PredDnf = Vec::new();
    'outer: for conj in dnf {
        for existing in &out {
            if same_conj(existing, &conj) {
                continue 'outer;
            }
        }
        out.push(conj);
    }
    out
}

fn same_conj(lhs: &[PredAtom], rhs: &[PredAtom]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().all(|atom| rhs.contains(atom))
}
