//! Per-bit `(_ BitVec 1)` formula IR shared by every SyGuS-BV emit path.
//!
//! A constructor's output bitvector is described as a `Vec<BoolExpr>`, one
//! `BoolExpr` (an `i1`, despite the historical name) per bit position. From
//! there:
//!
//! - `BoolExpr::compact` produces a SyGuS BV term in the standard layout
//!   (concat-of-i1 bit-vectors, with constant runs folded into `BinConst`
//!   literals). Boolean disjunctions are represented with `bvor` rather than
//!   a single `ite (or ...) #b1 #b0`.
//!   Used when the constructor's bits are closed in its arg-bits alone.
//! - `BoolExpr::resolve_closure` first eliminates intra-constructor
//!   `loom.self.<bit>` self-references via a topo-sorted let chain, then
//!   compacts. Used when the relation is in `RecurrenceMode::SelfClosure`.
//!
//! Self-references inside a `BoolExpr::Dynamic(term)` are encoded as the
//! identifier `loom.self.<bit>`; `resolve_self_refs` walks a term and
//! substitutes each such occurrence by the corresponding sibling bit's
//! already-resolved `BoolExpr`, folding `and`/`or`/`not` so constants
//! propagate.

use crate::fxmap::HashSet;
use sygus_parser::{BitVec, Identifier, Literal, SyGuSTerm, Term};

/// Identifier prefix for self-bit placeholders. The chc/compiler lowerings
/// emit `loom.self.<i>` as a `SyGuSTerm::Identifier` standing for bit `i`
/// of the constructor's own output; `resolve_closure` eliminates these.
pub const SELF_BIT_PREFIX: &str = "loom.self.";

#[derive(Clone)]
pub enum BoolExpr {
    Const(bool),
    /// A term whose SyGuS sort is always `(_ BitVec 1)`.
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
            _ => Self::Dynamic(fold_i1(dynamic, BitVec::bvand, true)),
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
            _ => Self::Dynamic(fold_i1(dynamic, BitVec::bvor, false)),
        }
    }

    pub fn not(self) -> Self {
        match self {
            Self::Const(value) => Self::Const(!value),
            Self::Dynamic(term) => Self::Dynamic(BitVec::bvnot(term)),
        }
    }

    /// Convert an SMT-LIB Boolean expression to the internal `(_ BitVec 1)`
    /// representation. This is the only Boolean-to-bit-vector boundary for
    /// formula leaves which cannot themselves be represented as an i1.
    pub fn from_bool(term: SyGuSTerm) -> Self {
        match term {
            SyGuSTerm::Literal(Literal::Bool(value)) => Self::Const(value),
            term => Self::Dynamic(to_i1_term(term)),
        }
    }

    /// Concat bits (LSB-first) into a single SyGuS BV term. Constant runs
    /// collapse to a `BinConst`; dynamic bits become i1 bit-vectors.
    pub fn compact(bits: Vec<Self>) -> SyGuSTerm {
        let mut chunks: Vec<SyGuSTerm> = Vec::new();
        let mut const_bits: Vec<bool> = Vec::new();
        for bit in bits {
            match bit {
                BoolExpr::Const(value) => const_bits.push(value),
                BoolExpr::Dynamic(term) => {
                    if let SyGuSTerm::Literal(Literal::Bool(value)) = &term {
                        const_bits.push(*value);
                    } else {
                        if !const_bits.is_empty() {
                            chunks.push(bitvec_literal_from_lsb(&const_bits));
                            const_bits.clear();
                        }
                        // `Dynamic` is always an i1; Boolean source terms
                        // cross the boundary through `from_bool` instead.
                        chunks.push(term);
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

    /// Build a constructor body that statically unrolls the LFP introduced
    /// by `RecurrenceMode::SelfClosure`. If no bit has a self-dependency
    /// this falls back to `compact`; otherwise it emits nested let-bindings
    /// in topological order so each `loom.self.<i>` placeholder is defined
    /// before any other binding (or the final body) references it.
    pub fn resolve_closure(bits: &[Self]) -> SyGuSTerm {
        let n = bits.len();
        let deps: Vec<HashSet<usize>> = bits
            .iter()
            .map(|expr| {
                let mut d = HashSet::default();
                if let BoolExpr::Dynamic(t) = expr {
                    collect_self_bit_deps(t, &mut d);
                }
                d
            })
            .collect();
        let has_self_refs = deps.iter().any(|d| !d.is_empty());
        if !has_self_refs {
            return BoolExpr::compact(bits.to_vec());
        }
        // Compute SCCs. Within an SCC of size > 1 (or a single bit with a
        // self-loop), there are cyclic dependencies. For monotone LFP
        // starting from 0, dropping the within-SCC dependencies gives
        // the correct least fixpoint: any formula `X = A OR (B AND X)`
        // has LFP `X = A`, equivalent to substituting `X` with `false`
        // inside the formula.
        let scc_of: Vec<usize> = compute_sccs(&deps);
        let deps = drop_within_scc_deps(&deps, &scc_of);
        let order = match topo_sort_bits(&deps) {
            Ok(o) => o,
            Err(msg) => {
                log::warn!("self-closure body: {msg}; falling back to runtime self-references");
                return BoolExpr::compact(bits.to_vec());
            }
        };
        // Only bits referenced by other bits need `let`-bindings, otherwise
        // the surrounding term tree clones the subexpression instead of
        // reusing it by name.
        let mut ref_count = vec![0usize; n];
        for d in &deps {
            for &j in d {
                ref_count[j] += 1;
            }
        }
        let mut resolved: Vec<BoolExpr> = vec![BoolExpr::Const(false); n];
        let mut bindings: Vec<(String, SyGuSTerm)> = Vec::new();
        for &i in &order {
            let computed = match &bits[i] {
                BoolExpr::Const(v) => BoolExpr::Const(*v),
                BoolExpr::Dynamic(t) => resolve_self_refs(t, &resolved),
            };
            match computed {
                BoolExpr::Const(_) => resolved[i] = computed,
                BoolExpr::Dynamic(term) if ref_count[i] >= 2 => {
                    let name = format!("loom.share.b{i}");
                    bindings.push((name.clone(), term));
                    resolved[i] = BoolExpr::Dynamic(Term::var(&name));
                }
                expr => resolved[i] = expr,
            }
        }
        let body = BoolExpr::compact(resolved);
        if bindings.is_empty() {
            body
        } else {
            let mut out = body;
            for (name, term) in bindings.into_iter().rev() {
                out = Term::let_binding(vec![(name, term)], out);
            }
            out
        }
    }
}

/// Encode a Boolean term as a one-bit bit-vector.
///
/// Bit-vector `bvor` is equivalent to Boolean `or` once each operand has
/// been encoded as `#b0` or `#b1`. Keeping an `or` as the condition of one
/// large `ite` prevents bit-vector simplifiers from seeing that structure:
///
/// ```text
/// (ite (or p q) #b1 #b0)
/// ```
///
/// becomes (using the SMT-LIB spelling, `bvor`):
///
/// ```text
/// (bvor (ite p #b1 #b0) (ite q #b1 #b0))
/// ```
///
/// A Boolean leaf that cannot be decomposed into i1 operations retains an
/// `ite` conversion at this single boundary.
fn to_i1_term(term: SyGuSTerm) -> SyGuSTerm {
    match term {
        SyGuSTerm::Literal(Literal::Bool(value)) => BitVec::const_(u64::from(value), 1),
        SyGuSTerm::Application(Identifier::Symbol(op), args) if op == "or" => fold_i1(
            args.into_iter().map(to_i1_term).collect(),
            BitVec::bvor,
            false,
        ),
        SyGuSTerm::Application(Identifier::Symbol(op), args) if op == "and" => fold_i1(
            args.into_iter().map(to_i1_term).collect(),
            BitVec::bvand,
            true,
        ),
        SyGuSTerm::Application(Identifier::Symbol(op), mut args)
            if op == "not" && args.len() == 1 =>
        {
            BitVec::bvnot(to_i1_term(args.remove(0)))
        }
        SyGuSTerm::Application(Identifier::Symbol(op), mut args)
            if op == "=" && args.len() == 2 =>
        {
            // Loom's semantic leaves are normally `((_ extract i i) arg)`,
            // written as a Boolean equality with `#b1` by older emitters.
            // Preserve the bit directly (or invert it for `#b0`) so the
            // generated constructor body never needs a Boolean node.
            let rhs = args.pop().expect("checked equality arity");
            let lhs = args.pop().expect("checked equality arity");
            match (lhs.as_single_bit_bv_const(), rhs.as_single_bit_bv_const()) {
                (Some(true), _) => rhs,
                (Some(false), _) => BitVec::bvnot(rhs),
                (_, Some(true)) => lhs,
                (_, Some(false)) => BitVec::bvnot(lhs),
                _ => Term::ite(
                    Term::app("=", vec![lhs, rhs]),
                    BitVec::const_(1, 1),
                    BitVec::const_(0, 1),
                ),
            }
        }
        SyGuSTerm::Application(Identifier::Symbol(op), args)
            if matches!(op.as_str(), "bvand" | "bvor") =>
        {
            // `BoolExpr::and`/`or` have already produced i1 operands.
            SyGuSTerm::Application(Identifier::Symbol(op), args)
        }
        SyGuSTerm::Application(Identifier::Symbol(op), args) if op == "bvnot" => {
            SyGuSTerm::Application(Identifier::Symbol(op), args)
        }
        term => Term::ite(term, BitVec::const_(1, 1), BitVec::const_(0, 1)),
    }
}

fn fold_i1(
    mut terms: Vec<SyGuSTerm>,
    op: fn(SyGuSTerm, SyGuSTerm) -> SyGuSTerm,
    identity: bool,
) -> SyGuSTerm {
    match terms.len() {
        0 => BitVec::const_(u64::from(identity), 1),
        1 => terms.remove(0),
        _ => {
            let first = terms.remove(0);
            terms.into_iter().fold(first, op)
        }
    }
}

pub fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
}

/// Placeholder identifier for bit `i` of the enclosing constructor's own
/// output. Resolved at emit time by `BoolExpr::resolve_closure`.
pub fn self_bit_name(bit: usize) -> String {
    format!("{SELF_BIT_PREFIX}{bit}")
}

fn bitvec_literal_from_lsb(bits: &[bool]) -> SyGuSTerm {
    let bitstring = bits
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect::<String>();
    Term::lit(Literal::BinConst(bitstring))
}

/// Resolve `loom.self.<j>` references in a SyGuSTerm by substituting
/// `resolved[j]`. Combines `and`/`or`/`not` via `BoolExpr` so constants
/// propagate; everything else becomes `BoolExpr::Dynamic`.
fn resolve_self_refs(term: &SyGuSTerm, resolved: &[BoolExpr]) -> BoolExpr {
    match term {
        SyGuSTerm::Literal(Literal::Bool(v)) => BoolExpr::Const(*v),
        SyGuSTerm::Identifier(Identifier::Symbol(name)) => {
            if let Some(idx) = name.strip_prefix(SELF_BIT_PREFIX) {
                if let Ok(i) = idx.parse::<usize>() {
                    return resolved[i].clone();
                }
            }
            BoolExpr::Dynamic(term.clone())
        }
        SyGuSTerm::Application(id, args) => {
            let op = id.get_name();
            match op {
                "bvand" => BoolExpr::and(
                    args.iter()
                        .map(|a| resolve_self_refs(a, resolved))
                        .collect(),
                ),
                "bvor" => BoolExpr::or(
                    args.iter()
                        .map(|a| resolve_self_refs(a, resolved))
                        .collect(),
                ),
                "bvnot" if args.len() == 1 => resolve_self_refs(&args[0], resolved).not(),
                _ => BoolExpr::Dynamic(term.clone()),
            }
        }
        _ => BoolExpr::Dynamic(term.clone()),
    }
}

/// Walk a SyGuSTerm and collect bit indices `j` for any `loom.self.<j>`
/// identifier appearing in it.
fn collect_self_bit_deps(term: &SyGuSTerm, out: &mut HashSet<usize>) {
    match term {
        SyGuSTerm::Identifier(id) => {
            if let Some(j) = parse_self_bit_index(id) {
                out.insert(j);
            }
        }
        SyGuSTerm::Application(_, args) => {
            for a in args {
                collect_self_bit_deps(a, out);
            }
        }
        SyGuSTerm::Annotated(inner, _) => collect_self_bit_deps(inner, out),
        SyGuSTerm::Let(bindings, body) => {
            for b in bindings {
                collect_self_bit_deps(&b.term, out);
            }
            collect_self_bit_deps(body, out);
        }
        SyGuSTerm::Forall(_, body) | SyGuSTerm::Exists(_, body) => collect_self_bit_deps(body, out),
        SyGuSTerm::Literal(_) => {}
    }
}

fn parse_self_bit_index(id: &Identifier) -> Option<usize> {
    match id {
        Identifier::Symbol(name) => name
            .strip_prefix(SELF_BIT_PREFIX)
            .and_then(|s| s.parse::<usize>().ok()),
        _ => None,
    }
}

/// Compute strongly-connected components of the dependency graph and
/// return a `scc_of` vector mapping each bit to its SCC id. SCC ids are
/// assigned in reverse topological order (Tarjan-style).
fn compute_sccs(deps: &[HashSet<usize>]) -> Vec<usize> {
    let n = deps.len();
    let mut index = 0usize;
    let mut idx = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut scc_of = vec![usize::MAX; n];
    let mut next_scc = 0usize;

    fn strongconnect(
        v: usize,
        deps: &[HashSet<usize>],
        index: &mut usize,
        idx: &mut [usize],
        lowlink: &mut [usize],
        on_stack: &mut [bool],
        stack: &mut Vec<usize>,
        scc_of: &mut [usize],
        next_scc: &mut usize,
    ) {
        idx[v] = *index;
        lowlink[v] = *index;
        *index += 1;
        stack.push(v);
        on_stack[v] = true;
        for &w in &deps[v] {
            if idx[w] == usize::MAX {
                strongconnect(
                    w, deps, index, idx, lowlink, on_stack, stack, scc_of, next_scc,
                );
                lowlink[v] = lowlink[v].min(lowlink[w]);
            } else if on_stack[w] {
                lowlink[v] = lowlink[v].min(idx[w]);
            }
        }
        if lowlink[v] == idx[v] {
            let scc_id = *next_scc;
            *next_scc += 1;
            loop {
                let w = stack.pop().unwrap();
                on_stack[w] = false;
                scc_of[w] = scc_id;
                if w == v {
                    break;
                }
            }
        }
    }

    for v in 0..n {
        if idx[v] == usize::MAX {
            strongconnect(
                v,
                deps,
                &mut index,
                &mut idx,
                &mut lowlink,
                &mut on_stack,
                &mut stack,
                &mut scc_of,
                &mut next_scc,
            );
        }
    }
    scc_of
}

/// Drop edges within the same SCC. Result is acyclic (the SCC contraction
/// is a DAG, and edges remaining are inter-SCC). Equivalent to substituting
/// each in-SCC self-reference with `false` for monotone-LFP semantics.
fn drop_within_scc_deps(deps: &[HashSet<usize>], scc_of: &[usize]) -> Vec<HashSet<usize>> {
    deps.iter()
        .enumerate()
        .map(|(i, set)| {
            let i_scc = scc_of[i];
            set.iter()
                .filter(|&&j| scc_of[j] != i_scc)
                .copied()
                .collect()
        })
        .collect()
}

/// Topological sort of bit dependencies. Returns bits in an order where
/// every dependency precedes its dependents; `Err` on cyclic graphs.
fn topo_sort_bits(deps: &[HashSet<usize>]) -> Result<Vec<usize>, String> {
    let n = deps.len();
    let mut in_deg = vec![0usize; n];
    let mut rev: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, ds) in deps.iter().enumerate() {
        for &j in ds {
            if j >= n {
                return Err(format!(
                    "self-bit dep out of range: bit {i} references {j} but only {n} bits exist"
                ));
            }
            in_deg[i] += 1;
            rev[j].push(i);
        }
    }
    let mut queue: std::collections::VecDeque<usize> = (0..n).filter(|&i| in_deg[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = queue.pop_front() {
        order.push(i);
        for &k in &rev[i] {
            in_deg[k] -= 1;
            if in_deg[k] == 0 {
                queue.push_back(k);
            }
        }
    }
    if order.len() != n {
        return Err("cyclic self-bit dependency".to_string());
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_encodes_boolean_or_with_bvor() {
        let p = Term::var("p");
        let q = Term::var("q");
        let r = Term::var("r");
        let term = Term::app("or", vec![p.clone(), q.clone(), r.clone()]);

        let actual = BoolExpr::compact(vec![BoolExpr::from_bool(term)]);
        let i1 = |condition| Term::ite(condition, BitVec::const_(1, 1), BitVec::const_(0, 1));
        let expected = BitVec::bvor(BitVec::bvor(i1(p), i1(q)), i1(r));

        assert_eq!(actual, expected);
    }

    #[test]
    fn compact_encodes_boolean_and_with_bvand() {
        let p = Term::var("p");
        let q = Term::var("q");
        let term = Term::app("and", vec![p.clone(), q.clone()]);

        let actual = BoolExpr::compact(vec![BoolExpr::from_bool(term)]);
        assert_eq!(
            actual,
            BitVec::bvand(
                Term::ite(p, BitVec::const_(1, 1), BitVec::const_(0, 1)),
                Term::ite(q, BitVec::const_(1, 1), BitVec::const_(0, 1)),
            )
        );
    }

    #[test]
    fn compact_keeps_an_i1_extract_as_an_i1_extract() {
        let bit = BitVec::extract(4, 4, Term::var("arg0"));
        assert_eq!(BoolExpr::compact(vec![BoolExpr::Dynamic(bit.clone())]), bit);
    }

    #[test]
    fn from_bool_turns_bit_test_into_the_bit() {
        let bit = BitVec::extract(4, 4, Term::var("arg0"));
        let bool_test = Term::app("=", vec![bit.clone(), BitVec::const_(1, 1)]);
        match BoolExpr::from_bool(bool_test) {
            BoolExpr::Dynamic(actual) => assert_eq!(actual, bit),
            BoolExpr::Const(_) => panic!("a dynamic bit test must remain dynamic"),
        }
    }
}
