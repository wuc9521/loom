//! Auto-derivation of LFP bit-link triples from HornProg POP rules.
//!
//! For each rule of the form
//!
//! ```text
//! qChild_pred(child_app) :- qParent_pred(parent_app(child_app, ...))
//! ```
//!
//! the LFP closure says: at any node `u` of `g` whose constructor matches
//! `child_app`, `u.qChild_pred` holds iff `u`'s parent (whose constructor
//! matches `parent_app`) has `qParent_pred`. After propagation through any
//! intermediate constructors whose own POP rules preserve the predicate
//! (e.g. CFG's `reset.S0` mode at "_" parents), this reduces to
//! `u.qChild_pred = g.qRootEquivalent_pred` for nullary leaves like
//! `cfg.rhs.S0`.
//!
//! This module extracts `(child_cons_name, src_pred, tgt_pred)` triples
//! that downstream emit consumes to add the LFP closure constraints.
//!
//! ## Known limitation
//!
//! The triples only encode one POP step. Whether the closure actually
//! reduces to `g[tgt_pred]` depends on the spec's propagation pattern.
//! For CFG-style semantics where `(parent x)` is called only on a single
//! mode that propagates unchanged at intermediate constructors, the
//! reduction holds. For semantics with bounded-depth `(parent (parent x))`,
//! more analysis is needed.

use crate::backends::smt2::ast::{HornProg, HornRule, HornTerm};

/// `(child constructor name, source predicate, target predicate)`.
///
/// The constraint encoded: at every position of `child_cons_name` in `g`,
/// `child.bit[src_pred] = parent.bit[tgt_pred]` after the LFP propagation
/// settles to `g.bit[tgt_pred]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitLinkTriple {
    pub child_cons: String,
    pub src_pred: String,
    pub tgt_pred: String,
}

/// Scan `prog.rules` for POP rules and return the list of bit-link triples.
pub fn derive_bit_links(prog: &HornProg) -> Vec<BitLinkTriple> {
    let mut out = Vec::new();
    for rule in &prog.rules {
        if let Some(t) = classify_pop(rule) {
            out.push(t);
        }
    }
    out
}

/// True iff `rule` is a POP rule of shape
/// `qChild_pred(child_app) :- qParent_pred(parent_app(... child_app ...))`.
/// Returns the `(child_cons, src_pred, tgt_pred)` triple if so.
fn classify_pop(rule: &HornRule) -> Option<BitLinkTriple> {
    if rule.body_atoms.len() != 1 {
        return None;
    }
    let body = &rule.body_atoms[0];
    let head = &rule.head;
    // head.arg must be an App (constructor application). The child constructor
    // is that App's `fun`.
    let HornTerm::App { fun: child_cons, .. } = &head.arg else {
        return None;
    };
    // body.arg must also be App (the parent's constructor), and must contain
    // head.arg as a direct child.
    let HornTerm::App { args: parent_args, .. } = &body.arg else {
        return None;
    };
    if !parent_args.iter().any(|a| a == &head.arg) {
        return None;
    }
    Some(BitLinkTriple {
        child_cons: child_cons.clone(),
        src_pred: head.pred.clone(),
        tgt_pred: body.pred.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::smt2::ast::{HornAtom, HornRule, HornTerm};

    fn app(fun: &str, args: Vec<HornTerm>) -> HornTerm {
        HornTerm::app(fun, args)
    }
    fn var(name: &str) -> HornTerm {
        HornTerm::var(name)
    }
    fn atom(pred: &str, arg: HornTerm) -> HornAtom {
        HornAtom {
            pred: pred.to_string(),
            arg,
        }
    }

    #[test]
    fn detects_nullary_pop() {
        // qRhs.base(cfg.rhs.S0) :- qCFG.reset.S0(cfg.top.S0(cfg.rhs.S0, __up1))
        let rule = HornRule {
            body_atoms: vec![atom(
                "qCFG.reset.S0",
                app("cfg.top.S0", vec![app("cfg.rhs.S0", vec![]), var("__up1")]),
            )],
            head: atom("qRhs.base", app("cfg.rhs.S0", vec![])),
        };
        let t = classify_pop(&rule).expect("should detect");
        assert_eq!(t.child_cons, "cfg.rhs.S0");
        assert_eq!(t.src_pred, "qRhs.base");
        assert_eq!(t.tgt_pred, "qCFG.reset.S0");
    }

    #[test]
    fn detects_binary_pop() {
        // qRhs.reset.S0(cfg.++(x, y)) :- qCFG.reset.S0(cfg.top.S0(cfg.++(x, y), __up1))
        let cpp = app("cfg.++", vec![var("x"), var("y")]);
        let rule = HornRule {
            body_atoms: vec![atom(
                "qCFG.reset.S0",
                app("cfg.top.S0", vec![cpp.clone(), var("__up1")]),
            )],
            head: atom("qRhs.reset.S0", cpp),
        };
        let t = classify_pop(&rule).expect("should detect");
        assert_eq!(t.child_cons, "cfg.++");
        assert_eq!(t.src_pred, "qRhs.reset.S0");
        assert_eq!(t.tgt_pred, "qCFG.reset.S0");
    }

    #[test]
    fn skips_push() {
        // qRhs.base(cfg.++(x, y)) :- qRhs.base(x), qRhs.base(y)  — PUSH, not POP
        let rule = HornRule {
            body_atoms: vec![
                atom("qRhs.base", var("x")),
                atom("qRhs.base", var("y")),
            ],
            head: atom("qRhs.base", app("cfg.++", vec![var("x"), var("y")])),
        };
        assert!(classify_pop(&rule).is_none(), "PUSH must not classify as POP");
    }

    #[test]
    fn skips_non_app_head() {
        let rule = HornRule {
            body_atoms: vec![atom("qParent", app("cons", vec![var("x")]))],
            head: atom("qChild", var("x")),
        };
        assert!(classify_pop(&rule).is_none());
    }

    #[test]
    fn skips_body_without_head_arg_child() {
        let rule = HornRule {
            body_atoms: vec![atom("qParent", app("cons", vec![var("y")]))],
            head: atom("qChild", app("c", vec![])),
        };
        assert!(classify_pop(&rule).is_none());
    }
}
