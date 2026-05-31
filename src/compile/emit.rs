use sygus_parser::{BitVec, Bool, Literal, SyGuSTerm, Term};

#[derive(Clone)]
pub enum BoolExpr {
    Const(bool),
    Dynamic(SyGuSTerm),
}

impl BoolExpr {
    pub fn and(conjuncts: Vec<Self>) -> Self {
        let mut dynamic = Vec::new();
        for conjunct in conjuncts {
            match conjunct {
                Self::Const(false) => return Self::Const(false),
                Self::Const(true) => {}
                Self::Dynamic(term) => dynamic.push(term),
            }
        }
        match dynamic.len() {
            0 => Self::Const(true),
            1 => Self::Dynamic(dynamic.remove(0)),
            _ => Self::Dynamic(Bool::and(dynamic)),
        }
    }

    pub fn or(disjuncts: Vec<Self>) -> Self {
        let mut dynamic = Vec::new();
        for disjunct in disjuncts {
            match disjunct {
                Self::Const(true) => return Self::Const(true),
                Self::Const(false) => {}
                Self::Dynamic(term) => dynamic.push(term),
            }
        }
        match dynamic.len() {
            0 => Self::Const(false),
            1 => Self::Dynamic(dynamic.remove(0)),
            _ => Self::Dynamic(Bool::or(dynamic)),
        }
    }

    pub fn not(self) -> Self {
        match self {
            Self::Const(value) => Self::Const(!value),
            Self::Dynamic(term) => Self::Dynamic(Bool::not(term)),
        }
    }
}

pub fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
}

fn bitvec_literal_from_lsb(bits: &[bool]) -> SyGuSTerm {
    let bitstring = bits
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect::<String>();
    Term::lit(Literal::BinConst(bitstring))
}

pub fn compact_bitvec(bits: Vec<BoolExpr>) -> SyGuSTerm {
    let mut chunks = Vec::new();
    let mut const_bits = Vec::new();
    for bit in bits {
        match bit {
            BoolExpr::Const(value) => const_bits.push(value),
            BoolExpr::Dynamic(term) => {
                if !const_bits.is_empty() {
                    chunks.push(bitvec_literal_from_lsb(&const_bits));
                    const_bits.clear();
                }
                if let SyGuSTerm::Literal(Literal::Bool(value)) = &term {
                    const_bits.push(*value);
                } else {
                    chunks.push(Term::ite(term, BitVec::const_(1, 1), BitVec::const_(0, 1)));
                }
            }
        }
    }
    if !const_bits.is_empty() {
        chunks.push(bitvec_literal_from_lsb(&const_bits));
    }
    let mut body = chunks
        .first()
        .cloned()
        .unwrap_or_else(|| BitVec::const_(0, 1));
    for chunk in chunks.iter().skip(1) {
        body = BitVec::concat(chunk.clone(), body);
    }
    body
}
