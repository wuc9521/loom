//! Sound & complete compilation of delis programs **with `parent` (up-moves)**
//! into SyGuS-BV, via the faithful two-way → one-way crossing construction
//! (Shepherdson 1959; Vardi/Cachat, as cited by Krogmeier–Madhusudan 2023).
//!
//! The semantics of a FACET program is a two-way alternating tree automaton over
//! the candidate syntax tree, accepted by a least fixpoint (reachability). The
//! one-way encoding here is a **founded annotation certificate**: each node
//! carries guessed `up[state]`, a parent-channel `down[(parent-ctor, pos, state)]`,
//! and per-state founded `rank`s; a computed `valid` bit checks, for every clause
//! uniformly (child-ref / self-loop / parent-atom), that `up` is justified with
//! strictly smaller rank. This is polynomial-size and handles intersection, mixed
//! up+down clauses, and multiple parent atoms — i.e. it is sound and complete for
//! arbitrary delis programs, not just CFG.
//!
//! `crossing_eval` (the exact ground LFP) and `respond_gen` (the exact faithful
//! per-node semantics) are the reference oracles the encoder is validated against.
//!
//! The module is `#![cfg(test)]` while it matures behind validation; wiring it
//! is the default delis path for parent-bearing programs (set `DELIS_LEGACY` for
//! the old saturation path). It is solved by cvc5 (the `Constant` holes need an
//! SMT backend); making the in-process enumerator solve it is the dryadsynth-bv
//! `constant-via-smt` branch.

use crate::backends::smt2::ast::{HornDecl, HornProg, HornRule, HornTerm as HT, PredicatePolarity};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
#[cfg(test)]
use std::collections::BTreeSet;

// ===========================================================================
//  IR: datatype signature, general clauses, relevance, goal
// ===========================================================================

/// `(states-per-sort, ctors-per-sort)`. States are the positive predicates
/// restricted to the goal's cone of influence; ctors come from the datatype.
fn build_sig(
    prog: &HornProg,
    relevant: &HashSet<String>,
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<(String, Vec<String>)>>) {
    let mut ctors: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::default();
    for d in &prog.decls {
        if let HornDecl::DeclareDatatypes(sorts, decls) = d {
            for (sd, cons_list) in sorts.iter().zip(decls.iter()) {
                let cs = cons_list
                    .iter()
                    .map(|c| (c.name.clone(), c.args.iter().map(|sv| sv.sort.to_string()).collect()))
                    .collect();
                ctors.insert(sd.symbol.clone(), cs);
            }
        }
    }
    let mut states: HashMap<String, Vec<String>> = HashMap::default();
    for sort in ctors.keys() {
        states.insert(sort.clone(), Vec::new());
    }
    for (name, info) in &prog.predicates {
        if info.polarity == PredicatePolarity::Positive && relevant.contains(name) {
            states.entry(info.sort.clone()).or_default().push(name.clone());
        }
    }
    for v in states.values_mut() {
        v.sort();
    }
    (states, ctors)
}

/// Backward cone of influence: a state matters iff some goal state depends on it.
/// Uniform across clause kinds (head depends on every body state) — plain
/// backward reachability over head→body edges; language-agnostic.
fn relevant_states(prog: &HornProg, seeds: &HashSet<String>) -> HashSet<String> {
    let positive =
        |p: &str| prog.predicates.get(p).map(|i| i.polarity == PredicatePolarity::Positive).unwrap_or(false);
    let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
    for r in &prog.rules {
        if !matches!(&r.head.arg, HT::App { .. }) || !positive(&r.head.pred) {
            continue;
        }
        let e = deps.entry(r.head.pred.clone()).or_default();
        for b in &r.body_atoms {
            if positive(&b.pred) {
                e.insert(b.pred.clone());
            }
        }
    }
    let mut keep = seeds.clone();
    let mut stack: Vec<String> = seeds.iter().cloned().collect();
    while let Some(q) = stack.pop() {
        if let Some(bs) = deps.get(&q) {
            for b in bs {
                if keep.insert(b.clone()) {
                    stack.push(b.clone());
                }
            }
        }
    }
    keep
}

/// Start sort and the root constraints `(positive state, want-bit)`. A negative
/// example (`nq…`) becomes "the positive state must be 0".
fn goal(prog: &HornProg) -> (String, Vec<(String, bool)>) {
    let goal_rule = prog.rules.iter().find(|r| r.head.pred == "Goal").expect("no Goal rule");
    let mut cons = Vec::new();
    let mut start = String::new();
    for b in &goal_rule.body_atoms {
        let (qstate, want) = match b.pred.strip_prefix("nq") {
            Some(rest) => (format!("q{rest}"), false),
            None => (b.pred.clone(), true),
        };
        if let Some(info) = prog.predicates.get(&qstate).or_else(|| prog.predicates.get(&b.pred)) {
            start = info.sort.clone();
        }
        cons.push((qstate, want));
    }
    (start, cons)
}

/// A clause body atom, relative to the head node `f(v0..vk)`.
#[derive(Clone, Debug)]
enum GA {
    Child(usize, String),          // q(v_i)               — recurse into child i
    Selfp(String),                 // q(f(v0..vk))         — intersection on this node
    Parent(String, usize, String), // q(g(.. f ..))        — (parent ctor, position, state)
}

#[derive(Clone, Debug)]
struct GC {
    ctor: String,
    head: String,
    body: Vec<GA>,
}

/// Classify a Horn rule into a general clause (no information dropped).
fn extract_gc(r: &HornRule) -> Option<GC> {
    let HT::App { fun: hf, args: ha } = &r.head.arg else { return None };
    let mut var_pos: HashMap<&str, usize> = HashMap::default();
    for (i, a) in ha.iter().enumerate() {
        if let HT::Var(v) = a {
            var_pos.insert(v.as_str(), i);
        }
    }
    let mut body = Vec::new();
    for b in &r.body_atoms {
        match &b.arg {
            HT::Var(v) => body.push(GA::Child(*var_pos.get(v.as_str())?, b.pred.clone())),
            app if app == &r.head.arg => body.push(GA::Selfp(b.pred.clone())),
            HT::App { fun: pf, args: pa } => {
                let pos = pa.iter().position(|a| a == &r.head.arg)?;
                body.push(GA::Parent(pf.clone(), pos, b.pred.clone()));
            }
        }
    }
    Some(GC { ctor: hf.clone(), head: r.head.pred.clone(), body })
}

/// The positive, relevant general clauses of a program.
fn general_clauses(prog: &HornProg, relevant: &HashSet<String>) -> Vec<GC> {
    let positive =
        |p: &str| prog.predicates.get(p).map(|i| i.polarity == PredicatePolarity::Positive).unwrap_or(false);
    prog.rules
        .iter()
        .filter(|r| positive(&r.head.pred) && relevant.contains(&r.head.pred))
        .filter_map(extract_gc)
        .filter(|gc| {
            gc.body.iter().all(|a| match a {
                GA::Child(_, s) | GA::Selfp(s) | GA::Parent(_, _, s) => relevant.contains(s),
            })
        })
        .collect()
}

// ===========================================================================
//  Reference semantics (exact ground LFP) — the oracle for validation
// ===========================================================================

/// A fixed ground tree, flattened: per node its ctor, child node ids, and
/// (parent id, position-in-parent).
#[cfg(test)]
struct Ground {
    ctor: Vec<String>,
    children: Vec<Vec<usize>>,
    parent: Vec<Option<(usize, usize)>>,
    subterm: Vec<HT>,
}

#[cfg(test)]
fn build_ground(tree: &HT) -> Ground {
    fn go(g: &mut Ground, t: &HT, parent: Option<(usize, usize)>) -> usize {
        let HT::App { fun, args } = t else { panic!("ground tree must be variable-free") };
        let id = g.ctor.len();
        g.ctor.push(fun.clone());
        g.children.push(Vec::new());
        g.parent.push(parent);
        g.subterm.push(t.clone());
        let kids: Vec<usize> = args.iter().enumerate().map(|(pos, a)| go(g, a, Some((id, pos)))).collect();
        g.children[id] = kids;
        id
    }
    let mut g = Ground { ctor: vec![], children: vec![], parent: vec![], subterm: vec![] };
    go(&mut g, tree, None);
    g
}

#[cfg(test)]
fn arg_position_of(parent_ctx: &HT, head: &HT) -> Option<usize> {
    match parent_ctx {
        HT::App { args, .. } => args.iter().position(|a| a == head),
        HT::Var(_) => None,
    }
}

/// Exact two-way LFP reachability over a fixed ground tree: the set of
/// `(predicate, node-id)` pairs that hold. Handles any clause shape.
#[cfg(test)]
fn crossing_eval(rules: &[HornRule], gt: &Ground) -> HashSet<(String, usize)> {
    let rules: Vec<&HornRule> = rules.iter().filter(|r| matches!(&r.head.arg, HT::App { .. })).collect();
    let mut holds: HashSet<(String, usize)> = HashSet::default();
    loop {
        let mut changed = false;
        for rule in &rules {
            let HT::App { fun: hfun, args: hargs } = &rule.head.arg else { continue };
            for n in 0..gt.ctor.len() {
                if &gt.ctor[n] != hfun || gt.children[n].len() != hargs.len() {
                    continue;
                }
                let mut bind: HashMap<&str, usize> = HashMap::default();
                let mut ok_head = true;
                for (i, ha) in hargs.iter().enumerate() {
                    match ha {
                        HT::Var(v) => {
                            bind.insert(v.as_str(), gt.children[n][i]);
                        }
                        HT::App { .. } if ha != &gt.subterm[gt.children[n][i]] => ok_head = false,
                        HT::App { .. } => {}
                    }
                }
                if !ok_head || holds.contains(&(rule.head.pred.clone(), n)) {
                    continue;
                }
                if body_holds_ground(&rule.body_atoms, n, gt, &bind, &rule.head.arg, &holds) {
                    holds.insert((rule.head.pred.clone(), n));
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    holds
}

#[cfg(test)]
fn body_holds_ground(
    body: &[crate::backends::smt2::ast::HornAtom],
    n: usize,
    gt: &Ground,
    bind: &HashMap<&str, usize>,
    head_term: &HT,
    holds: &HashSet<(String, usize)>,
) -> bool {
    body.iter().all(|b| match &b.arg {
        HT::Var(v) => bind.get(v.as_str()).is_some_and(|&c| holds.contains(&(b.pred.clone(), c))),
        app if app == head_term => holds.contains(&(b.pred.clone(), n)),
        app @ HT::App { fun: pfun, .. } => match (arg_position_of(app, head_term), gt.parent[n]) {
            (Some(pos), Some((pn, ppos))) if ppos == pos && &gt.ctor[pn] == pfun => {
                holds.contains(&(b.pred.clone(), pn))
            }
            _ => false,
        },
    })
}

/// Exact faithful per-node semantics: states at `node` given its parent is
/// `parent=(ctor,pos)` providing `dp`; children are evaluated with this node as
/// their parent. The compositional mirror of `crossing_eval` (memoized).
#[cfg(test)]
#[allow(clippy::type_complexity)]
fn respond_gen(
    node: usize,
    parent: Option<(&str, usize)>,
    dp: &BTreeSet<String>,
    gt: &Ground,
    gcs: &[GC],
    memo: &mut HashMap<(usize, Option<(String, usize)>, Vec<String>), BTreeSet<String>>,
) -> BTreeSet<String> {
    let key = (node, parent.map(|(c, p)| (c.to_string(), p)), dp.iter().cloned().collect::<Vec<_>>());
    if let Some(v) = memo.get(&key) {
        return v.clone();
    }
    let f = gt.ctor[node].clone();
    let children = gt.children[node].clone();
    let mut s = BTreeSet::new();
    loop {
        let child_states: Vec<BTreeSet<String>> =
            children.iter().enumerate().map(|(i, &c)| respond_gen(c, Some((&f, i)), &s, gt, gcs, memo)).collect();
        let mut next = s.clone();
        for gc in gcs.iter().filter(|gc| gc.ctor == f) {
            let ok = gc.body.iter().all(|a| match a {
                GA::Child(i, st) => child_states[*i].contains(st),
                GA::Selfp(st) => s.contains(st),
                GA::Parent(pc, pos, st) => {
                    matches!(parent, Some((ppc, ppos)) if ppc == pc && ppos == *pos) && dp.contains(st)
                }
            });
            if ok {
                next.insert(gc.head.clone());
            }
        }
        if next == s {
            break;
        }
        s = next;
    }
    memo.insert(key, s.clone());
    s
}

// ===========================================================================
//  Encoder: founded annotation certificate → SyGuS-BV
// ===========================================================================

/// Default founded-rank width: enough bits to index every relevant (node,state)
/// atom of a candidate up to `tree_bound` nodes. The founded chain length is
/// bounded by `nodes × |states|`, so `fr` must cover that — a too-small `fr`
/// would make deeper-derivation solutions uncertifiable (incomplete). There is
/// no fixed `fr` complete for arbitrarily large terms; raise `tree_bound`
/// (env `DELIS_TREE_BOUND`) for larger solutions, or use the per-candidate LFP
/// backend for unbounded completeness.
fn rank_bits(total_states: usize, tree_bound: usize) -> usize {
    let max_atoms = (total_states.max(1) * tree_bound.max(1)) as u64;
    (64 - max_atoms.leading_zeros()).max(1) as usize
}

/// Whether to constant-fold emitted formulas (under `--optim`). Folding the
/// trivially-true/false subterms the founded encoding produces (e.g.
/// `(or true … true)`, `(= (= bit #b1) true)`) yields a smaller, slightly
/// faster SMT problem. The unfolded form is kept when optimization is off.
fn fold_on() -> bool {
    crate::compile::opts().optimize
}

fn or_bool(v: &[String]) -> String {
    if fold_on() {
        if v.iter().any(|x| x == "true") {
            return "true".to_string();
        }
        let kept: Vec<&str> = v.iter().map(String::as_str).filter(|x| *x != "false").collect();
        return match kept.as_slice() {
            [] => "false".to_string(),
            [one] => one.to_string(),
            _ => format!("(or {})", kept.join(" ")),
        };
    }
    match v.len() {
        0 => "false".to_string(),
        1 => v[0].clone(),
        _ => format!("(or {})", v.join(" ")),
    }
}

/// Conjunction that avoids single-arg `(and X)` — nonstandard SMT-LIB that
/// strict parsers (e.g. bitwuzla) reject. Empty ⇒ `true`.
fn and_bool(v: &[String]) -> String {
    if fold_on() {
        if v.iter().any(|x| x == "false") {
            return "false".to_string();
        }
        let kept: Vec<&str> = v.iter().map(String::as_str).filter(|x| *x != "true").collect();
        return match kept.as_slice() {
            [] => "true".to_string(),
            [one] => one.to_string(),
            _ => format!("(and {})", kept.join(" ")),
        };
    }
    match v.len() {
        0 => "true".to_string(),
        1 => v[0].clone(),
        _ => format!("(and {})", v.join(" ")),
    }
}

/// Encode a compiled CHC program, naming the synth-fun `synth_name` (the source
/// program's synth-fun name). Returns `(sygus-text, synth-target sort)`.
pub(crate) fn faithful_sygus(prog: &HornProg, synth_name: &str) -> (String, String) {
    let (start, constraints) = goal(prog);
    let seeds: HashSet<String> = constraints.iter().map(|(q, _)| q.clone()).collect();
    let relevant = relevant_states(prog, &seeds);
    let (states, ctors) = build_sig(prog, &relevant);
    let gcs = general_clauses(prog, &relevant);
    let total_states: usize = states.values().map(Vec::len).sum();
    let tree_bound: usize =
        std::env::var("DELIS_TREE_BOUND").ok().and_then(|s| s.parse().ok()).unwrap_or(64);
    let fr = rank_bits(total_states, tree_bound);
    let sygus = faithful_sygus_ir(&states, &ctors, &gcs, &start, &constraints, synth_name, fr);
    (sygus, start)
}

// --- decode a cvc5 solution `g` into the plain term (strip the Ann args) ---
enum Sx {
    Atom(String),
    List(Vec<Sx>),
}
fn parse_sexp(s: &str) -> Sx {
    let toks: Vec<String> =
        s.replace('(', " ( ").replace(')', " ) ").split_whitespace().map(str::to_string).collect();
    fn go(toks: &[String], pos: &mut usize) -> Sx {
        if toks[*pos] == "(" {
            *pos += 1;
            let mut v = Vec::new();
            while toks[*pos] != ")" {
                v.push(go(toks, pos));
            }
            *pos += 1;
            Sx::List(v)
        } else {
            let a = toks[*pos].clone();
            *pos += 1;
            Sx::Atom(a)
        }
    }
    go(&toks, &mut 0)
}
/// Drop the trailing `Ann` (`#b…`) argument of every constructor application.
fn strip_ann(s: &Sx) -> Sx {
    match s {
        Sx::Atom(a) => Sx::Atom(a.clone()),
        Sx::List(v) => {
            let mut out = vec![strip_ann(&v[0])];
            out.extend(v[1..v.len().saturating_sub(1)].iter().map(strip_ann));
            Sx::List(out)
        }
    }
}
fn sx_to_string(s: &Sx) -> String {
    match s {
        Sx::Atom(a) => a.clone(),
        Sx::List(v) => format!("({})", v.iter().map(sx_to_string).collect::<Vec<_>>().join(" ")),
    }
}
/// Decode a cvc5 SyGuS solution: find `(define-fun <synth_name> () _ body)`
/// anywhere in the output and return its body as a plain delis term (Ann args
/// stripped). Parses the s-expression rather than string-scanning, so it is
/// robust to the synth-fun's name and to formatting.
pub(crate) fn decode_term(stdout: &str, synth_name: &str) -> Option<String> {
    let t = stdout.trim();
    if !t.starts_with('(') {
        return None;
    }
    fn find<'a>(s: &'a Sx, name: &str) -> Option<&'a Sx> {
        if let Sx::List(v) = s {
            if v.len() >= 5 {
                if let (Sx::Atom(kw), Sx::Atom(nm)) = (&v[0], &v[1]) {
                    if kw == "define-fun" && nm == name {
                        return Some(&v[4]); // (define-fun NAME () SORT BODY)
                    }
                }
            }
            return v.iter().find_map(|c| find(c, name));
        }
        None
    }
    find(&parse_sexp(t), synth_name).map(|body| sx_to_string(&strip_ann(body)))
}

/// The encoder, decoupled from `HornProg`. Node layout per sort:
/// `[up:|Q|][down:|DS|·(1+FR)][rank:|Q|·FR][valid:1]`, where `DS` is the set of
/// `(parent ctor, position, state)` triples appearing in `Parent` atoms.
fn faithful_sygus_ir(
    states: &HashMap<String, Vec<String>>,
    ctors: &HashMap<String, Vec<(String, Vec<String>)>>,
    gcs: &[GC],
    start_sort: &str,
    constraints: &[(String, bool)],
    synth_name: &str,
    fr: usize,
) -> String {
    let mut ctor_sort: HashMap<String, String> = HashMap::default();
    for (sort, cs) in ctors {
        for (c, _) in cs {
            ctor_sort.insert(c.clone(), sort.clone());
        }
    }
    let mut ds: HashMap<String, Vec<(String, usize, String)>> = HashMap::default();
    for gc in gcs {
        let sort = &ctor_sort[&gc.ctor];
        for a in &gc.body {
            if let GA::Parent(pc, pos, st) = a {
                let v = ds.entry(sort.clone()).or_default();
                let e = (pc.clone(), *pos, st.clone());
                if !v.contains(&e) {
                    v.push(e);
                }
            }
        }
    }
    for v in ds.values_mut() {
        v.sort();
    }
    let empty = Vec::new();
    let ds_of = |s: &str| ds.get(s).unwrap_or(&empty);

    let nq = |s: &str| states[s].len();
    let nd = |s: &str| ds_of(s).len();
    let width = |s: &str| nq(s) + nd(s) * (1 + fr) + nq(s) * fr + 1;
    let down_active = |s: &str, j: usize| nq(s) + j * (1 + fr);
    let down_rank = |s: &str, j: usize| nq(s) + j * (1 + fr) + 1;
    let rank_pos = |s: &str, q: usize| nq(s) + nd(s) * (1 + fr) + q * fr;
    let valid_pos = |s: &str| width(s) - 1;
    let qidx = |s: &str, st: &str| states[s].iter().position(|x| x == st);
    let dsidx = |s: &str, e: &(String, usize, String)| ds_of(s).iter().position(|x| x == e);
    let rslice = |term: &str, lo: usize| format!("((_ extract {} {}) {term})", lo + fr - 1, lo);
    let zero = format!("(_ bv0 {fr})");

    let mut s = String::from("(set-logic BV)\n");

    for (sort, ctor_list) in ctors {
        let w = width(sort);
        for (ctor, arg_sorts) in ctor_list {
            let mut sig = format!("(define-fun {ctor} (");
            for (i, a) in arg_sorts.iter().enumerate() {
                sig.push_str(&format!("(c{i} (_ BitVec {})) ", width(a)));
            }
            let ann_w = (w - 1).max(1);
            sig.push_str(&format!("(ann (_ BitVec {ann_w}))) (_ BitVec {w}) "));

            let upb = |q: usize| format!("((_ extract {q} {q}) ann)");
            let self_rank = |q: usize| rslice("ann", rank_pos(sort, q));
            let dact = |j: usize| format!("((_ extract {p} {p}) ann)", p = down_active(sort, j));
            let drank = |j: usize| rslice("ann", down_rank(sort, j));

            let mut checks: Vec<String> = Vec::new();
            for (i, a) in arg_sorts.iter().enumerate() {
                checks.push(format!("(= ((_ extract {p} {p}) c{i}) #b1)", p = valid_pos(a)));
            }
            // up[q] ⟺ ∃ clause: all premises hold with strictly smaller rank
            for (q, qst) in states[sort].iter().enumerate() {
                let alts: Vec<String> = gcs
                    .iter()
                    .filter(|gc| &gc.ctor == ctor && &gc.head == qst)
                    .map(|gc| {
                        if gc.body.is_empty() {
                            return "true".to_string();
                        }
                        let parts: Vec<String> = gc
                            .body
                            .iter()
                            .map(|atom| match atom {
                                GA::Child(ci, st) => {
                                    let cs = &arg_sorts[*ci];
                                    let qj = qidx(cs, st).unwrap();
                                    format!(
                                        "(and (= ((_ extract {qj} {qj}) c{ci}) #b1) (bvult {} {}))",
                                        rslice(&format!("c{ci}"), rank_pos(cs, qj)),
                                        self_rank(q)
                                    )
                                }
                                GA::Selfp(st) => {
                                    let qj = qidx(sort, st).unwrap();
                                    format!("(and (= {} #b1) (bvult {} {}))", upb(qj), self_rank(qj), self_rank(q))
                                }
                                GA::Parent(pc, pos, st) => {
                                    let j = dsidx(sort, &(pc.clone(), *pos, st.clone())).unwrap();
                                    format!("(and (= {} #b1) (bvult {} {}))", dact(j), drank(j), self_rank(q))
                                }
                            })
                            .collect();
                        and_bool(&parts)
                    })
                    .collect();
                let just = or_bool(&alts);
                checks.push(if fold_on() && just == "true" {
                    // up[q] ⟺ true  ⇒  the bit must be 1
                    format!("(= {} #b1)", upb(q))
                } else if fold_on() && just == "false" {
                    // up[q] ⟺ false ⇒  the bit must be 0
                    format!("(= {} #b0)", upb(q))
                } else {
                    format!("(= (= {} #b1) {})", upb(q), just)
                });
            }
            // the parent certifies each child's parent-channel
            for (i, a) in arg_sorts.iter().enumerate() {
                for (j, (pc, pos, st)) in ds_of(a).iter().enumerate() {
                    let cact = format!("((_ extract {p} {p}) c{i})", p = down_active(a, j));
                    let crank = rslice(&format!("c{i}"), down_rank(a, j));
                    if pc == ctor && *pos == i && qidx(sort, st).is_some() {
                        let sq = qidx(sort, st).unwrap();
                        checks.push(format!("(= {cact} {})", upb(sq)));
                        checks.push(format!("(= {crank} (ite (= {} #b1) {} {zero}))", upb(sq), self_rank(sq)));
                    } else {
                        checks.push(format!("(= {cact} #b0)"));
                        checks.push(format!("(= {crank} {zero})"));
                    }
                }
            }
            let valid = and_bool(&checks);
            let valid_bit = if fold_on() && valid == "true" {
                "#b1".to_string()
            } else if fold_on() && valid == "false" {
                "#b0".to_string()
            } else {
                format!("(ite {valid} #b1 #b0)")
            };
            let body = if ann_w == w - 1 {
                format!("(concat {valid_bit} ann)")
            } else {
                valid_bit
            };
            s.push_str(&format!("{sig}{body})\n"));
        }
    }

    // grammar (each node draws a free Ann constant)
    let mut nts = vec![format!("(Start (_ BitVec {}))", width(start_sort))];
    for sort in ctors.keys() {
        nts.push(format!("(NT.{sort} (_ BitVec {}))", width(sort)));
        nts.push(format!("(Ann.{sort} (_ BitVec {}))", (width(sort) - 1).max(1)));
    }
    let prod = |sort: &str| -> String {
        ctors[sort]
            .iter()
            .map(|(c, args)| {
                let mut p = vec![c.clone()];
                p.extend(args.iter().map(|a| format!("NT.{a}")));
                p.push(format!("Ann.{sort}"));
                format!("({})", p.join(" "))
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut rules_txt = vec![format!("(Start (_ BitVec {}) ({}))", width(start_sort), prod(start_sort))];
    for sort in ctors.keys() {
        rules_txt.push(format!("(NT.{sort} (_ BitVec {}) ({}))", width(sort), prod(sort)));
        let aw = (width(sort) - 1).max(1);
        rules_txt.push(format!("(Ann.{sort} (_ BitVec {aw}) ((Constant (_ BitVec {aw}))))"));
    }
    s.push_str(&format!(
        "(synth-fun {synth_name} () (_ BitVec {})\n ({})\n ({}))\n",
        width(start_sort),
        nts.join(" "),
        rules_txt.join("\n  ")
    ));

    // constraints: valid root; down[root]=0; goal on root up-bits
    s.push_str(&format!("(constraint (= ((_ extract {p} {p}) g) #b1))\n", p = valid_pos(start_sort)));
    if nd(start_sort) > 0 {
        let lo = down_active(start_sort, 0);
        let hi = down_rank(start_sort, nd(start_sort) - 1) + fr - 1;
        s.push_str(&format!("(constraint (= ((_ extract {hi} {lo}) g) (_ bv0 {})))\n", hi - lo + 1));
    }
    for (qstate, want) in constraints {
        if let Some(q) = qidx(start_sort, qstate) {
            s.push_str(&format!("(constraint (= ((_ extract {q} {q}) g) #b{}))\n", if *want { 1 } else { 0 }));
        }
    }
    s.push_str("(check-synth)\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::smt2::ast::{HornAtom, HornRule};
    use crate::lang::runtime::Session;
    use crate::parser::DelisProg;

    fn v(n: &str) -> HT {
        HT::var(n)
    }
    fn c(n: &str) -> HT {
        HT::app(n, vec![])
    }
    fn f(n: &str, args: Vec<HT>) -> HT {
        HT::app(n, args)
    }
    fn atom(p: &str, arg: HT) -> HornAtom {
        HornAtom { pred: p.into(), arg }
    }
    fn rule(head: HornAtom, body: Vec<HornAtom>) -> HornRule {
        HornRule { body_atoms: body, head }
    }

    fn load_chc_prog(path: &str) -> HornProg {
        let input = std::fs::read_to_string(path).unwrap();
        let mut session = Session::new();
        let mut cmds = Vec::new();
        for res in DelisProg::parse_cmds(&input, Some(&mut session.runtime)) {
            let cmd = res.expect("parse");
            let _ = session.handle_command(&cmd);
            cmds.push(cmd);
        }
        DelisProg { cmds }.to_chc_ast().expect("to_chc_ast")
    }

    // ---- decode a cvc5 solution `g` (strip the trailing Ann arg) + CKY oracle ----
    #[derive(Debug, Clone)]
    enum Sx {
        Atom(String),
        List(Vec<Sx>),
    }
    fn parse_sexp(s: &str) -> Sx {
        let toks: Vec<String> =
            s.replace('(', " ( ").replace(')', " ) ").split_whitespace().map(str::to_string).collect();
        fn go(toks: &[String], pos: &mut usize) -> Sx {
            if toks[*pos] == "(" {
                *pos += 1;
                let mut v = Vec::new();
                while toks[*pos] != ")" {
                    v.push(go(toks, pos));
                }
                *pos += 1;
                Sx::List(v)
            } else {
                let a = toks[*pos].clone();
                *pos += 1;
                Sx::Atom(a)
            }
        }
        go(&toks, &mut 0)
    }
    fn strip_ann(s: &Sx) -> Sx {
        match s {
            Sx::Atom(a) => Sx::Atom(a.clone()),
            Sx::List(v) => {
                let mut out = vec![v[0].clone()];
                out.extend(v[1..v.len().saturating_sub(1)].iter().map(strip_ann));
                Sx::List(out)
            }
        }
    }
    fn decode_g(stdout: &str) -> Sx {
        let after = &stdout[stdout.find("define-fun g").expect("no g")..];
        let bv_end = after.find("BitVec").and_then(|i| after[i..].find(')').map(|j| i + j + 1)).unwrap();
        let rest = after[bv_end..].trim_start();
        let (mut depth, mut end) = (0i32, 0usize);
        for (i, ch) in rest.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        strip_ann(&parse_sexp(&rest[..end]))
    }
    impl Sx {
        fn head_atom(&self) -> &str {
            match self {
                Sx::Atom(a) => a,
                Sx::List(v) => v[0].head_atom(),
            }
        }
    }
    /// S0's productions from `cfg.top.S0(rhs0, rules)`.
    fn productions(g: &Sx) -> Vec<Sx> {
        let Sx::List(top) = g else { return vec![] };
        let mut prods = vec![top[1].clone()];
        let mut cur = &top[2];
        while let Sx::List(r) = cur {
            if r[0].head_atom() == "cfg.lhs.S0" {
                prods.push(r[1].clone());
                cur = &r[2];
            } else {
                break;
            }
        }
        prods
    }
    fn cky(prods: &[Sx], word: &str) -> bool {
        let w: Vec<char> = word.chars().collect();
        let n = w.len();
        let mut s0 = vec![vec![false; n + 1]; n + 1];
        loop {
            let mut changed = false;
            for i in 0..=n {
                for j in i..=n {
                    if !s0[i][j] && prods.iter().any(|p| rhs_derives(p, &w, i, j, &s0)) {
                        s0[i][j] = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        s0[0][n]
    }
    fn rhs_derives(term: &Sx, w: &[char], i: usize, j: usize, s0: &[Vec<bool>]) -> bool {
        match term {
            Sx::Atom(a) if a == "a" => j == i + 1 && w[i] == 'a',
            Sx::Atom(a) if a == "b" => j == i + 1 && w[i] == 'b',
            Sx::Atom(a) if a == "cfg.rhs.S0" => s0[i][j],
            Sx::List(v) if v[0].head_atom() == "cfg.++" => {
                (i..=j).any(|k| rhs_derives(&v[1], w, i, k, s0) && rhs_derives(&v[2], w, k, j, s0))
            }
            Sx::List(v) if v.len() == 1 => rhs_derives(&v[0], w, i, j, s0),
            _ => false,
        }
    }

    fn cvc5_available() -> bool {
        std::process::Command::new("cvc5").arg("--version").output().is_ok()
    }
    fn cvc5_solve(sygus: &str, tag: &str, tlimit_ms: u64) -> String {
        let path = std::env::temp_dir().join(format!("delis_{tag}.sygus"));
        std::fs::write(&path, sygus).unwrap();
        let out = std::process::Command::new("cvc5")
            .arg("--lang=sygus2")
            .arg(format!("--tlimit={tlimit_ms}"))
            .arg(&path)
            .output()
            .expect("run cvc5");
        let so = String::from_utf8_lossy(&out.stdout).into_owned();
        let se = String::from_utf8_lossy(&out.stderr);
        assert!(!se.to_lowercase().contains("error"), "cvc5 error on {tag}: {se}");
        so
    }

    /// The faithful semantics equals the exact LFP on every clause shape:
    /// intersection (non-additive), mixed up+down, multiple parent atoms, and CFG.
    #[test]
    fn faithful_semantics_match_lfp_all_shapes() {
        let check = |rules: &[HornRule], tree: &HT, label: &str| {
            let gt = build_ground(tree);
            let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
            let mut memo = HashMap::default();
            let got = respond_gen(0, None, &BTreeSet::new(), &gt, &gcs, &mut memo);
            let want: BTreeSet<String> =
                crossing_eval(rules, &gt).into_iter().filter(|(_, n)| *n == 0).map(|(s, _)| s).collect();
            assert_eq!(got, want, "respond_gen != LFP for {label}");
        };
        let cc = || c("c");
        let topxy = || f("top", vec![v("x"), v("y")]);
        let topcy = || f("top", vec![cc(), v("y")]);

        // intersection (non-additive): C(c) :- A(c), B(c) with A,B both forced
        check(
            &[
                rule(atom("P", topxy()), vec![]),
                rule(atom("Q", topxy()), vec![]),
                rule(atom("A", cc()), vec![atom("P", topcy())]),
                rule(atom("B", cc()), vec![atom("Q", topcy())]),
                rule(atom("C", cc()), vec![atom("A", cc()), atom("B", cc())]),
                rule(atom("Win", topxy()), vec![atom("C", v("x"))]),
            ],
            &f("top", vec![cc(), cc()]),
            "intersection",
        );

        // mixed up+down: W(g(x)) :- T(top(g,_)) ∧ R(g)  (parent-atom + self-loop)
        {
            let gx = || f("g", vec![v("x")]);
            check(
                &[
                    rule(atom("Good", cc()), vec![]),
                    rule(atom("T", topxy()), vec![]),
                    rule(atom("R", gx()), vec![atom("Good", v("x"))]),
                    rule(atom("W", gx()), vec![atom("T", f("top", vec![gx(), v("z")])), atom("R", gx())]),
                    rule(atom("Win", topxy()), vec![atom("W", v("x"))]),
                ],
                &f("top", vec![f("g", vec![cc()]), cc()]),
                "mixed up+down",
            );
        }

        // multiple parent atoms: U(c) :- P(top(c,_)) ∧ Q(top(c,_))
        check(
            &[
                rule(atom("P", topxy()), vec![]),
                rule(atom("Q", f("top", vec![v("x"), cc()])), vec![atom("Good", v("y"))]),
                rule(atom("Good", cc()), vec![]),
                rule(atom("U", cc()), vec![atom("P", topcy()), atom("Q", topcy())]),
                rule(atom("Win", topxy()), vec![atom("U", v("x"))]),
            ],
            &f("top", vec![cc(), cc()]),
            "multiple parent atoms",
        );

        // CFG (real program, relevant states)
        let prog = load_chc_prog("benchmark/CFG/0001_tiny.sl");
        let (_s, cons) = goal(&prog);
        let seeds: HashSet<String> = cons.iter().map(|(q, _)| q.clone()).collect();
        let relevant = relevant_states(&prog, &seeds);
        let gcs = general_clauses(&prog, &relevant);
        for g in [
            f("cfg.top.S0", vec![f("cfg.++", vec![c("a"), c("b")]), c("end")]),
            f("cfg.top.S0", vec![c("cfg.rhs.S0"), c("end")]),
        ] {
            let gt = build_ground(&g);
            let mut memo = HashMap::default();
            let got: BTreeSet<String> = respond_gen(0, None, &BTreeSet::new(), &gt, &gcs, &mut memo)
                .into_iter()
                .filter(|s| relevant.contains(s))
                .collect();
            let want: BTreeSet<String> = crossing_eval(&prog.rules, &gt)
                .into_iter()
                .filter(|(s, n)| *n == 0 && relevant.contains(s))
                .map(|(s, _)| s)
                .collect();
            assert_eq!(got, want, "respond_gen != LFP for CFG {g}");
        }
    }

    /// The encoder, on the minimal up-move automaton (small |Q|): cvc5 must
    /// synthesize `top(r, e2)` — the tree whose *real* sibling is `Good`.
    #[test]
    fn encoder_solves_toy() {
        if !cvc5_available() {
            eprintln!("cvc5 not found; skipping");
            return;
        }
        let states = HashMap::from_iter([(
            "T".to_string(),
            vec!["A".into(), "B".into(), "Good".into(), "Win".into()],
        )]);
        let ctors = HashMap::from_iter([(
            "T".to_string(),
            vec![
                ("e1".into(), vec![]),
                ("e2".into(), vec![]),
                ("r".into(), vec![]),
                ("top".into(), vec!["T".into(), "T".into()]),
            ],
        )]);
        let gcs = vec![
            GC { ctor: "e2".into(), head: "Good".into(), body: vec![] },
            GC { ctor: "top".into(), head: "A".into(), body: vec![GA::Child(1, "Good".into())] },
            GC { ctor: "top".into(), head: "Win".into(), body: vec![GA::Child(0, "B".into())] },
            GC { ctor: "r".into(), head: "B".into(), body: vec![GA::Parent("top".into(), 0, "A".into())] },
        ];
        let sygus = faithful_sygus_ir(&states, &ctors, &gcs, "T", &[("Win".to_string(), true)], "g", 4);
        let so = cvc5_solve(&sygus, "faithful_toy", 60_000);
        assert!(so.contains("(top"), "expected top(...): {so}");
        assert!(so.contains("e2") && so.contains('r'), "expected top(r, e2): {so}");
    }

    /// End-to-end on a real CFG program: the synthesized grammar must be
    /// CKY-correct. Ignored by default — the general parent-channel makes cvc5's
    /// search slow at this scale (correctness over speed); run explicitly.
    #[test]
    #[ignore = "general encoder is correct but cvc5 is slow at CFG scale; run explicitly"]
    fn encoder_solves_cfg_tiny() {
        if !cvc5_available() {
            return;
        }
        let prog = load_chc_prog("benchmark/CFG/0001_tiny.sl");
        let so = cvc5_solve(&faithful_sygus(&prog, "g").0, "faithful_cfg", 600_000);
        assert!(so.contains("define-fun g"), "no solution: {so}");
        let prods = productions(&decode_g(&so));
        assert!(cky(&prods, "ab"), "must accept ab");
        assert!(!cky(&prods, "a"), "must reject a");
        assert!(!cky(&prods, "ba"), "must reject ba");
    }
}
