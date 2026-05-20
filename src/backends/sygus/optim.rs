use rustc_hash::FxHashMap as HashMap;
use sygus_parser::{BitVec, BuiltinOp, Literal, SyGuSTerm, Term};

#[derive(Clone, Debug)]
pub enum BitwiseExpr<T> {
    Const(bool),
    Expr(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BitRef {
    arg_idx: usize,
    bit_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BitFormula {
    Const(bool),
    Var(BitRef),
    Not(Box<BitFormula>),
    And(Vec<BitFormula>),
    Or(Vec<BitFormula>),
}

#[derive(Clone, Debug)]
enum Chunk {
    Const(Vec<bool>),
    Extract {
        arg_idx: usize,
        low: usize,
        high: usize,
    },
}

fn normalize_formula(formula: BitFormula) -> BitFormula {
    match formula {
        BitFormula::Const(value) => BitFormula::Const(value),
        BitFormula::Var(bit) => BitFormula::Var(bit),
        BitFormula::Not(inner) => {
            let inner = normalize_formula(*inner);
            match inner {
                BitFormula::Const(value) => BitFormula::Const(!value),
                BitFormula::Not(double_inner) => *double_inner,
                _ => BitFormula::Not(Box::new(inner)),
            }
        }
        BitFormula::And(children) => {
            let mut normalized = Vec::new();
            for child in children.into_iter().map(normalize_formula) {
                match child {
                    BitFormula::Const(false) => return BitFormula::Const(false),
                    BitFormula::Const(true) => {}
                    BitFormula::And(nested) => normalized.extend(nested),
                    other => normalized.push(other),
                }
            }
            normalized.sort();
            normalized.dedup();
            match normalized.len() {
                0 => BitFormula::Const(true),
                1 => normalized.remove(0),
                _ => BitFormula::And(normalized),
            }
        }
        BitFormula::Or(children) => {
            let mut normalized = Vec::new();
            for child in children.into_iter().map(normalize_formula) {
                match child {
                    BitFormula::Const(true) => return BitFormula::Const(true),
                    BitFormula::Const(false) => {}
                    BitFormula::Or(nested) => normalized.extend(nested),
                    other => normalized.push(other),
                }
            }
            normalized.sort();
            normalized.dedup();
            match normalized.len() {
                0 => BitFormula::Const(false),
                1 => normalized.remove(0),
                _ => BitFormula::Or(normalized),
            }
        }
    }
}

fn parse_arg_var(term: &SyGuSTerm, arg_index: &HashMap<String, usize>) -> Option<usize> {
    arg_index.get(term.as_var_name()?).copied()
}

fn parse_extract_arg_bit(term: &SyGuSTerm, arg_index: &HashMap<String, usize>) -> Option<BitRef> {
    let (hi, lo, arg) = term.as_extract()?;
    if hi != lo {
        return None;
    }
    Some(BitRef {
        arg_idx: parse_arg_var(arg, arg_index)?,
        bit_idx: hi,
    })
}

fn parse_eq_as_formula(
    lhs: &SyGuSTerm,
    rhs: &SyGuSTerm,
    arg_index: &HashMap<String, usize>,
) -> Option<BitFormula> {
    if let (Some(bit_ref), Some(bit)) = (
        parse_extract_arg_bit(lhs, arg_index),
        rhs.as_single_bit_bv_const(),
    ) {
        return Some(if bit {
            BitFormula::Var(bit_ref)
        } else {
            BitFormula::Not(Box::new(BitFormula::Var(bit_ref)))
        });
    }
    if let (Some(bit_ref), Some(bit)) = (
        parse_extract_arg_bit(rhs, arg_index),
        lhs.as_single_bit_bv_const(),
    ) {
        return Some(if bit {
            BitFormula::Var(bit_ref)
        } else {
            BitFormula::Not(Box::new(BitFormula::Var(bit_ref)))
        });
    }
    None
}

fn parse_bit_formula(term: &SyGuSTerm, arg_index: &HashMap<String, usize>) -> Option<BitFormula> {
    if let Some(value) = term.as_bool_const() {
        return Some(BitFormula::Const(value));
    }
    let (op, args) = term.as_op()?;
    match op {
        BuiltinOp::Not if args.len() == 1 => Some(BitFormula::Not(Box::new(parse_bit_formula(
            &args[0], arg_index,
        )?))),
        BuiltinOp::And => Some(BitFormula::And(
            args.iter()
                .map(|arg| parse_bit_formula(arg, arg_index))
                .collect::<Option<Vec<_>>>()?,
        )),
        BuiltinOp::Or => Some(BitFormula::Or(
            args.iter()
                .map(|arg| parse_bit_formula(arg, arg_index))
                .collect::<Option<Vec<_>>>()?,
        )),
        BuiltinOp::Eq if args.len() == 2 => parse_eq_as_formula(&args[0], &args[1], arg_index),
        _ => None,
    }
}

fn bitvec_literal_from_lsb(bits: &[bool]) -> SyGuSTerm {
    let bitstring = bits
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect::<String>();
    Term::lit(Literal::BinConst(bitstring))
}

fn bitvec_literal_to_lsb(term: &SyGuSTerm) -> Option<Vec<bool>> {
    let SyGuSTerm::Literal(Literal::BinConst(bitstring)) = term else {
        return None;
    };
    bitstring
        .chars()
        .rev()
        .map(|ch| match ch {
            '0' => Some(false),
            '1' => Some(true),
            _ => None,
        })
        .collect()
}

fn uniform_bv_const(width: usize, value: bool) -> SyGuSTerm {
    if value {
        Term::bconst_ones(width)
    } else {
        Term::bconst_zeros(width)
    }
}

fn arg_term(arg_names: &[String], idx: usize) -> Option<SyGuSTerm> {
    Some(Term::var(arg_names.get(idx)?))
}

fn arg_slice(
    arg_names: &[String],
    idx: usize,
    low: usize,
    width: usize,
    total_width: usize,
) -> Option<SyGuSTerm> {
    let arg = arg_term(arg_names, idx)?;
    if low == 0 && width == total_width {
        Some(arg)
    } else {
        Some(BitVec::extract(low + width - 1, low, arg))
    }
}

fn fold_bitwise(
    terms: impl IntoIterator<Item = SyGuSTerm>,
    op: fn(SyGuSTerm, SyGuSTerm) -> SyGuSTerm,
    identity: bool,
) -> SyGuSTerm {
    let mut terms = terms.into_iter().collect::<Vec<_>>();
    match terms.len() {
        0 => BitVec::const_(identity as u64, 1),
        1 => terms.remove(0),
        _ => {
            let mut iter = terms.into_iter();
            let first = iter
                .next()
                .expect("fold_bitwise called with len >= 2 must have a first term");
            iter.fold(first, op)
        }
    }
}

fn formula_to_native_bv(
    formula: &BitFormula,
    arg_names: &[String],
    width: usize,
) -> Option<SyGuSTerm> {
    match formula {
        BitFormula::Const(value) => Some(uniform_bv_const(width, *value)),
        BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
            arg_term(arg_names, *arg_idx)
        }
        BitFormula::Not(inner) => match inner.as_ref() {
            BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
                Some(BitVec::bvnot(arg_term(arg_names, *arg_idx)?))
            }
            _ => None,
        },
        BitFormula::And(children) => {
            let args = children
                .iter()
                .map(|child| match child {
                    BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
                        arg_term(arg_names, *arg_idx)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            Some(fold_bitwise(args, BitVec::bvand, true))
        }
        BitFormula::Or(children) => {
            let args = children
                .iter()
                .map(|child| match child {
                    BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
                        arg_term(arg_names, *arg_idx)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            Some(fold_bitwise(args, BitVec::bvor, false))
        }
        _ => None,
    }
}

fn formula_to_native_bv_segment(
    formula: &BitFormula,
    arg_names: &[String],
    low: usize,
    width: usize,
    total_width: usize,
) -> Option<SyGuSTerm> {
    match formula {
        BitFormula::Const(value) => Some(uniform_bv_const(width, *value)),
        BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
            arg_slice(arg_names, *arg_idx, low, width, total_width)
        }
        BitFormula::Not(inner) => match inner.as_ref() {
            BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => Some(BitVec::bvnot(
                arg_slice(arg_names, *arg_idx, low, width, total_width)?,
            )),
            _ => None,
        },
        BitFormula::And(children) => {
            let args = children
                .iter()
                .map(|child| match child {
                    BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
                        arg_slice(arg_names, *arg_idx, low, width, total_width)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            Some(fold_bitwise(args, BitVec::bvand, true))
        }
        BitFormula::Or(children) => {
            let args = children
                .iter()
                .map(|child| match child {
                    BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == 0 => {
                        arg_slice(arg_names, *arg_idx, low, width, total_width)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            Some(fold_bitwise(args, BitVec::bvor, false))
        }
        _ => None,
    }
}

fn append_const(chunks: &mut Vec<Chunk>, value: bool) {
    match chunks.last_mut() {
        Some(Chunk::Const(bits)) => bits.push(value),
        _ => chunks.push(Chunk::Const(vec![value])),
    }
}

fn append_extract(chunks: &mut Vec<Chunk>, bit: BitRef) {
    match chunks.last_mut() {
        Some(Chunk::Extract { arg_idx, high, .. })
            if *arg_idx == bit.arg_idx && *high + 1 == bit.bit_idx =>
        {
            *high = bit.bit_idx;
        }
        _ => chunks.push(Chunk::Extract {
            arg_idx: bit.arg_idx,
            low: bit.bit_idx,
            high: bit.bit_idx,
        }),
    }
}

fn chunk_width(chunk: &Chunk) -> usize {
    match chunk {
        Chunk::Const(bits) => bits.len(),
        Chunk::Extract { low, high, .. } => high - low + 1,
    }
}

fn chunk_to_term(chunk: &Chunk, arg_names: &[String]) -> Option<SyGuSTerm> {
    match chunk {
        Chunk::Const(bits) => Some(bitvec_literal_from_lsb(bits)),
        Chunk::Extract { arg_idx, low, high } => {
            Some(BitVec::extract(*high, *low, arg_term(arg_names, *arg_idx)?))
        }
    }
}

fn concat_chunks(chunks: &[Chunk], arg_names: &[String]) -> Option<SyGuSTerm> {
    let mut terms = Vec::new();
    for chunk in chunks {
        if chunk_width(chunk) > 0 {
            terms.push(chunk_to_term(chunk, arg_names)?);
        }
    }
    let mut body = terms.first()?.clone();
    for term in terms.iter().skip(1) {
        body = BitVec::concat(term.clone(), body);
    }
    Some(body)
}

fn formulas_to_cvc5_concat(formulas: &[BitFormula], arg_names: &[String]) -> Option<SyGuSTerm> {
    let mut chunks = Vec::new();
    for formula in formulas {
        match formula {
            BitFormula::Const(value) => append_const(&mut chunks, *value),
            BitFormula::Var(bit) => append_extract(&mut chunks, *bit),
            _ => return None,
        }
    }
    concat_chunks(&chunks, arg_names)
}

fn push_concat_term(terms: &mut Vec<SyGuSTerm>, term: SyGuSTerm) {
    let Some(bits) = bitvec_literal_to_lsb(&term) else {
        terms.push(term);
        return;
    };
    match terms
        .last_mut()
        .and_then(|term| bitvec_literal_to_lsb(term))
    {
        Some(mut prev_bits) => {
            prev_bits.extend(bits);
            if let Some(last) = terms.last_mut() {
                *last = bitvec_literal_from_lsb(&prev_bits);
            } else {
                terms.push(bitvec_literal_from_lsb(&prev_bits));
            }
        }
        None => terms.push(term),
    }
}

fn terms_to_concat_lsb_first(terms: impl IntoIterator<Item = SyGuSTerm>) -> Option<SyGuSTerm> {
    let terms = terms.into_iter().fold(Vec::new(), |mut acc, term| {
        push_concat_term(&mut acc, term);
        acc
    });
    let mut body = terms.first()?.clone();
    for term in terms.iter().skip(1) {
        body = BitVec::concat(term.clone(), body);
    }
    Some(body)
}

fn formulas_to_segmented_native_bv(
    formulas: &[BitFormula],
    arg_names: &[String],
) -> Option<SyGuSTerm> {
    let mut terms = Vec::new();
    let mut start = 0;
    while start < formulas.len() {
        let first = normalize_formula(shift_formula_refs(&formulas[start], start)?);
        let mut end = start + 1;
        while end < formulas.len()
            && normalize_formula(shift_formula_refs(&formulas[end], end)?) == first
        {
            end += 1;
        }
        terms.push(formula_to_native_bv_segment(
            &first,
            arg_names,
            start,
            end - start,
            formulas.len(),
        )?);
        start = end;
    }
    terms_to_concat_lsb_first(terms)
}

fn shift_formula_refs(formula: &BitFormula, expected_idx: usize) -> Option<BitFormula> {
    match formula {
        BitFormula::Const(value) => Some(BitFormula::Const(*value)),
        BitFormula::Var(BitRef { arg_idx, bit_idx }) if *bit_idx == expected_idx => {
            Some(BitFormula::Var(BitRef {
                arg_idx: *arg_idx,
                bit_idx: 0,
            }))
        }
        BitFormula::Var(_) => None,
        BitFormula::Not(inner) => Some(BitFormula::Not(Box::new(shift_formula_refs(
            inner,
            expected_idx,
        )?))),
        BitFormula::And(children) => Some(BitFormula::And(
            children
                .iter()
                .map(|child| shift_formula_refs(child, expected_idx))
                .collect::<Option<Vec<_>>>()?,
        )),
        BitFormula::Or(children) => Some(BitFormula::Or(
            children
                .iter()
                .map(|child| shift_formula_refs(child, expected_idx))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

pub fn try_lift_native_bv(
    bits: &[BitwiseExpr<SyGuSTerm>],
    arg_names: &[String],
) -> Option<SyGuSTerm> {
    if bits.is_empty() {
        return None;
    }
    let arg_index = arg_names
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.clone(), idx))
        .collect::<HashMap<_, _>>();
    let formulas = bits
        .iter()
        .map(|expr| {
            let raw = match expr {
                BitwiseExpr::Const(value) => BitFormula::Const(*value),
                BitwiseExpr::Expr(term) => parse_bit_formula(term, &arg_index)?,
            };
            Some(normalize_formula(raw))
        })
        .collect::<Option<Vec<_>>>()?;

    let positional_formulas = formulas
        .iter()
        .enumerate()
        .map(|(idx, formula)| shift_formula_refs(formula, idx))
        .collect::<Option<Vec<_>>>();
    if let Some(positional_formulas) = positional_formulas {
        let first = positional_formulas.first()?.clone();
        if positional_formulas
            .iter()
            .skip(1)
            .all(|formula| *formula == first)
        {
            if let Some(term) = formula_to_native_bv(&first, arg_names, bits.len()) {
                return Some(term);
            }
        }
    }

    formulas_to_segmented_native_bv(&formulas, arg_names)
        .or_else(|| formulas_to_cvc5_concat(&formulas, arg_names))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bit_eq(arg: &str, idx: usize, value: bool) -> SyGuSTerm {
        Term::eq(
            BitVec::extract(idx, idx, Term::var(arg)),
            BitVec::const_(value as u64, 1),
        )
    }

    fn lift(bits: Vec<BitwiseExpr<SyGuSTerm>>, args: &[&str]) -> String {
        try_lift_native_bv(
            &bits,
            &args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
        )
        .expect("test helper expects native BV lifting to succeed")
        .to_string()
    }

    #[test]
    fn keeps_positional_binary_ops_compact() {
        let bits = (0..3)
            .map(|idx| {
                BitwiseExpr::Expr(Term::app(
                    "or",
                    vec![bit_eq("arg0", idx, true), bit_eq("arg1", idx, true)],
                ))
            })
            .collect();

        assert_eq!(lift(bits, &["arg0", "arg1"]), "(bvor arg0 arg1)");
    }

    #[test]
    fn keeps_positional_not_compact() {
        let bits = (0..3)
            .map(|idx| BitwiseExpr::Expr(bit_eq("arg0", idx, false)))
            .collect();

        assert_eq!(lift(bits, &["arg0"]), "(bvnot arg0)");
    }

    #[test]
    fn segments_mixed_positional_patterns() {
        let bits = vec![
            BitwiseExpr::Expr(bit_eq("arg0", 0, true)),
            BitwiseExpr::Expr(bit_eq("arg0", 1, false)),
            BitwiseExpr::Const(true),
        ];

        assert_eq!(
            lift(bits, &["arg0"]),
            "(concat #b1 (concat (bvnot ((_ extract 1 1) arg0)) ((_ extract 0 0) arg0)))"
        );
    }

    #[test]
    fn folds_segmented_constants_into_one_literal() {
        let bits = vec![
            BitwiseExpr::Const(false),
            BitwiseExpr::Const(false),
            BitwiseExpr::Const(false),
            BitwiseExpr::Const(true),
            BitwiseExpr::Const(true),
            BitwiseExpr::Const(false),
            BitwiseExpr::Const(false),
        ];

        assert_eq!(lift(bits, &["arg0"]), "#b0011000");
    }

    #[test]
    fn lifts_distinct_semantic_slices() {
        let sem_bits = (0..3).map(|idx| {
            BitwiseExpr::Expr(Term::app(
                "or",
                vec![bit_eq("arg0", idx, true), bit_eq("arg1", idx, true)],
            ))
        });
        let valid_bits = (3..6).map(|idx| {
            BitwiseExpr::Expr(Term::app(
                "and",
                vec![bit_eq("arg0", idx, true), bit_eq("arg1", idx, true)],
            ))
        });
        let bits = sem_bits.chain(valid_bits).collect();

        assert_eq!(
            lift(bits, &["arg0", "arg1"]),
            "(concat (bvand ((_ extract 5 3) arg0) ((_ extract 5 3) arg1)) (bvor ((_ extract 2 0) arg0) ((_ extract 2 0) arg1)))"
        );
    }

    #[test]
    fn leaves_boolean_connectives_to_fallback() {
        let bit = BitwiseExpr::Expr(Term::app(
            "and",
            vec![bit_eq("arg0", 0, true), bit_eq("arg1", 2, true)],
        ));

        assert!(
            try_lift_native_bv(&[bit], &[String::from("arg0"), String::from("arg1")]).is_none()
        );
    }

    #[test]
    fn merges_adjacent_constants_and_extracts() {
        let bits = vec![
            BitwiseExpr::Const(true),
            BitwiseExpr::Const(false),
            BitwiseExpr::Expr(bit_eq("arg0", 2, true)),
            BitwiseExpr::Expr(bit_eq("arg0", 3, true)),
            BitwiseExpr::Expr(bit_eq("arg1", 0, true)),
        ];

        assert_eq!(
            lift(bits, &["arg0", "arg1"]),
            "(concat ((_ extract 0 0) arg1) (concat ((_ extract 3 2) arg0) #b01))"
        );
    }
}
