//! Sound & complete compilation of delis programs **with `parent` (up-moves)**
//! into SyGuS-BV, via the faithful two-way → one-way crossing construction
//! (Shepherdson 1959; Vardi/Cachat, as cited by Krogmeier–Madhusudan 2023).
//!
//! The semantics of a FACET program is a two-way alternating tree automaton over
//! the candidate syntax tree, accepted by a least fixpoint (reachability). The
//! one-way encoding here is a **crossing-relation annotation certificate**: each
//! node carries guessed `up[state]`, a same-node support relation `R ⊆ Q×Q`, a
//! parent-channel (`dact`/`ddem`) and child→parent demand (`udem`), and clause
//! `pick` bits. A computed `valid` bit checks, for every clause uniformly
//! (child-ref / self-loop / parent-atom), that `up` is justified and that `R` is
//! the chosen support — irreflexive and transitively closed, i.e. acyclic, hence
//! the least fixpoint. `R` summarizes the up/down crossings (Shepherdson 1959;
//! Vardi 1998, specialized to no-parity since delis is monotone LFP), so its
//! width depends only on `|Q|`, **never on tree size** — the encoding is sound and
//! complete for unbounded trees and handles intersection, mixed up+down clauses,
//! and multiple parent atoms (arbitrary delis programs, not just CFG).
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
//  Encoder: crossing-relation annotation certificate → SyGuS-BV
// ===========================================================================

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

// --- small SMT-BV string builders for the crossing-relation encoding ---
fn bit(term: &str, k: usize) -> String {
    format!("((_ extract {k} {k}) {term})")
}
fn slice(term: &str, lo: usize, hi: usize) -> String {
    format!("((_ extract {hi} {lo}) {term})")
}
fn bv0(w: usize) -> String {
    format!("(_ bv0 {w})")
}
/// The `w`-bit set `{q}` (bit `q` set). `q < w ≤ 127`.
fn singleton(q: usize, w: usize) -> String {
    format!("(_ bv{} {w})", 1u128 << q)
}
/// `bvor` of `w`-bit masks; `[]`⇒`bv0`, `[x]`⇒`x`.
fn bvor_all(xs: &[String], w: usize) -> String {
    match xs {
        [] => bv0(w),
        [one] => one.clone(),
        _ => format!("(bvor {})", xs.join(" ")),
    }
}
/// `mask ⊆ row` as `w`-bit sets: `(bvand mask ¬row) = 0`.
fn subset(mask: &str, row: &str, w: usize) -> String {
    format!("(= (bvand {mask} (bvnot {row})) {})", bv0(w))
}

/// Encode a compiled CHC program, naming the synth-fun `synth_name` (the source
/// program's synth-fun name). Returns `(sygus-text, synth-target sort)`.
pub(crate) fn faithful_sygus(prog: &HornProg, synth_name: &str) -> (String, String) {
    let (start, constraints) = goal(prog);
    let seeds: HashSet<String> = constraints.iter().map(|(q, _)| q.clone()).collect();
    let relevant = relevant_states(prog, &seeds);
    let (states, ctors) = build_sig(prog, &relevant);
    let gcs = general_clauses(prog, &relevant);
    let sygus = faithful_sygus_ir(&states, &ctors, &gcs, &start, &constraints, synth_name);
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

/// Per-sort node layout + the local `node_checks` certificate, shared by the
/// SyGuS emitter and the fixed-tree validation oracle. Node layout per sort `s`
/// (`Q=|states|`, `D=|DS|`, `P=#clauses producing s`):
///
/// `[ up:Q ][ R:Q² ][ dact:D ][ ddem:D·Q ][ udem:Q·D ][ pick:P ][ valid:1 ]`
///
/// - `up[q]`            — guessed acceptance: state `q` holds at this node.
/// - `R[p][q]`          — same-node support relation: `up[p]` (transitively)
///   depends on `up[q]` at this node. Required irreflexive + transitively
///   closed ⇒ acyclic support ⇒ least fixpoint. This replaces the old numeric
///   rank; it is the Shepherdson/Vardi crossing relation (no parity, since delis
///   is monotone LFP). Width depends only on `|Q|`, never on tree size.
/// - `dact[j]`/`ddem[j]` — parent-channel for `DS` triple `j=(pc,pos,st)`:
///   `dact` = "my parent is `pc`, I'm at `pos`, parent has `st`" (certified by
///   the parent = parent's `up[st]`); `ddem` (over this node's `Q`) = the set of
///   *my* states that `st@parent`'s support routes back to (parent→child demand).
/// - `udem[Y][j]`       — child→parent demand: my state `Y` routes up to `st@parent`
///   for triple `j`. Computed here (I see my own children), read by my parent.
/// - `pick[k]`          — which clause founds each `up`; makes the chosen
///   derivation unambiguous so `R` is the chosen support, not an over-approx.
struct Enc<'a> {
    states: &'a HashMap<String, Vec<String>>,
    gcs: &'a [GC],
    ds: HashMap<String, Vec<(String, usize, String)>>,
    /// per sort: indices into `gcs` of clauses producing that sort (pick order).
    sort_clauses: HashMap<String, Vec<usize>>,
    /// reflexive-transitive closure of the predicate dependency graph
    /// (`Y → Z` iff `Z` appears in some premise of a clause for `Y`). Over-
    /// approximates which states can transitively support which: if `Z ∉ reach[Y]`
    /// then `R[Y][Z]`/`udem` involving them are statically 0, so the emitter skips
    /// those `(and …)` terms (sound — they can never be true).
    reach: HashMap<String, HashSet<String>>,
    /// per sort, the `(state index, DS triple index)` pairs that can route up
    /// (precomputed `can_route_up`), so the hot pruning checks are O(1) lookups.
    route: HashMap<String, HashSet<(usize, usize)>>,
}

impl<'a> Enc<'a> {
    fn new(
        states: &'a HashMap<String, Vec<String>>,
        ctors: &'a HashMap<String, Vec<(String, Vec<String>)>>,
        gcs: &'a [GC],
    ) -> Self {
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
        let mut sort_clauses: HashMap<String, Vec<usize>> = HashMap::default();
        for sort in ctors.keys() {
            sort_clauses.entry(sort.clone()).or_default();
        }
        for (k, gc) in gcs.iter().enumerate() {
            if let Some(sort) = ctor_sort.get(&gc.ctor) {
                sort_clauses.get_mut(sort).expect("sort registered").push(k);
            }
        }
        // dependency successors, then reflexive-transitive closure per state.
        let mut succ: HashMap<String, HashSet<String>> = HashMap::default();
        for gc in gcs {
            let e = succ.entry(gc.head.clone()).or_default();
            for a in &gc.body {
                let (GA::Child(_, s) | GA::Selfp(s) | GA::Parent(_, _, s)) = a;
                e.insert(s.clone());
            }
        }
        let mut reach: HashMap<String, HashSet<String>> = HashMap::default();
        for st in states.values().flatten() {
            let mut seen: HashSet<String> = HashSet::default();
            seen.insert(st.clone());
            let mut stack = vec![st.clone()];
            while let Some(x) = stack.pop() {
                if let Some(ns) = succ.get(&x) {
                    for n in ns {
                        if seen.insert(n.clone()) {
                            stack.push(n.clone());
                        }
                    }
                }
            }
            reach.insert(st.clone(), seen);
        }
        // routability: `y` of `sort` can route up via triple `t` iff `y` reaches
        // (incl. itself) a `sort`-state whose clause directly uses `Parent(t)`.
        let empty_ds: Vec<(String, usize, String)> = Vec::new();
        let mut route: HashMap<String, HashSet<(usize, usize)>> = HashMap::default();
        for (sort, sts) in states {
            let triples = ds.get(sort).unwrap_or(&empty_ds);
            let clauses = sort_clauses.get(sort).map(Vec::as_slice).unwrap_or(&[]);
            let mut set: HashSet<(usize, usize)> = HashSet::default();
            for (t, trip) in triples.iter().enumerate() {
                let users: Vec<&String> = clauses
                    .iter()
                    .filter(|&&k| gcs[k].body.iter().any(|a| matches!(a, GA::Parent(p, po, s) if p == &trip.0 && *po == trip.1 && s == &trip.2)))
                    .map(|&k| &gcs[k].head)
                    .collect();
                if users.is_empty() {
                    continue;
                }
                for (y, yst) in sts.iter().enumerate() {
                    if reach.get(yst).is_some_and(|r| users.iter().any(|u| r.contains(*u))) {
                        set.insert((y, t));
                    }
                }
            }
            route.insert(sort.clone(), set);
        }
        Enc { states, gcs, ds, sort_clauses, reach, route }
    }

    /// Can `y` (state index of `sort`) route up via DS triple `t` in some candidate?
    fn routes_up(&self, sort: &str, y: usize, t: usize) -> bool {
        self.route.get(sort).is_some_and(|s| s.contains(&(y, t)))
    }
    /// Can `R[p][q]` ever be 1? Only if `q` is reachable from `p`.
    fn may_dep(&self, p: &str, q: &str) -> bool {
        self.reach.get(p).is_some_and(|r| r.contains(q))
    }

    fn nq(&self, s: &str) -> usize {
        self.states[s].len()
    }
    fn ds_of(&self, s: &str) -> &[(String, usize, String)] {
        self.ds.get(s).map(Vec::as_slice).unwrap_or(&[])
    }
    fn nd(&self, s: &str) -> usize {
        self.ds_of(s).len()
    }
    fn clauses(&self, s: &str) -> &[usize] {
        self.sort_clauses.get(s).map(Vec::as_slice).unwrap_or(&[])
    }
    fn np(&self, s: &str) -> usize {
        self.clauses(s).len()
    }
    fn qidx(&self, s: &str, st: &str) -> Option<usize> {
        self.states[s].iter().position(|x| x == st)
    }
    fn dsidx(&self, s: &str, e: &(String, usize, String)) -> Option<usize> {
        self.ds_of(s).iter().position(|x| x == e)
    }
    fn clause_local(&self, s: &str, k: usize) -> usize {
        self.clauses(s).iter().position(|&x| x == k).expect("clause of sort")
    }

    // --- layout offsets ---
    fn width(&self, s: &str) -> usize {
        let (q, d, p) = (self.nq(s), self.nd(s), self.np(s));
        q + q * q + d + d * q + q * d + p + 1
    }
    fn r_lo(&self, s: &str, p: usize) -> usize {
        self.nq(s) + p * self.nq(s)
    }
    fn r_pos(&self, s: &str, p: usize, q: usize) -> usize {
        self.r_lo(s, p) + q
    }
    fn r_row(&self, s: &str, term: &str, p: usize) -> String {
        slice(term, self.r_lo(s, p), self.r_lo(s, p) + self.nq(s) - 1)
    }
    fn dact_pos(&self, s: &str, j: usize) -> usize {
        let q = self.nq(s);
        q + q * q + j
    }
    fn ddem_lo(&self, s: &str, j: usize) -> usize {
        let (q, d) = (self.nq(s), self.nd(s));
        q + q * q + d + j * q
    }
    fn ddem_row(&self, s: &str, term: &str, j: usize) -> String {
        slice(term, self.ddem_lo(s, j), self.ddem_lo(s, j) + self.nq(s) - 1)
    }
    fn udem_pos(&self, s: &str, y: usize, t: usize) -> usize {
        let (q, d) = (self.nq(s), self.nd(s));
        q + q * q + d + d * q + y * d + t
    }
    fn pick_pos(&self, s: &str, k_local: usize) -> usize {
        let (q, d) = (self.nq(s), self.nd(s));
        q + q * q + d + d * q + q * d + k_local
    }
    fn valid_pos(&self, s: &str) -> usize {
        self.width(s) - 1
    }

    /// The local certificate for a node of ctor `ctor` (sort `sort`, children
    /// sorts `arg_sorts`), reading its own annotation from `ann` and child `i`'s
    /// full value from `kids[i]`. Returns `(lets, checks)`: `checks`' conjunction
    /// is the node's `valid` bit, and `lets` are `(name, width, expr)` shared
    /// sub-terms the checks reference (each state's per-child down-routing — bound
    /// once so the `D` `ddem` rows share it instead of re-emitting an `O(P)` scan
    /// `D` times). The SAME formulas drive the fixed-tree oracle. `pfx` namespaces
    /// the let names (the oracle flattens many nodes into one SMT scope).
    fn node_checks(
        &self,
        sort: &str,
        ctor: &str,
        arg_sorts: &[String],
        ann: &str,
        kids: &[String],
        pfx: &str,
    ) -> (Vec<(String, usize, String)>, Vec<String>) {
        let q_n = self.nq(sort);
        let mut out: Vec<String> = Vec::new();

        // (C0) every child is itself valid.
        for (i, a) in arg_sorts.iter().enumerate() {
            out.push(format!("(= {} #b1)", bit(&kids[i], self.valid_pos(a))));
        }

        // premise-activeness of a clause (premises currently hold).
        let prem_active = |gc: &GC| -> String {
            let parts: Vec<String> = gc
                .body
                .iter()
                .map(|atom| match atom {
                    GA::Child(ci, st) => {
                        let qj = self.qidx(&arg_sorts[*ci], st).unwrap();
                        format!("(= {} #b1)", bit(&kids[*ci], qj))
                    }
                    GA::Selfp(st) => format!("(= {} #b1)", bit(ann, self.qidx(sort, st).unwrap())),
                    GA::Parent(pc, pos, st) => {
                        let j = self.dsidx(sort, &(pc.clone(), *pos, st.clone())).unwrap();
                        format!("(= {} #b1)", bit(ann, self.dact_pos(sort, j)))
                    }
                })
                .collect();
            and_bool(&parts)
        };

        // same-node deps of a clause, as a Q_n-bit mask over this node's states.
        let deps_mask = |gc: &GC| -> String {
            let mut parts: Vec<String> = Vec::new();
            for atom in &gc.body {
                match atom {
                    GA::Selfp(st) => {
                        let qj = self.qidx(sort, st).unwrap();
                        parts.push(format!("(bvor {} {})", singleton(qj, q_n), self.r_row(sort, ann, qj)));
                    }
                    GA::Parent(pc, pos, st) => {
                        let j = self.dsidx(sort, &(pc.clone(), *pos, st.clone())).unwrap();
                        parts.push(self.ddem_row(sort, ann, j));
                    }
                    GA::Child(ci, st) => {
                        // states of THIS node that `st@child` routes back to,
                        // read from the child's `udem` (child→parent demand). Skip
                        // child triples `st` can't statically route up via (their
                        // `udem` is pinned 0, so the term is always-zero).
                        let a = &arg_sorts[*ci];
                        let qa = self.qidx(a, st).unwrap();
                        let mut terms: Vec<String> = Vec::new();
                        for (tj, trip) in self.ds_of(a).iter().enumerate() {
                            let (pc2, pos2, st2) = trip;
                            if pc2 == ctor && *pos2 == *ci && self.routes_up(a, qa, tj) {
                                if let Some(sj) = self.qidx(sort, st2) {
                                    terms.push(format!(
                                        "(ite (= {} #b1) {} {})",
                                        bit(&kids[*ci], self.udem_pos(a, qa, tj)),
                                        singleton(sj, q_n),
                                        bv0(q_n)
                                    ));
                                }
                            }
                        }
                        parts.push(bvor_all(&terms, q_n));
                    }
                }
            }
            bvor_all(&parts, q_n)
        };

        // (Cup) up[q] ⟺ some clause for (ctor, q) is picked (foundedness, via
        // pick ⇒ active ∧ deps⊆R below); plus *closure*: if any clause for q is
        // active, up[q] must hold — without this the solver could choose a sound
        // but non-least sub-fixpoint (set derivable states to 0). The two together
        // pin the least fixpoint. (R irreflexive.)
        for (q, qst) in self.states[sort].iter().enumerate() {
            let mine: Vec<&usize> = self
                .clauses(sort)
                .iter()
                .filter(|&&k| &self.gcs[k].ctor == ctor && &self.gcs[k].head == qst)
                .collect();
            let picks: Vec<String> =
                mine.iter().map(|&&k| format!("(= {} #b1)", bit(ann, self.pick_pos(sort, self.clause_local(sort, k))))).collect();
            out.push(format!("(= (= {} #b1) {})", bit(ann, q), or_bool(&picks)));
            let actives: Vec<String> = mine.iter().map(|&&k| prem_active(&self.gcs[k])).collect();
            if !actives.is_empty() {
                out.push(format!("(=> {} (= {} #b1))", or_bool(&actives), bit(ann, q)));
            }
            out.push(format!("(= {} #b0)", bit(ann, self.r_pos(sort, q, q))));
        }

        // (Cpick) a picked clause must be active and contribute its deps to R[head];
        // clauses of other ctors cannot be picked here.
        for &k in self.clauses(sort) {
            let gc = &self.gcs[k];
            let pbit = bit(ann, self.pick_pos(sort, self.clause_local(sort, k)));
            if gc.ctor == ctor {
                let head = self.qidx(sort, &gc.head).unwrap();
                let mut conj = vec![prem_active(gc)];
                conj.push(subset(&deps_mask(gc), &self.r_row(sort, ann, head), q_n));
                out.push(format!("(=> (= {pbit} #b1) {})", and_bool(&conj)));
            } else {
                out.push(format!("(= {pbit} #b0)"));
            }
        }

        // (Rtrans) R[p][q] ⇒ R[q]-row ⊆ R[p]-row (transitive closure). Skip pairs
        // where `q` is statically unreachable from `p` (`R[p][q]` is then 0 in the
        // least solution; a spurious 1 only adds cycle risk, never hides one).
        for p in 0..q_n {
            for q in 0..q_n {
                if p == q || !self.may_dep(&self.states[sort][p], &self.states[sort][q]) {
                    continue;
                }
                out.push(format!(
                    "(=> (= {} #b1) {})",
                    bit(ann, self.r_pos(sort, p, q)),
                    subset(&self.r_row(sort, ann, q), &self.r_row(sort, ann, p), q_n)
                ));
            }
        }

        // (Cudem) udem[Y][t] ⟺ Y directly routes up via t (picked Parent clause)
        // or Y depends (R) on some Y' that routes up via t. Statically-impossible
        // routings are pinned to 0; closure terms emitted only for reachable Y'
        // that can themselves route up via t (the bulk of the `(and …)` prune).
        for y in 0..q_n {
            let yst = &self.states[sort][y];
            for (t, trip) in self.ds_of(sort).iter().enumerate() {
                let (pc, pos, st) = trip;
                if !self.routes_up(sort, y, t) {
                    out.push(format!("(= {} #b0)", bit(ann, self.udem_pos(sort, y, t))));
                    continue;
                }
                let mut terms: Vec<String> = Vec::new();
                for &k in self.clauses(sort) {
                    let gc = &self.gcs[k];
                    if gc.ctor == ctor
                        && self.qidx(sort, &gc.head) == Some(y)
                        && gc.body.iter().any(|a| matches!(a, GA::Parent(p2, po2, s2) if p2 == pc && po2 == pos && s2 == st))
                    {
                        terms.push(format!("(= {} #b1)", bit(ann, self.pick_pos(sort, self.clause_local(sort, k)))));
                    }
                }
                for yp in 0..q_n {
                    let ypst = &self.states[sort][yp];
                    if yp == y || !self.may_dep(yst, ypst) || !self.routes_up(sort, yp, t) {
                        continue;
                    }
                    terms.push(format!(
                        "(and (= {} #b1) (= {} #b1))",
                        bit(ann, self.r_pos(sort, y, yp)),
                        bit(ann, self.udem_pos(sort, yp, t))
                    ));
                }
                out.push(format!("(= (= {} #b1) {})", bit(ann, self.udem_pos(sort, y, t)), or_bool(&terms)));
            }
        }

        // (Cchannel) the parent certifies each child's dact + ddem. First bind, per
        // child `i` and self-state `p`, the down-routing of `p` into child `i`
        // (`p`'s picked `Child(i,z)` deps) — independent of the triple, so the `D`
        // ddem rows reference the shared let var instead of re-scanning `P` clauses.
        let mut lets: Vec<(String, usize, String)> = Vec::new();
        let mut dr: HashMap<(usize, usize), String> = HashMap::default();
        for (i, a) in arg_sorts.iter().enumerate() {
            let q_a = self.nq(a);
            for p in 0..q_n {
                let mut dd_terms: Vec<String> = Vec::new();
                for &k in self.clauses(sort) {
                    let gc = &self.gcs[k];
                    if gc.ctor == ctor && self.qidx(sort, &gc.head) == Some(p) {
                        let z_terms: Vec<String> = gc
                            .body
                            .iter()
                            .filter_map(|atom| match atom {
                                GA::Child(ci2, z) if *ci2 == i => Some(singleton(self.qidx(a, z).unwrap(), q_a)),
                                _ => None,
                            })
                            .collect();
                        if !z_terms.is_empty() {
                            dd_terms.push(format!(
                                "(ite (= {} #b1) {} {})",
                                bit(ann, self.pick_pos(sort, self.clause_local(sort, k))),
                                bvor_all(&z_terms, q_a),
                                bv0(q_a)
                            ));
                        }
                    }
                }
                if !dd_terms.is_empty() {
                    let name = format!("{pfx}dr_{i}_{p}");
                    lets.push((name.clone(), q_a, bvor_all(&dd_terms, q_a)));
                    dr.insert((i, p), name);
                }
            }
        }
        for (i, a) in arg_sorts.iter().enumerate() {
            let q_a = self.nq(a);
            for (j, (pc, pos, st)) in self.ds_of(a).iter().enumerate() {
                let cact = bit(&kids[i], self.dact_pos(a, j));
                let cddem = self.ddem_row(a, &kids[i], j);
                if pc == ctor && *pos == i && self.qidx(sort, st).is_some() {
                    let sq = self.qidx(sort, st).unwrap();
                    out.push(format!("(= {cact} {})", bit(ann, sq)));
                    // ddem = ⋃ over self-states p ∈ {sq}∪R[sq] of p's down-routing dr[i][p].
                    let p_terms: Vec<String> = (0..q_n)
                        .filter_map(|p| {
                            let name = dr.get(&(i, p))?;
                            let guard = if p == sq { "true".to_string() } else { format!("(= {} #b1)", bit(ann, self.r_pos(sort, sq, p))) };
                            Some(format!("(ite {guard} {name} {})", bv0(q_a)))
                        })
                        .collect();
                    out.push(format!("(= {cddem} {})", bvor_all(&p_terms, q_a)));
                } else {
                    out.push(format!("(= {cact} #b0)"));
                    out.push(format!("(= {cddem} {})", bv0(q_a)));
                }
            }
        }
        (lets, out)
    }
}

/// The encoder, decoupled from `HornProg`. See [`Enc`] for the node layout.
fn faithful_sygus_ir(
    states: &HashMap<String, Vec<String>>,
    ctors: &HashMap<String, Vec<(String, Vec<String>)>>,
    gcs: &[GC],
    start_sort: &str,
    constraints: &[(String, bool)],
    synth_name: &str,
) -> String {
    let e = Enc::new(states, ctors, gcs);
    let mut s = String::from("(set-logic BV)\n");

    for (sort, ctor_list) in ctors {
        let w = e.width(sort);
        for (ctor, arg_sorts) in ctor_list {
            let mut sig = format!("(define-fun {ctor} (");
            for (i, a) in arg_sorts.iter().enumerate() {
                sig.push_str(&format!("(c{i} (_ BitVec {})) ", e.width(a)));
            }
            let ann_w = (w - 1).max(1);
            sig.push_str(&format!("(ann (_ BitVec {ann_w}))) (_ BitVec {w}) "));
            let kids: Vec<String> = (0..arg_sorts.len()).map(|i| format!("c{i}")).collect();
            let (lets, checks) = e.node_checks(sort, ctor, arg_sorts, "ann", &kids, "");
            let valid = and_bool(&checks);
            let valid_bit = if fold_on() && valid == "true" {
                "#b1".to_string()
            } else if fold_on() && valid == "false" {
                "#b0".to_string()
            } else if lets.is_empty() {
                format!("(ite {valid} #b1 #b0)")
            } else {
                let bl = lets.iter().map(|(n, _w, x)| format!("({n} {x})")).collect::<Vec<_>>().join(" ");
                format!("(let ({bl}) (ite {valid} #b1 #b0))")
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
    let mut nts = vec![format!("(Start (_ BitVec {}))", e.width(start_sort))];
    for sort in ctors.keys() {
        nts.push(format!("(NT.{sort} (_ BitVec {}))", e.width(sort)));
        nts.push(format!("(Ann.{sort} (_ BitVec {}))", (e.width(sort) - 1).max(1)));
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
    let mut rules_txt = vec![format!("(Start (_ BitVec {}) ({}))", e.width(start_sort), prod(start_sort))];
    for sort in ctors.keys() {
        rules_txt.push(format!("(NT.{sort} (_ BitVec {}) ({}))", e.width(sort), prod(sort)));
        let aw = (e.width(sort) - 1).max(1);
        rules_txt.push(format!("(Ann.{sort} (_ BitVec {aw}) ((Constant (_ BitVec {aw}))))"));
    }
    s.push_str(&format!(
        "(synth-fun {synth_name} () (_ BitVec {})\n ({})\n ({}))\n",
        e.width(start_sort),
        nts.join(" "),
        rules_txt.join("\n  ")
    ));

    // constraints: valid root; the root has no parent ⇒ its channel/up-demand
    // (dact, ddem, udem) are all zero; goal on root up-bits.
    s.push_str(&format!("(constraint (= {} #b1))\n", bit(synth_name, e.valid_pos(start_sort))));
    let d = e.nd(start_sort);
    if d > 0 {
        let q = e.nq(start_sort);
        let lo = e.dact_pos(start_sort, 0);
        let hi = e.udem_pos(start_sort, q - 1, d - 1);
        s.push_str(&format!("(constraint (= {} {}))\n", slice(synth_name, lo, hi), bv0(hi - lo + 1)));
    }
    for (qstate, want) in constraints {
        if let Some(q) = e.qidx(start_sort, qstate) {
            s.push_str(&format!("(constraint (= {} #b{}))\n", bit(synth_name, q), if *want { 1 } else { 0 }));
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

    /// Run a plain `QF_BV` `(check-sat)` and return whether it is `sat`.
    fn smt_sat(smt2: &str, tag: &str) -> bool {
        let path = std::env::temp_dir().join(format!("delis_{tag}.smt2"));
        std::fs::write(&path, smt2).unwrap();
        let out = std::process::Command::new("cvc5").arg("--lang=smt2").arg(&path).output().expect("run cvc5");
        let so = String::from_utf8_lossy(&out.stdout);
        let t = so.trim();
        if t.starts_with("unsat") {
            false
        } else if t.starts_with("sat") {
            true
        } else {
            panic!("cvc5 on {tag}: stdout={t:?} stderr={:?}", String::from_utf8_lossy(&out.stderr))
        }
    }

    /// A single-sort signature ("T") from abstract rules + a ground tree:
    /// `ctors` with arities (all args sort "T"), `states` = all predicate names.
    fn single_sort_sig(
        rules: &[HornRule],
        tree: &HT,
    ) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<(String, Vec<String>)>>) {
        fn walk(t: &HT, ar: &mut std::collections::BTreeMap<String, usize>) {
            if let HT::App { fun, args } = t {
                ar.insert(fun.clone(), args.len());
                for a in args {
                    walk(a, ar);
                }
            }
        }
        let mut ar = std::collections::BTreeMap::new();
        for r in rules {
            walk(&r.head.arg, &mut ar);
            for b in &r.body_atoms {
                walk(&b.arg, &mut ar);
            }
        }
        walk(tree, &mut ar);
        let cs: Vec<(String, Vec<String>)> = ar.iter().map(|(f, n)| (f.clone(), vec!["T".to_string(); *n])).collect();
        let mut ctors = HashMap::default();
        ctors.insert("T".to_string(), cs);
        let mut preds = std::collections::BTreeSet::new();
        for r in rules {
            preds.insert(r.head.pred.clone());
            for b in &r.body_atoms {
                preds.insert(b.pred.clone());
            }
        }
        let mut states = HashMap::default();
        states.insert("T".to_string(), preds.into_iter().collect::<Vec<_>>());
        (states, ctors)
    }

    /// Partial-evaluate the node certificate on a FIXED ground tree: declare each
    /// node's annotation as a free const, assert `node_checks` for every node plus
    /// the root constraints, and report whether SOME annotation satisfies them.
    /// By soundness+completeness this is `sat` iff `crossing_eval` accepts the goal.
    fn local_check_sat(
        states: &HashMap<String, Vec<String>>,
        ctors: &HashMap<String, Vec<(String, Vec<String>)>>,
        gcs: &[GC],
        constraints: &[(String, bool)],
        gt: &Ground,
    ) -> bool {
        let e = Enc::new(states, ctors, gcs);
        let sort = "T";
        let w = e.width(sort);
        let n_nodes = gt.ctor.len();
        let mut smt = String::from("(set-logic QF_BV)\n");
        for n in 0..n_nodes {
            smt.push_str(&format!("(declare-const n{n} (_ BitVec {w}))\n"));
        }
        for n in 0..n_nodes {
            let ctor = &gt.ctor[n];
            let arg_sorts = &ctors[sort].iter().find(|(c, _)| c == ctor).expect("ctor in sig").1;
            let kids: Vec<String> = gt.children[n].iter().map(|c| format!("n{c}")).collect();
            let (lets, checks) = e.node_checks(sort, ctor, arg_sorts, &format!("n{n}"), &kids, &format!("d{n}_"));
            for (name, bw, expr) in &lets {
                smt.push_str(&format!("(declare-const {name} (_ BitVec {bw}))\n(assert (= {name} {expr}))\n"));
            }
            for chk in &checks {
                smt.push_str(&format!("(assert {chk})\n"));
            }
        }
        let d = e.nd(sort);
        if d > 0 {
            let q = e.nq(sort);
            let lo = e.dact_pos(sort, 0);
            let hi = e.udem_pos(sort, q - 1, d - 1);
            smt.push_str(&format!("(assert (= {} {}))\n", slice("n0", lo, hi), bv0(hi - lo + 1)));
        }
        for (qst, want) in constraints {
            if let Some(q) = e.qidx(sort, qst) {
                smt.push_str(&format!("(assert (= {} #b{}))\n", bit("n0", q), if *want { 1 } else { 0 }));
            }
        }
        smt.push_str("(check-sat)\n");
        smt_sat(&smt, "oracle")
    }

    /// The crossing-relation certificate, partially evaluated on a fixed tree, is
    /// `sat` iff the exact LFP (`crossing_eval`) accepts — across all clause
    /// shapes, an unfounded cycle (rejected), and DEEP trees (no tree bound).
    #[test]
    fn local_checks_match_lfp_and_deep() {
        if !cvc5_available() {
            eprintln!("cvc5 not found; skipping");
            return;
        }
        let check = |rules: &[HornRule], tree: &HT, cons: &[(String, bool)], label: &str| {
            let (states, ctors) = single_sort_sig(rules, tree);
            let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
            let gt = build_ground(tree);
            let holds = crossing_eval(rules, &gt);
            let want = cons.iter().all(|(q, w)| holds.contains(&(q.clone(), 0)) == *w);
            let got = local_check_sat(&states, &ctors, &gcs, cons, &gt);
            let at0: Vec<&String> = holds.iter().filter(|(_, n)| *n == 0).map(|(s, _)| s).collect();
            assert_eq!(got, want, "oracle({label}) got={got} want={want}; holds@root={at0:?}");
        };
        let cc = || c("c");
        let topxy = || f("top", vec![v("x"), v("y")]);
        let topcy = || f("top", vec![cc(), v("y")]);

        // intersection (non-additive)
        let inter = vec![
            rule(atom("P", topxy()), vec![]),
            rule(atom("Q", topxy()), vec![]),
            rule(atom("A", cc()), vec![atom("P", topcy())]),
            rule(atom("B", cc()), vec![atom("Q", topcy())]),
            rule(atom("C", cc()), vec![atom("A", cc()), atom("B", cc())]),
            rule(atom("Win", topxy()), vec![atom("C", v("x"))]),
        ];
        let tcc = f("top", vec![cc(), cc()]);
        check(&inter, &tcc, &[("Win".into(), true)], "inter+");
        check(&inter, &tcc, &[("Win".into(), false)], "inter-");

        // mixed up+down
        let gx = || f("g", vec![v("x")]);
        let mixed = vec![
            rule(atom("Good", cc()), vec![]),
            rule(atom("T", topxy()), vec![]),
            rule(atom("R", gx()), vec![atom("Good", v("x"))]),
            rule(atom("W", gx()), vec![atom("T", f("top", vec![gx(), v("z")])), atom("R", gx())]),
            rule(atom("Win", topxy()), vec![atom("W", v("x"))]),
        ];
        let tgc = f("top", vec![f("g", vec![cc()]), cc()]);
        check(&mixed, &tgc, &[("Win".into(), true)], "mixed+");
        check(&mixed, &tgc, &[("Win".into(), false)], "mixed-");

        // multiple parent atoms
        let multi = vec![
            rule(atom("P", topxy()), vec![]),
            rule(atom("Q", f("top", vec![v("x"), cc()])), vec![atom("Good", v("y"))]),
            rule(atom("Good", cc()), vec![]),
            rule(atom("U", cc()), vec![atom("P", topcy()), atom("Q", topcy())]),
            rule(atom("Win", topxy()), vec![atom("U", v("x"))]),
        ];
        check(&multi, &tcc, &[("Win".into(), true)], "multi+");
        check(&multi, &tcc, &[("Win".into(), false)], "multi-");

        // unfounded cycle: A@child ⟷ B@parent with no base ⇒ LFP empty.
        let cyc = vec![
            rule(atom("A", cc()), vec![atom("B", topcy())]),
            rule(atom("B", topxy()), vec![atom("A", v("x"))]),
        ];
        check(&cyc, &tcc, &[("B".into(), true)], "cyc-unfounded");
        check(&cyc, &tcc, &[("B".into(), false)], "cyc-empty");
        // founded once a base is added.
        let mut cyc2 = cyc.clone();
        cyc2.push(rule(atom("A", cc()), vec![]));
        check(&cyc2, &tcc, &[("B".into(), true)], "cyc-founded");

        // 3-level mutual cycle across grandparent–parent–child — the case a
        // set-only channel would miss: Q0@R → Q1@M → Q2@c → Q3@M → Q0@R (no base
        // ⇒ LFP empty; the relation must turn it into a same-node Q1↔Q3 cycle at M).
        let mx = || f("M", vec![v("x")]);
        let cyc3 = vec![
            rule(atom("Q0", f("R", vec![v("x")])), vec![atom("Q1", v("x"))]),
            rule(atom("Q1", mx()), vec![atom("Q2", v("x"))]),
            rule(atom("Q2", cc()), vec![atom("Q3", f("M", vec![cc()]))]),
            rule(atom("Q3", mx()), vec![atom("Q0", f("R", vec![mx()]))]),
        ];
        let t3 = f("R", vec![f("M", vec![cc()])]);
        check(&cyc3, &t3, &[("Q0".into(), true)], "cyc3-unfounded");
        check(&cyc3, &t3, &[("Q0".into(), false)], "cyc3-empty");
        // founded once the child gets a base.
        let mut cyc3f = cyc3.clone();
        cyc3f.push(rule(atom("Q2", cc()), vec![]));
        check(&cyc3f, &t3, &[("Q0".into(), true)], "cyc3-founded");

        // DEEP pure-down chain (depth ≫ |Q|): no tree bound needed.
        let deep = vec![
            rule(atom("Acc", c("e")), vec![]),
            rule(atom("Acc", f("s", vec![v("x")])), vec![atom("Acc", v("x"))]),
        ];
        let mut spine = c("e");
        for _ in 0..20 {
            spine = f("s", vec![spine]);
        }
        check(&deep, &spine, &[("Acc".into(), true)], "deep-chain");
    }

    /// COMPUTE-STYLE (no annotation): for a **one-way** program (no `Parent`
    /// atoms) a node's up-states are a pure bottom-up fold of its children —
    /// the deterministic run, no `ann`, no SMT. This is what "remove `ann`"
    /// looks like where it works. `build_ground` numbers nodes pre-order, so
    /// reverse id order is children-first.
    fn compute_up_oneway(gt: &Ground, gcs: &[GC]) -> Vec<HashSet<String>> {
        let n = gt.ctor.len();
        let mut up: Vec<HashSet<String>> = vec![HashSet::default(); n];
        for node in (0..n).rev() {
            let f = &gt.ctor[node];
            loop {
                let mut changed = false;
                for gc in gcs.iter().filter(|gc| &gc.ctor == f) {
                    if up[node].contains(&gc.head) {
                        continue;
                    }
                    let ok = gc.body.iter().all(|a| match a {
                        GA::Child(i, st) => up[gt.children[node][*i]].contains(st),
                        GA::Selfp(st) => up[node].contains(st),
                        GA::Parent(..) => false, // one-way fold cannot see the parent
                    });
                    if ok {
                        up[node].insert(gc.head.clone());
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
        }
        up
    }

    /// The compute-style fold equals the exact LFP for one-way programs — so for
    /// these, `ann` is unnecessary (the run is a deterministic value, ideal as the
    /// exact-path DP key). The cases include intersection (AND) and a deep chain.
    #[test]
    fn compute_oneway_matches_lfp() {
        let check = |rules: &[HornRule], tree: &HT, label: &str| {
            let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
            let gt = build_ground(tree);
            let got = compute_up_oneway(&gt, &gcs);
            let want: HashSet<String> =
                crossing_eval(rules, &gt).into_iter().filter(|(_, node)| *node == 0).map(|(s, _)| s).collect();
            assert_eq!(got[0], want, "compute_up_oneway != LFP for {label}");
        };
        let cc = || c("c");
        // pure-down intersection (AND): C(f(x,y)) :- A(x),B(y).
        let inter = vec![
            rule(atom("A", cc()), vec![]),
            rule(atom("B", cc()), vec![]),
            rule(atom("C", f("f", vec![v("x"), v("y")])), vec![atom("A", v("x")), atom("B", v("y"))]),
        ];
        check(&inter, &f("f", vec![cc(), cc()]), "inter-down");
        // and the case it *can't* derive without parent info: a base only present at
        // the root would need a down-move; with only the leaf base it still folds.
        let deep = vec![
            rule(atom("Acc", c("e")), vec![]),
            rule(atom("Acc", f("s", vec![v("x")])), vec![atom("Acc", v("x"))]),
        ];
        let mut spine = c("e");
        for _ in 0..20 {
            spine = f("s", vec![spine]);
        }
        check(&deep, &spine, "deep-chain");
    }

    /// The wall: once a `Parent` atom appears (two-way), a node's up-states depend
    /// on the parent context, which a bottom-up fold cannot see — so the
    /// annotation-free fold gives the WRONG answer, while the LFP (and the
    /// `ann`-certified encoder) is correct. This is why `ann` is essential for
    /// two-way: removing it forces either the exponential determinized response
    /// (the 2WATA→1NTA blow-up) or backend iteration — not a free bottom-up fold.
    #[test]
    fn compute_fold_insufficient_for_twoway() {
        let cc = || c("c");
        let gx = || f("g", vec![v("x")]);
        // mixed up+down: W(g(x)) :- T(top(g(x),z)) ∧ R(g(x)); Win(top(x,y)) :- W(x).
        let mixed = vec![
            rule(atom("Good", cc()), vec![]),
            rule(atom("T", f("top", vec![v("x"), v("y")])), vec![]),
            rule(atom("R", gx()), vec![atom("Good", v("x"))]),
            rule(atom("W", gx()), vec![atom("T", f("top", vec![gx(), v("z")])), atom("R", gx())]),
            rule(atom("Win", f("top", vec![v("x"), v("y")])), vec![atom("W", v("x"))]),
        ];
        let tree = f("top", vec![f("g", vec![cc()]), cc()]);
        let gcs: Vec<GC> = mixed.iter().filter_map(extract_gc).collect();
        let gt = build_ground(&tree);
        let fold0 = &compute_up_oneway(&gt, &gcs)[0];
        let lfp0: HashSet<String> =
            crossing_eval(&mixed, &gt).into_iter().filter(|(_, n)| *n == 0).map(|(s, _)| s).collect();
        // the LFP accepts `Win` at the root; the bottom-up fold misses it (it cannot
        // see that the parent provides `T` via the up-move) — they MUST differ.
        assert!(lfp0.contains("Win"), "LFP should accept Win");
        assert!(!fold0.contains("Win"), "fold cannot derive Win without parent info");
        assert_ne!(fold0, &lfp0, "two-way: fold ≠ LFP — ann is doing real work here");
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
        let sygus = faithful_sygus_ir(&states, &ctors, &gcs, "T", &[("Win".to_string(), true)], "g");
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
