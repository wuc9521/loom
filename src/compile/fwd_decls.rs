//! Fwd-decls emission for parent-referencing nullaries.
//!
//! Per Comon §7.6.3 the saturated push-only Horn IR, lowered to per-bit
//! BV, is *almost* a closed bottom-up tree automaton evaluator — except
//! for nullary constructors whose value depends on the surrounding tree
//! (the parent-referencing nullaries). For each such nullary `c`, the
//! body that would assign its bit pattern is a function of the entry-
//! point synth-fun's value, so `c` is determined by an LFP equation
//! `c = baseline ∨ F(entry, c)`.
//!
//! Rather than baking that LFP into the file as a Kleene chain, we emit
//! SyGuS 2.1 `fwd-decls` shape:
//!
//! - The entry-point synth-fun stays as the user declared it. Its
//!   grammar references each parent-referencing nullary as a terminal.
//! - Each parent-referencing nullary becomes a `synth-fun () Sort`
//!   forward-declaration.
//! - The original constructor body for each nullary is renamed
//!   `<name>_baseline` and kept as a constant `define-fun`.
//! - The bit-link-driven propagation for each nullary is hoisted into a
//!   unary `define-fun <name>_propagation` that takes the entry sort
//!   as its single argument, so structural / bit-circuit optimisation
//!   can compile away the per-bit extraction. The fixpoint constraint
//!   is then a single line `(= c (bvor c_baseline (c_propagation entry)))`.
//!
//! `:feature fwd-decls true` is set so a conformant SyGuS 2.1 solver
//! accepts the grammar's references to other synth-funs. The LFP
//! semantics of the fixpoint constraint is the solver's responsibility:
//! a sound backend computes each nullary by iteration from baseline
//! (bottom-up search saturation), not by SAT-search over fixpoints.

use crate::backends::smt2::ast::HornProg;
use crate::compile::bit_link::BitLinkTriple;
use crate::compile::layout::BitLayout;
use crate::compile::syntax::TermSignature;
use rustc_hash::FxHashMap as HashMap;
use std::collections::BTreeSet;
use sygus_parser::{
    BitVec, Bool, Identifier, Sort, SyGuSCmd, SyGuSFeature, SyGuSTerm,
};

/// Constructors with no args whose sort appears as an arg of some other
/// constructor — these are the candidates whose bit pattern depends on
/// the surrounding tree (parent navigation through the entry-point
/// synth-fun's value).
pub fn parent_referencing_nullaries(sig: &TermSignature) -> Vec<String> {
    let mut used_as_arg: BTreeSet<String> = BTreeSet::new();
    for cons in &sig.constructors {
        for s in &cons.arg_sorts {
            used_as_arg.insert(s.clone());
        }
    }
    sig.constructors
        .iter()
        .filter(|c| c.arg_sorts.is_empty() && used_as_arg.contains(&c.result_sort))
        .map(|c| c.name.clone())
        .collect()
}

/// Rewrite `cmds` to the fwd-decls shape. If there are no parent-
/// referencing nullaries or no bit-link triples (parent-free spec),
/// returns `cmds` unchanged. `triples` must be extracted from the
/// PRE-saturation HornProg.
pub fn apply(
    cmds: Vec<SyGuSCmd>,
    prog: &HornProg,
    triples: &[BitLinkTriple],
    layout: &BitLayout,
    sig: &TermSignature,
) -> Result<Vec<SyGuSCmd>, String> {
    let nullaries = parent_referencing_nullaries(sig);
    if nullaries.is_empty() {
        return Ok(cmds);
    }

    let synth_ret_sort = sig.start_sort().to_string();
    let sort_for: HashMap<String, String> = nullaries
        .iter()
        .map(|c| {
            let s = sig
                .constructors
                .iter()
                .find(|cc| &cc.name == c)
                .map(|cc| cc.result_sort.clone())
                .unwrap_or_default();
            (c.clone(), s)
        })
        .collect();

    // Build links per nullary: src_bit (in nullary's sort) → tgt_bit (in
    // synth-ret sort = CFG). Only triples whose tgt predicate is on the
    // synth-ret sort survive — those are the bits the propagation can
    // extract directly from `g`.
    let mut links_for: HashMap<String, Vec<(usize, usize)>> = HashMap::default();
    for t in triples {
        let Some(nullary_sort) = sort_for.get(&t.child_cons) else {
            continue;
        };
        let tgt_sort = prog.predicates.get(&t.tgt_pred).map(|p| p.sort.clone());
        if tgt_sort.as_deref() != Some(synth_ret_sort.as_str()) {
            continue;
        }
        let src_sort = prog.predicates.get(&t.src_pred).map(|p| p.sort.clone());
        if src_sort.as_deref() != Some(nullary_sort.as_str()) {
            continue;
        }
        let (Some(src_bit), Some(tgt_bit)) =
            (layout.bit_of(&t.src_pred), layout.bit_of(&t.tgt_pred))
        else {
            continue;
        };
        links_for
            .entry(t.child_cons.clone())
            .or_default()
            .push((src_bit, tgt_bit));
    }
    for ls in links_for.values_mut() {
        ls.sort_unstable();
        ls.dedup();
    }
    // Prune to nullaries that actually have at least one link — others
    // are baseline-only constants and don't need fwd-declaration.
    let active: Vec<String> = nullaries
        .iter()
        .filter(|c| !links_for.get(*c).map(Vec::is_empty).unwrap_or(true))
        .cloned()
        .collect();
    if active.is_empty() {
        return Ok(cmds);
    }
    let mut active_sorted = active.clone();
    active_sorted.sort();

    let verbose = std::env::var("DELIS_DUMP_FWDDECLS").is_ok();
    if verbose {
        eprintln!(
            "[fwd-decls] triples={}, active nullaries={:?}",
            triples.len(),
            active_sorted
        );
    }

    rewrite(cmds, &active_sorted, &sort_for, &links_for, layout)
}

fn sanitize(name: &str) -> String {
    name.replace('.', "_").replace('+', "p").replace('-', "m")
}

/// Build the modified command stream.
fn rewrite(
    cmds: Vec<SyGuSCmd>,
    active: &[String],
    sort_for: &HashMap<String, String>,
    links_for: &HashMap<String, Vec<(usize, usize)>>,
    layout: &BitLayout,
) -> Result<Vec<SyGuSCmd>, String> {
    // We need the synth-fun's name so the fixpoint constraints can
    // reference `g` directly. Pre-scan to grab it.
    let synth_name: String = cmds
        .iter()
        .find_map(|c| match c {
            SyGuSCmd::SynthFun(name, _, _, _) => Some(name.clone()),
            _ => None,
        })
        .ok_or_else(|| "fwd_decls::apply: no synth-fun in cmds".to_string())?;

    let mut out: Vec<SyGuSCmd> = Vec::with_capacity(cmds.len() + active.len() * 3 + 4);
    let mut emitted_feature = false;
    for cmd in cmds {
        match cmd {
            SyGuSCmd::SetLogic(_) if !emitted_feature => {
                out.push(cmd);
                out.push(SyGuSCmd::SetFeature(SyGuSFeature::FwdDecls, true));
                emitted_feature = true;
            }
            SyGuSCmd::DefineFun(name, args, sort, body) if active.contains(&name) => {
                let renamed = format!("{}_baseline", sanitize(&name));
                out.push(SyGuSCmd::DefineFun(renamed, args, sort, body));
            }
            SyGuSCmd::SynthFun(name, args, ret_sort, grammar) if name == synth_name => {
                let entry_ret = ret_sort.clone();
                // Emit the fwd-declared synth-funs FIRST so the entry-
                // point synth-fun's grammar can reference them as
                // terminals (per SyGuS 2.1 :fwd-decls feature).
                for c in active {
                    let s = sort_for[c].clone();
                    out.push(SyGuSCmd::SynthFun(
                        c.clone(),
                        Vec::new(),
                        Sort::from_name(&s),
                        None,
                    ));
                }
                out.push(SyGuSCmd::SynthFun(name, args, ret_sort, grammar));

                // Hoist each nullary's propagation expression into a
                // unary `define-fun <c>_propagation ((arg0 <entry-sort>))
                // <c-sort>`. The propagation is a closed function of the
                // entry-point synth-fun's value: a smaller, BV-flat
                // body that any structural / bit-circuit optimiser can
                // compile away. The fixpoint constraint then reduces to
                // `(= c (bvor c_baseline (c_propagation <entry>)))`.
                let arg_name = "arg0".to_string();
                for c in active {
                    let s = &sort_for[c];
                    let width = layout.width(s);
                    let body = build_fixpoint_body(
                        width,
                        links_for.get(c).map(|v| v.as_slice()).unwrap_or(&[]),
                        &arg_name,
                    );
                    out.push(SyGuSCmd::DefineFun(
                        format!("{}_propagation", sanitize(c)),
                        vec![sygus_parser::SortedVar {
                            name: arg_name.clone(),
                            sort: entry_ret.clone(),
                        }],
                        Sort::from_name(s),
                        body,
                    ));
                }
                // Fixpoint constraints — each `(= c (bvor c_baseline
                // (c_propagation <entry>)))`.
                for c in active {
                    let lhs = SyGuSTerm::Identifier(Identifier::Symbol(c.clone()));
                    let baseline = SyGuSTerm::Identifier(Identifier::Symbol(format!(
                        "{}_baseline",
                        sanitize(c)
                    )));
                    let prop_call = SyGuSTerm::Application(
                        Identifier::Symbol(format!("{}_propagation", sanitize(c))),
                        vec![SyGuSTerm::Identifier(Identifier::Symbol(synth_name.clone()))],
                    );
                    let rhs = SyGuSTerm::Application(
                        Identifier::Symbol("bvor".to_string()),
                        vec![baseline, prop_call],
                    );
                    let eq = Bool::eq(lhs, rhs);
                    out.push(SyGuSCmd::Constraint(eq));
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// Build a `width`-bit BV where bit `b` is the bvor of
/// `((_ extract tgt tgt) <arg>)` over each (b, tgt) in the bit-link
/// table, or `#b0` when there are no links for bit `b`. `<arg>` is the
/// caller-supplied arg name (the propagation define-fun's parameter).
///
/// Emission is two passes:
/// 1. Per-bit MSB→LSB, choose between `#b0`, a single 1-bit extract,
///    or `(bvor extract1 extract2 …)`.
/// 2. Coalesce consecutive `#b0` bits into one wider zero literal.
///    Then fold the chunks into a left-deep `(concat …)` tree.
///
/// Each bit is either a constant or a single 1-bit BV expression — no
/// `(ite (= x #b1) #b1 #b0)` wrappers, no per-bit redundant equality.
fn build_fixpoint_body(
    width: usize,
    links: &[(usize, usize)],
    arg_name: &str,
) -> SyGuSTerm {
    let mut by_src: HashMap<usize, Vec<usize>> = HashMap::default();
    for &(src, tgt) in links {
        by_src.entry(src).or_default().push(tgt);
    }
    let input = SyGuSTerm::Identifier(Identifier::Symbol(arg_name.to_string()));

    enum Chunk {
        Zero(usize),
        Bv1(SyGuSTerm),
    }

    // Pass 1: build per-bit chunks MSB→LSB, coalescing runs of Zero.
    let mut chunks: Vec<Chunk> = Vec::with_capacity(width);
    for b in (0..width).rev() {
        let next = match by_src.get(&b) {
            None => Chunk::Zero(1),
            Some(tgts) if tgts.len() == 1 => {
                Chunk::Bv1(BitVec::extract(tgts[0], tgts[0], input.clone()))
            }
            Some(tgts) => {
                // bvor over 1-bit extracts. Fold left for stability.
                let mut iter = tgts.iter();
                let first = iter.next().unwrap();
                let mut term = BitVec::extract(*first, *first, input.clone());
                for &t in iter {
                    term = BitVec::bvor(term, BitVec::extract(t, t, input.clone()));
                }
                Chunk::Bv1(term)
            }
        };
        match (chunks.last_mut(), &next) {
            (Some(Chunk::Zero(w)), Chunk::Zero(1)) => {
                *w += 1;
            }
            _ => chunks.push(next),
        }
    }

    // Pass 2: fold into a left-deep concat tree.
    fn chunk_term(c: Chunk) -> SyGuSTerm {
        match c {
            Chunk::Zero(w) => BitVec::const_(0, w),
            Chunk::Bv1(t) => t,
        }
    }
    let mut iter = chunks.into_iter();
    let mut body = iter.next().map(chunk_term).unwrap_or_else(|| BitVec::const_(0, width));
    for c in iter {
        body = BitVec::concat(body, chunk_term(c));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::syntax::TermConstructor;

    fn ctor(name: &str, args: Vec<&str>, result: &str) -> TermConstructor {
        TermConstructor {
            name: name.to_string(),
            arg_sorts: args.into_iter().map(String::from).collect(),
            result_sort: result.to_string(),
        }
    }

    fn sig_from(cons: Vec<TermConstructor>, start: &str) -> TermSignature {
        let mut seen: Vec<String> = Vec::new();
        for c in &cons {
            if !seen.contains(&c.result_sort) {
                seen.push(c.result_sort.clone());
            }
            for s in &c.arg_sorts {
                if !seen.contains(s) {
                    seen.push(s.clone());
                }
            }
        }
        TermSignature::from_parts(cons, seen, start.to_string())
    }

    #[test]
    fn no_parent_nullaries_in_reg_shape() {
        let sig = sig_from(
            vec![
                ctor("char.0", vec![], "RegLan"),
                ctor("char.1", vec![], "RegLan"),
                ctor("re.allchar", vec![], "RegLan"),
                ctor("re.++", vec!["RegLan", "RegLan"], "RegLan"),
                ctor("re.*", vec!["RegLan"], "RegLan"),
            ],
            "RegLan",
        );
        let mut got = parent_referencing_nullaries(&sig);
        got.sort();
        assert_eq!(got, vec!["char.0", "char.1", "re.allchar"]);
    }

    #[test]
    fn cfg_nullaries_detected_generically() {
        let sig = sig_from(
            vec![
                ctor("cfg.top.S0", vec!["Rhs", "Rules"], "CFG"),
                ctor("a", vec![], "Rhs"),
                ctor("b", vec![], "Rhs"),
                ctor("cfg.++", vec!["Rhs", "Rhs"], "Rhs"),
                ctor("cfg.rhs.S0", vec![], "Rhs"),
                ctor("cfg.lhs.S0", vec!["Rhs", "Rules"], "Rules"),
                ctor("end", vec![], "Rules"),
            ],
            "CFG",
        );
        let mut got = parent_referencing_nullaries(&sig);
        got.sort();
        assert_eq!(got, vec!["a", "b", "cfg.rhs.S0", "end"]);
    }

    #[test]
    fn start_sort_only_no_arg_use() {
        let sig = sig_from(
            vec![ctor("cfg.top.S0", vec!["Rhs", "Rules"], "CFG"), ctor("a", vec![], "Rhs")],
            "CFG",
        );
        assert_eq!(parent_referencing_nullaries(&sig), vec!["a"]);
    }
}
