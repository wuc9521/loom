//! Automatic dualization of FACET semantics for sound negative examples.
//!
//! A `define-semantics` relation `P` is an existential/least-fixpoint
//! acceptance: `P(g, σ)` holds iff the program has a (founded) derivation. The
//! founded encoding witnesses that soundly — but a negative example
//! `(not (P g σ))` asks to witness *non*-membership, which negating the
//! acceptance bit cannot do (you can always present a run that declines to
//! fire). Per Krogmeier–Madhusudan 2023 §4.4, negation is handled by the
//! **dual program**: `P_dual ≡ ¬P`, obtained as the negation-normal-form of the
//! body with recursive calls redirected to their duals. The dual is itself an
//! existential automaton, so its founded acceptance soundly witnesses
//! rejection. We derive `P_dual` automatically (no source change) and compile
//! negative examples as `(P_dual g σ)`.

use std::collections::HashSet;

use sygus_parser::SExpr as Value;

use crate::compile::types::SemanticSpec;

const DUAL_SUFFIX: &str = ".dual";

/// The dual relation name for `relation`.
pub fn dual_name(relation: &str) -> String {
    format!("{relation}{DUAL_SUFFIX}")
}

/// Dual specs for the whole (mutually-recursive) relation set: each computes
/// the negation of its primal, with recursive calls redirected to the dual.
pub fn dual_specs(specs: &[SemanticSpec]) -> Vec<SemanticSpec> {
    let rels: HashSet<String> = specs.iter().map(|s| s.relation.clone()).collect();
    specs
        .iter()
        .map(|s| SemanticSpec {
            relation: dual_name(&s.relation),
            term_sort: s.term_sort.clone(),
            aspect_var: s.aspect_var.clone(),
            aspect_space: s.aspect_space.clone(),
            return_kind: s.return_kind.clone(),
            body: dual(&s.body, &rels),
        })
        .collect()
}

fn cl(v: &Value) -> Value {
    v.clone()
}

fn negate(expr: &Value) -> Value {
    Value::list(vec![Value::symbol("not"), cl(expr)])
}

/// Negation-normal-form of `¬expr`: flip `and`↔`or` and `exists`↔`forall`,
/// `true`↔`false`; keep `ite`/`match` dispatch and dualize the result
/// branches; redirect a semantic call `(P …)` to `(P.dual …)` (arguments
/// unchanged — they are term/aspect navigation, not booleans); and negate
/// signature/comparison atoms (`(a i)`, `(= …)`, …).
fn dual(expr: &Value, rels: &HashSet<String>) -> Value {
    let Some(items) = expr.to_ref_vec() else {
        return match expr.as_symbol() {
            Some("true") => Value::symbol("false"),
            Some("false") => Value::symbol("true"),
            _ => negate(expr),
        };
    };
    let Some(op) = items.first().and_then(|i| i.as_symbol()) else {
        // A grouping list whose head is itself a list — e.g. the
        // `define-semantics` body wrapper `((match …))`. Dualize inside,
        // preserving the wrapper.
        if items.len() == 1 {
            return Value::list(vec![dual(items[0], rels)]);
        }
        return negate(expr);
    };
    match op {
        "and" | "or" => {
            let flipped = if op == "and" { "or" } else { "and" };
            let mut out = vec![Value::symbol(flipped)];
            out.extend(items[1..].iter().map(|e| dual(e, rels)));
            Value::list(out)
        }
        "not" if items.len() == 2 => cl(items[1]), // ¬¬x = x
        "ite" if items.len() == 4 => Value::list(vec![
            Value::symbol("ite"),
            cl(items[1]), // condition kept
            dual(items[2], rels),
            dual(items[3], rels),
        ]),
        "exists" | "forall" if items.len() == 3 => {
            let flipped = if op == "exists" { "forall" } else { "exists" };
            Value::list(vec![
                Value::symbol(flipped),
                cl(items[1]),
                dual(items[2], rels),
            ])
        }
        "match" if items.len() >= 2 => {
            let mut out = vec![Value::symbol("match"), cl(items[1])]; // scrutinee
            for case in &items[2..] {
                match case.to_ref_vec().as_deref() {
                    Some([pattern, body]) => {
                        out.push(Value::list(vec![cl(pattern), dual(body, rels)]))
                    }
                    _ => out.push(cl(case)),
                }
            }
            Value::list(out)
        }
        _ if rels.contains(op) => {
            // semantic call → dual call, arguments preserved
            let mut out = vec![Value::symbol(dual_name(op))];
            out.extend(items[1..].iter().map(|e| cl(e)));
            Value::list(out)
        }
        _ => negate(expr),
    }
}
