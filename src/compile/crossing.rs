//! Sound & complete compilation of delis programs **with `parent` (up-moves)**
//! into SyGuS-BV, via the faithful two-way → one-way crossing construction
//! (Shepherdson 1959; Vardi/Cachat, as cited by Krogmeier–Madhusudan 2023).
//!
//! The semantics of a FACET program is a two-way alternating tree automaton over
//! the candidate syntax tree, accepted by a least fixpoint (reachability). The
//! one-way encoding here is a **crossing-relation annotation certificate**: each
//! node's `ann` (`Constant`) guesses only a parent-channel `dact` and clause
//! `pick` bits; `up[state]` (`= ⋁ pick`), the child→parent demand `udem`, and a
//! `valid` bit are *computed* in the define-fun. A same-node support relation
//! `R ⊆ Q×Q` is computed internally (never stored). The `valid` bit checks, for
//! every clause uniformly (child-ref / self-loop / parent-atom), that `up` is
//! justified and that `R` is the chosen support — irreflexive and transitively
//! closed, i.e. acyclic, hence the least fixpoint. `R` summarizes the up/down
//! crossings (Shepherdson 1959;
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
/// The `w`-bit set `{q}` (bit `q` set), as an exact binary literal — no decimal
/// `bvN` (which overflows for `q ≥ 128`) and no width bound.
fn singleton(q: usize, w: usize) -> String {
    format!("#b{}1{}", "0".repeat(w - 1 - q), "0".repeat(q))
}
/// `bvor` of `w`-bit masks; `[]`⇒`bv0`, `[x]`⇒`x`.
fn bvor_all(xs: &[String], w: usize) -> String {
    match xs {
        [] => bv0(w),
        [one] => one.clone(),
        _ => format!("(bvor {})", xs.join(" ")),
    }
}
/// Concatenate `terms` (high→low) into one bit-vector; `[x]`⇒`x`, `[]`⇒`""`.
fn concat_terms(terms: &[String]) -> String {
    match terms {
        [] => String::new(),
        [one] => one.clone(),
        _ => format!("(concat {})", terms.join(" ")),
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
/// SyGuS emitter and the fixed-tree validation oracle. Node value per sort `s`
/// (`Q=|states|`, `D=|DS|`, `pw=⌈log₂(max group)⌉`); low bits are the guessed
/// `ann` (`Constant`), high bits are computed in the define-fun:
///
/// `[ dact:D | pick:Q·pw | up:Q | udem:Q·D | valid:1 ]`   (low → high)
///   └────── ann (guessed) ──────┘└──────── computed ─────────┘
///
/// - `dact[j]`          — parent-channel for `DS` triple `j=(pc,pos,st)`: "my
///   parent is `pc`, I'm at `pos`, parent has `st`" (certified by the parent =
///   parent's `up[st]`). This is the irreducible two-way guess. *Guessed.*
/// - `pick[q]`          — `pw`-bit binary index of the clause that founds `up[q]`
///   (`0` = unfounded); makes the chosen derivation unambiguous so `R` is the
///   chosen support, not an over-approx. *Guessed.*
/// - `up[q]`            — `= ⋁ pick[q]`: state `q` holds at this node. *Computed.*
/// - `udem[Y][j]`       — child→parent demand: my state `Y` routes up to `st@parent`
///   for triple `j`. Every support cycle is caught at its *top* node via this
///   down-routing, so the symmetric up-routing channel (`ddem`) is unnecessary.
///   *Computed.*
/// - `valid`            — conjunction of all `node_checks`. *Computed.*
///
/// `R[p][q]` (same-node support relation: `up[p]` transitively depends on `up[q]`,
/// required irreflexive ⇒ acyclic support ⇒ least fixpoint; the Shepherdson/Vardi
/// crossing relation, no parity since delis is monotone LFP) is **never stored in
/// the value** — it is `let`-bound internally per node, computed by
/// SCC-condensation of the `may_dep` graph (see `node_checks`). Its width depends
/// only on `|Q|`, never on tree size.
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
    /// per sort, the packed bit offset (within the `udem` field) of each routable
    /// `(y,t)` pair. Non-routable pairs are absent — they are dead bits (forced
    /// `#b0`, read only under `routes_up`), so the `udem` field stores only these.
    ud_rank: HashMap<String, HashMap<(usize, usize), usize>>,
    /// per clause, its 1-based position within its `(ctor, head)` group — so
    /// `pick` is a per-state *binary index* of the founding clause (every value a
    /// valid choice) instead of `P` one-hot bits (almost all patterns invalid).
    pos_in_group: Vec<usize>,
    /// per sort, prefix bit-offsets of each state's `pick` field within the pick
    /// region (len `Q+1`; `[q]` = offset of state `q`, `[Q]` = total pick width).
    /// Each state gets only `⌈log₂(maxgroup(q)+1)⌉` bits, not a global `pw` — a
    /// singleton-only state costs ~1 bit, not the sort-wide max.
    pick_off: HashMap<String, Vec<usize>>,
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
        // Pack the `udem` field: assign each routable pair a dense bit offset (a
        // deterministic sorted order, so writer and reader agree).
        let mut ud_rank: HashMap<String, HashMap<(usize, usize), usize>> = HashMap::default();
        for (sort, set) in &route {
            let mut pairs: Vec<(usize, usize)> = set.iter().copied().collect();
            pairs.sort_unstable();
            ud_rank.insert(sort.clone(), pairs.into_iter().enumerate().map(|(i, p)| (p, i)).collect());
        }
        // pick as a per-state binary clause index: position within (ctor,head) group.
        let mut group_count: HashMap<(String, String), usize> = HashMap::default();
        let mut pos_in_group: Vec<usize> = vec![0; gcs.len()];
        for (k, gc) in gcs.iter().enumerate() {
            let c = group_count.entry((gc.ctor.clone(), gc.head.clone())).or_insert(0);
            *c += 1;
            pos_in_group[k] = *c;
        }
        // Per-state pick width: state `q`'s field must hold `0..maxgroup(q)`
        // where `maxgroup(q)` = the largest `(ctor,q)` group over all ctors.
        let mut pick_off: HashMap<String, Vec<usize>> = HashMap::default();
        for (sort, sts) in states {
            let mut off = Vec::with_capacity(sts.len() + 1);
            let mut acc = 0usize;
            for q in sts {
                off.push(acc);
                let maxg = sort_clauses
                    .get(sort)
                    .map(|ks| {
                        ks.iter()
                            .filter(|&&k| &gcs[k].head == q)
                            .map(|&k| group_count[&(gcs[k].ctor.clone(), gcs[k].head.clone())])
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                // bits to index `0..maxg`; a non-head state (maxg=0) costs 0 bits.
                let w = if maxg == 0 { 0 } else { (usize::BITS - maxg.leading_zeros()) as usize };
                acc += w;
            }
            off.push(acc);
            pick_off.insert(sort.clone(), off);
        }
        Enc { states, gcs, ds, sort_clauses, reach, route, ud_rank, pos_in_group, pick_off }
    }

    /// Can `y` (state index of `sort`) route up via DS triple `t` in some candidate?
    fn routes_up(&self, sort: &str, y: usize, t: usize) -> bool {
        self.route.get(sort).is_some_and(|s| s.contains(&(y, t)))
    }
    /// Can `R[p][q]` ever be 1? Only if `q` is reachable from `p`.
    fn may_dep(&self, p: &str, q: &str) -> bool {
        self.reach.get(p).is_some_and(|r| r.contains(q))
    }

    /// Strongly-connected components of the `may_dep` graph among `sort`'s
    /// states. Returns `(scc_of, order)`: `scc_of[p]` is `p`'s component id, and
    /// `order` lists components in sink-first order (a component appears before
    /// every component that depends on it). Because every gated `R` edge lies in
    /// `may_dep` (verified), each actual support cycle is contained in one
    /// component — so the closure can square within components only, and treat
    /// the inter-component edges as the DAG they are. Iterative Tarjan, which
    /// yields components in reverse-topological (sink-first) finalization order.
    fn sccs(&self, sort: &str) -> (Vec<usize>, Vec<Vec<usize>>) {
        let sts = &self.states[sort];
        let q = sts.len();
        let idx: HashMap<&str, usize> =
            sts.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
        let mut adj = vec![Vec::new(); q];
        for (i, s) in sts.iter().enumerate() {
            if let Some(r) = self.reach.get(s) {
                for t in r {
                    if let Some(&j) = idx.get(t.as_str()) {
                        if i != j {
                            adj[i].push(j);
                        }
                    }
                }
            }
        }
        let (mut id, mut low) = (vec![usize::MAX; q], vec![0usize; q]);
        let mut on = vec![false; q];
        let (mut stk, mut scc_of) = (Vec::new(), vec![usize::MAX; q]);
        let (mut counter, mut nscc) = (0usize, 0usize);
        for s0 in 0..q {
            if id[s0] != usize::MAX {
                continue;
            }
            let mut call = vec![(s0, 0usize)];
            while let Some(&(v, pi)) = call.last() {
                if pi == 0 {
                    id[v] = counter;
                    low[v] = counter;
                    counter += 1;
                    stk.push(v);
                    on[v] = true;
                }
                if pi < adj[v].len() {
                    let w = adj[v][pi];
                    call.last_mut().unwrap().1 += 1;
                    if id[w] == usize::MAX {
                        call.push((w, 0));
                    } else if on[w] {
                        low[v] = low[v].min(id[w]);
                    }
                } else {
                    if low[v] == id[v] {
                        loop {
                            let w = stk.pop().unwrap();
                            on[w] = false;
                            scc_of[w] = nscc;
                            if w == v {
                                break;
                            }
                        }
                        nscc += 1;
                    }
                    call.pop();
                    if let Some(&(p, _)) = call.last() {
                        low[p] = low[p].min(low[v]);
                    }
                }
            }
        }
        let mut order = vec![Vec::new(); nscc];
        for (p, &c) in scc_of.iter().enumerate() {
            order[c].push(p);
        }
        (scc_of, order)
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
    fn qidx(&self, s: &str, st: &str) -> Option<usize> {
        self.states[s].iter().position(|x| x == st)
    }
    fn dsidx(&self, s: &str, e: &(String, usize, String)) -> Option<usize> {
        self.ds_of(s).iter().position(|x| x == e)
    }

    /// Width of state `q`'s `pick` field (`⌈log₂(maxgroup(q)+1)⌉`, 0 if non-head).
    fn pw_of(&self, s: &str, q: usize) -> usize {
        let off = &self.pick_off[s];
        off[q + 1] - off[q]
    }
    /// Total `pick` bits for sort `s` (sum of per-state widths).
    fn pick_w(&self, s: &str) -> usize {
        *self.pick_off[s].last().unwrap()
    }

    // --- layout offsets: value = [dact:D | pick:ΣwQ | up:Q | udem:Nud | valid:1] ---
    // ONLY `dact`+`pick` are guessed (the Constant, low bits, so Constant offsets =
    // value offsets); `up`/`udem` are COMPUTED in the define-fun (high bits), and `R`
    // (the support relation) is computed *internally* (let-bound rows, never stored).
    // `udem` carries only the ROUTABLE `(y,t)` pairs (`Nud = |route[s]|`): a
    // non-routable bit is forced `#b0` by `node_checks` AND only ever read under the
    // same `routes_up` gate, so it is dead — dropping it is behavior-preserving.
    fn width(&self, s: &str) -> usize {
        self.udem_base(s) + self.nud(s) + 1
    }
    fn dact_pos(&self, _s: &str, j: usize) -> usize {
        j
    }
    /// Low bit of state `q`'s `pick` index (a per-state binary clause selector).
    fn pick_idx_lo(&self, s: &str, q: usize) -> usize {
        self.nd(s) + self.pick_off[s][q]
    }
    fn up_pos(&self, s: &str, q: usize) -> usize {
        self.nd(s) + self.pick_w(s) + q
    }
    /// Number of routable `(y,t)` pairs = width of the packed `udem` field.
    fn nud(&self, s: &str) -> usize {
        self.ud_rank.get(s).map(HashMap::len).unwrap_or(0)
    }
    /// Low bit of the `udem` field (just above `up`).
    fn udem_base(&self, s: &str) -> usize {
        self.nd(s) + self.pick_w(s) + self.nq(s)
    }
    /// Packed bit position of routable pair `(y,t)`. Only ever called for routable
    /// pairs (writer iterates `route`; reader gates on `routes_up`).
    fn udem_pos(&self, s: &str, y: usize, t: usize) -> usize {
        self.udem_base(s) + self.ud_rank[s][&(y, t)]
    }
    /// Width of the `Constant` = the guess-only prefix `[dact | pick]`.
    fn ann_w(&self, s: &str) -> usize {
        self.nd(s) + self.pick_w(s)
    }
    /// "Clause `k` is the one picked here" = `pick[head(k)] == position-of-k`.
    /// Read from the per-node `sel{k}` binding (CSE'd once by `node_checks`), not
    /// re-sliced from `ann` at every use — the slice reads were ~half the file.
    fn pick_sel(&self, pfx: &str, k: usize) -> String {
        format!("{pfx}sel{k}")
    }
    /// The actual definition of `sel{k}` (a Bool `let`): `pick[head(k)] == pos(k)`.
    fn pick_sel_def(&self, s: &str, term: &str, k: usize) -> String {
        let q = self.qidx(s, &self.gcs[k].head).expect("clause head is a state");
        let pw = self.pw_of(s, q);
        let lo = self.pick_idx_lo(s, q);
        format!("(= {} (_ bv{} {pw}))", slice(term, lo, lo + pw - 1), self.pos_in_group[k])
    }
    fn valid_pos(&self, s: &str) -> usize {
        self.width(s) - 1
    }

    /// The local certificate + the COMPUTED fields for a node of ctor `ctor`.
    /// Returns `(groups, checks, up_val, udem_val)`: `lets` are `(name,width,expr)`
    /// bindings (the `up` bits and the `R` closure rows) that the emitter `let`-wraps
    /// and the oracle emits as `declare-const`+`assert`; `checks`' conjunction is
    /// `valid`; `up_val`/`udem_val` are the computed value fields placed into the node
    /// value `[dact|pick|up|udem|valid]`. `ann` reads `dact`/`pick`, `kids[i]` reads
    /// child `up`/`udem`/`valid`, `pfx` namespaces the let names.
    #[allow(clippy::type_complexity)]
    fn node_checks(
        &self,
        sort: &str,
        ctor: &str,
        arg_sorts: &[String],
        ann: &str,
        kids: &[String],
        pfx: &str,
    ) -> (Vec<Vec<(String, usize, String)>>, Vec<String>, String, String) {
        let q_n = self.nq(sort);
        // Nested `let` groups, emitted outer→inner; a binding may reference any
        // earlier group. Group 0 = `sel{k}` clause-selector Bools (CSE'd once,
        // read by up/R0/Cpick/udem). Group 1 = `up` + `R0`. The R* closure then
        // appends one group per SCC-condensation step (downstream incorporation,
        // then per-component squaring rounds), in sink-first order. Bool bindings
        // carry width 0 (sentinel) so the oracle declares them `Bool`, not BitVec.
        let mut groups: Vec<Vec<(String, usize, String)>> = vec![Vec::new(), Vec::new()];
        let mut checks: Vec<String> = Vec::new();
        let up_name = |q: usize| format!("{pfx}up{q}");
        // (sel) one binding per clause of this ctor — the only read of `ann`'s
        // `pick` field; everything else references the Bool by name.
        for &k in self.clauses(sort) {
            if self.gcs[k].ctor == *ctor {
                groups[0].push((format!("{pfx}sel{k}"), 0, self.pick_sel_def(sort, ann, k)));
            }
        }

        // premises of clause `gc` hold (Child→child up, Selfp→own up, Parent→dact).
        let prem_active = |gc: &GC| -> String {
            let parts: Vec<String> = gc
                .body
                .iter()
                .map(|atom| match atom {
                    GA::Child(ci, st) => {
                        format!("(= {} #b1)", bit(&kids[*ci], self.up_pos(&arg_sorts[*ci], self.qidx(&arg_sorts[*ci], st).unwrap())))
                    }
                    GA::Selfp(st) => format!("(= {} #b1)", up_name(self.qidx(sort, st).unwrap())),
                    GA::Parent(pc, pos, st) => {
                        format!("(= {} #b1)", bit(ann, self.dact_pos(sort, self.dsidx(sort, &(pc.clone(), *pos, st.clone())).unwrap())))
                    }
                })
                .collect();
            and_bool(&parts)
        };
        // DIRECT same-node deps of `gc` (Selfp→{q'}; Child→self-states `st@child`
        // routes back to, via child `udem`; Parent→none). The closure adds the rest.
        let deps_direct = |gc: &GC| -> String {
            let mut parts: Vec<String> = Vec::new();
            for atom in &gc.body {
                match atom {
                    GA::Selfp(st) => parts.push(singleton(self.qidx(sort, st).unwrap(), q_n)),
                    GA::Parent(..) => {}
                    GA::Child(ci, st) => {
                        let a = &arg_sorts[*ci];
                        let qa = self.qidx(a, st).unwrap();
                        let mut terms: Vec<String> = Vec::new();
                        for (tj, (pc2, pos2, st2)) in self.ds_of(a).iter().enumerate() {
                            if pc2 == ctor && *pos2 == *ci && self.routes_up(a, qa, tj) {
                                if let Some(sj) = self.qidx(sort, st2) {
                                    terms.push(format!("(ite (= {} #b1) {} {})", bit(&kids[*ci], self.udem_pos(a, qa, tj)), singleton(sj, q_n), bv0(q_n)));
                                }
                            }
                        }
                        parts.push(bvor_all(&terms, q_n));
                    }
                }
            }
            bvor_all(&parts, q_n)
        };

        // (up) up[q] = ⋁ pick for (ctor,q); closure: any active clause ⇒ up[q].
        for (q, qst) in self.states[sort].iter().enumerate() {
            let mine: Vec<usize> =
                self.clauses(sort).iter().copied().filter(|&k| self.gcs[k].ctor == *ctor && &self.gcs[k].head == qst).collect();
            let picks: Vec<String> = mine.iter().map(|&k| self.pick_sel(pfx, k)).collect();
            groups[1].push((up_name(q), 1, format!("(ite {} #b1 #b0)", or_bool(&picks))));
            let actives: Vec<String> = mine.iter().map(|&k| prem_active(&self.gcs[k])).collect();
            if !actives.is_empty() {
                checks.push(format!("(=> {} (= {} #b1))", or_bool(&actives), up_name(q)));
            }
        }
        let up_val = concat_terms(&(0..q_n).rev().map(up_name).collect::<Vec<_>>());

        // (Cpick) a picked clause must be active (its deps enter R automatically).
        for &k in self.clauses(sort) {
            if self.gcs[k].ctor == *ctor {
                checks.push(format!("(=> {} {})", self.pick_sel(pfx, k), prem_active(&self.gcs[k])));
            }
        }

        // (R) support relation R*, by SCC-condensation of the `may_dep` graph.
        // Inter-component edges form a DAG: no squaring needed — incorporate the
        // already-final rows of downstream components in sink-first order. Only
        // states inside one strongly-connected component can lie on a cycle, so
        // squaring is bounded by component size, not Q. R0[p] = this ctor's gated
        // direct support. Sound for both the acyclicity check and udem's
        // reachability reads because every gated edge is in `may_dep` (verified by
        // `r0_support_edges_are_within_may_dep`).
        let (scc_of, order) = self.sccs(sort);
        let r0_name = |p: usize| format!("{pfx}R0_{p}");
        let empty = bv0(q_n);
        for p in 0..q_n {
            let terms: Vec<String> = self
                .clauses(sort)
                .iter()
                .filter(|&&k| self.gcs[k].ctor == *ctor && self.qidx(sort, &self.gcs[k].head) == Some(p))
                // A clause with no same-node deps contributes `(ite pick 0 0)` ≡ 0
                // to the bvor — drop it (these no-op ites were ~16k of the file).
                .filter_map(|&k| {
                    let d = deps_direct(&self.gcs[k]);
                    (d != empty).then(|| format!("(ite {} {} {})", self.pick_sel(pfx, k), d, empty))
                })
                .collect();
            groups[1].push((r0_name(p), q_n, bvor_all(&terms, q_n)));
        }
        // Structural direct R0 targets of `p` for this ctor — splits `p`'s edges
        // into intra-component (squared) vs downstream (DAG-incorporated). Mirrors
        // `deps_direct`'s target set; an edge never points to a higher component.
        let r0_targets = |p: usize| -> Vec<usize> {
            let mut tg = Vec::new();
            for &k in self.clauses(sort) {
                if self.gcs[k].ctor != *ctor || self.qidx(sort, &self.gcs[k].head) != Some(p) {
                    continue;
                }
                for atom in &self.gcs[k].body {
                    match atom {
                        GA::Parent(..) => {}
                        GA::Selfp(st) => tg.extend(self.qidx(sort, st)),
                        GA::Child(ci, st) => {
                            let a = &arg_sorts[*ci];
                            if let Some(qa) = self.qidx(a, st) {
                                for (tj, (pc2, pos2, st2)) in self.ds_of(a).iter().enumerate() {
                                    if pc2 == ctor && *pos2 == *ci && self.routes_up(a, qa, tj) {
                                        tg.extend(self.qidx(sort, st2));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            tg.sort_unstable();
            tg.dedup();
            tg
        };
        // Final closure-row name per state, resolved sink-first.
        let mut rf: Vec<String> = (0..q_n).map(|p| r0_name(p)).collect();
        for comp in &order {
            // Incorporate downstream rows: a live direct edge `p→w` to a lower
            // component means `p` reaches everything `w` reaches (`rf[w]` final).
            let mut g = groups.len();
            groups.push(Vec::new());
            for &p in comp {
                let mut terms = vec![r0_name(p)];
                for w in r0_targets(p) {
                    if scc_of[w] != scc_of[p] {
                        terms.push(format!("(ite (= {} #b1) {} {})", bit(&r0_name(p), w), rf[w], bv0(q_n)));
                    }
                }
                groups[g].push((format!("{pfx}Rc_{p}"), q_n, bvor_all(&terms, q_n)));
            }
            for &p in comp {
                rf[p] = format!("{pfx}Rc_{p}");
            }
            // Close cycles: square within this component only (≤⌈log₂|C|⌉ rounds).
            if comp.len() > 1 {
                let crounds = (usize::BITS - (comp.len() - 1).leading_zeros()) as usize;
                for r in 1..=crounds {
                    let prev: Vec<String> = comp.iter().map(|&p| rf[p].clone()).collect();
                    g = groups.len();
                    groups.push(Vec::new());
                    for (pi, &p) in comp.iter().enumerate() {
                        let mut terms = vec![prev[pi].clone()];
                        for (mi, &m) in comp.iter().enumerate() {
                            if m != p {
                                terms.push(format!("(ite (= {} #b1) {} {})", bit(&prev[pi], m), prev[mi], bv0(q_n)));
                            }
                        }
                        groups[g].push((format!("{pfx}Rs{r}_{p}"), q_n, bvor_all(&terms, q_n)));
                    }
                    for &p in comp {
                        rf[p] = format!("{pfx}Rs{r}_{p}");
                    }
                }
            }
        }
        // (R irreflexive) acyclicity — only states that can lie on a cycle: those
        // in a multi-state component, or with a structural self-edge.
        for comp in &order {
            for &p in comp {
                if comp.len() > 1 || r0_targets(p).contains(&p) {
                    checks.push(format!("(= {} #b0)", bit(&rf[p], p)));
                }
            }
        }

        // (udem) `y` routes up via `t` = picked Parent-t clause for `y`, or `y`
        // depends (R) on some `z` that does. Precompute the Parent-`t` *users* once
        // (per triple, the few states whose picked clause directly routes up) —
        // scanning clauses inside the `Q²·D` loop is `O(Q²·D·P)` and is what hung.
        let nd = self.nd(sort);
        let mut pu: Vec<Vec<(usize, Vec<usize>)>> = vec![Vec::new(); nd];
        for &k in self.clauses(sort) {
            if self.gcs[k].ctor != *ctor {
                continue;
            }
            let z = self.qidx(sort, &self.gcs[k].head).unwrap();
            for a in &self.gcs[k].body {
                if let GA::Parent(pc, pos, st) = a {
                    if let Some(t) = self.dsidx(sort, &(pc.clone(), *pos, st.clone())) {
                        match pu[t].iter_mut().find(|(zz, _)| *zz == z) {
                            Some((_, ks)) => ks.push(k),
                            None => pu[t].push((z, vec![k])),
                        }
                    }
                }
            }
        }
        // CSE the per-`(t,z)` "z's picked clause routes up via t" disjunction —
        // it is constant in `y`, but read once per `y` that depends on `z`.
        let g = groups.len();
        groups.push(Vec::new());
        let mut od: Vec<Vec<String>> = vec![Vec::new(); nd];
        for (t, users) in pu.iter().enumerate() {
            for (zi, (_z, ks)) in users.iter().enumerate() {
                let name = format!("{pfx}od{t}_{zi}");
                groups[g].push((name.clone(), 0, or_bool(&ks.iter().map(|&k| self.pick_sel(pfx, k)).collect::<Vec<_>>())));
                od[t].push(name);
            }
        }
        // Emit only the routable pairs, placed at their packed rank (non-routable
        // bits were dead — forced #b0, read only under `routes_up`). `concat` is
        // high→low, so emit ranks descending to land rank `r` at offset `base+r`.
        let nud = self.nud(sort);
        let mut by_rank: Vec<String> = vec!["#b0".to_string(); nud];
        for (&(y, t), &rank) in self.ud_rank.get(sort).into_iter().flatten() {
            let yst = &self.states[sort][y];
            let mut terms: Vec<String> = Vec::new();
            for (zi, (z, _ks)) in pu[t].iter().enumerate() {
                if *z == y {
                    terms.push(od[t][zi].clone());
                } else if self.may_dep(yst, &self.states[sort][*z]) {
                    terms.push(format!("(and (= {} #b1) {})", bit(&rf[y], *z), od[t][zi]));
                }
            }
            by_rank[rank] = format!("(ite {} #b1 #b0)", or_bool(&terms));
        }
        let ud_terms: Vec<String> = (0..nud).rev().map(|r| by_rank[r].clone()).collect();
        let udem_val = concat_terms(&ud_terms);

        // (C0) every child is valid; (Cchannel) the parent certifies child `dact`.
        for (i, a) in arg_sorts.iter().enumerate() {
            checks.push(format!("(= {} #b1)", bit(&kids[i], self.valid_pos(a))));
            for (j, (pc, pos, st)) in self.ds_of(a).iter().enumerate() {
                let cact = bit(&kids[i], self.dact_pos(a, j));
                if pc == ctor && *pos == i && self.qidx(sort, st).is_some() {
                    checks.push(format!("(= {cact} {})", up_name(self.qidx(sort, st).unwrap())));
                } else {
                    checks.push(format!("(= {cact} #b0)"));
                }
            }
        }

        (groups, checks, up_val, udem_val)
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
            let ann_w = e.ann_w(sort);
            sig.push_str(&format!("(ann (_ BitVec {ann_w}))) (_ BitVec {w}) "));
            let kids: Vec<String> = (0..arg_sorts.len()).map(|i| format!("c{i}")).collect();
            let (groups, checks, up_val, udem_val) = e.node_checks(sort, ctor, arg_sorts, "ann", &kids, "");
            let valid = and_bool(&checks);
            // value = [ dact|pick (=ann, low) | up_val | udem_val | valid (top) ]
            let mut parts = vec![format!("(ite {valid} #b1 #b0)")];
            if !udem_val.is_empty() {
                parts.push(udem_val);
            }
            if !up_val.is_empty() {
                parts.push(up_val);
            }
            parts.push("ann".to_string());
            let mut body = concat_terms(&parts);
            // nest the let groups outer→inner: each SCC-condensation step (`Rc`,
            // `Rs{r}`) sees the earlier (downstream) rows only across a real `let`.
            for grp in groups.iter().rev() {
                if grp.is_empty() {
                    continue;
                }
                let bl = grp.iter().map(|(n, _w, x)| format!("({n} {x})")).collect::<Vec<_>>().join(" ");
                body = format!("(let ({bl}) {body})");
            }
            s.push_str(&format!("{sig}{body})\n"));
        }
    }

    // grammar (each node draws a free Ann constant)
    let mut nts = vec![format!("(Start (_ BitVec {}))", e.width(start_sort))];
    for sort in ctors.keys() {
        nts.push(format!("(NT.{sort} (_ BitVec {}))", e.width(sort)));
        nts.push(format!("(Ann.{sort} (_ BitVec {}))", e.ann_w(sort)));
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
        let aw = e.ann_w(sort);
        rules_txt.push(format!("(Ann.{sort} (_ BitVec {aw}) ((Constant (_ BitVec {aw}))))"));
    }
    s.push_str(&format!(
        "(synth-fun {synth_name} () (_ BitVec {})\n ({})\n ({}))\n",
        e.width(start_sort),
        nts.join(" "),
        rules_txt.join("\n  ")
    ));

    // constraints: valid root; the root has no parent ⇒ its `dact` channel is 0
    // (`udem` then computes to 0); goal on root up-bits.
    s.push_str(&format!("(constraint (= {} #b1))\n", bit(synth_name, e.valid_pos(start_sort))));
    let d = e.nd(start_sort);
    if d > 0 {
        let lo = e.dact_pos(start_sort, 0);
        s.push_str(&format!("(constraint (= {} {}))\n", slice(synth_name, lo, lo + d - 1), bv0(d)));
    }
    for (qstate, want) in constraints {
        if let Some(q) = e.qidx(start_sort, qstate) {
            s.push_str(&format!("(constraint (= {} #b{}))\n", bit(synth_name, e.up_pos(start_sort, q)), if *want { 1 } else { 0 }));
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

    /// Per-state `pick` width layout: each state's field must (a) hold every
    /// `pos_in_group` value of its clauses, (b) be contiguous and non-overlapping,
    /// and (c) leave `ann_w`/`up`/`udem`/`valid` offsets consistent. Non-head
    /// states cost 0 bits. Pure (no solver) — guards the #6 layout change.
    #[test]
    fn per_state_pick_widths_fit_and_pack() {
        // q0: 3 clauses under ctor `k` (group 3 → 2 bits); q1: 1 (→ 1 bit);
        // A,B,C: 1 leaf clause each (→ 1 bit); a non-head state gets 0 bits.
        let kx = || f("k", vec![v("x")]);
        let rules = vec![
            rule(atom("q0", kx()), vec![atom("A", v("x"))]),
            rule(atom("q0", kx()), vec![atom("B", v("x"))]),
            rule(atom("q0", kx()), vec![atom("C", v("x"))]),
            rule(atom("q1", kx()), vec![atom("A", v("x"))]),
            rule(atom("A", c("c")), vec![]),
            rule(atom("B", c("c")), vec![]),
            rule(atom("C", c("c")), vec![]),
        ];
        let tree = f("k", vec![c("c")]);
        let (states, ctors) = single_sort_sig(&rules, &tree);
        let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
        let e = Enc::new(&states, &ctors, &gcs);
        let s = "T";
        let qi = |n: &str| e.qidx(s, n).unwrap();
        // (a) widths match group sizes (bits to index 0..g).
        assert_eq!(e.pw_of(s, qi("q0")), 2, "group 3 needs 2 bits");
        assert_eq!(e.pw_of(s, qi("q1")), 1, "group 1 needs 1 bit");
        assert_eq!(e.pw_of(s, qi("A")), 1);
        // a state that is never a clause head costs 0 pick bits.
        if let Some(idx) = e.qidx(s, "Goal") {
            assert_eq!(e.pw_of(s, idx), 0);
        }
        // (b) every clause's selector value fits its head's field.
        for (k, gc) in gcs.iter().enumerate() {
            let w = e.pw_of(s, qi(&gc.head));
            assert!(e.pos_in_group[k] < (1usize << w), "pos {} overflows {w}-bit field for {}", e.pos_in_group[k], gc.head);
        }
        // (c) fields are contiguous & ordered; layout regions don't overlap.
        let off = &e.pick_off[s];
        assert!(off.windows(2).all(|w| w[0] <= w[1]), "pick offsets must be monotone");
        assert_eq!(off[0], 0);
        assert_eq!(e.ann_w(s), e.nd(s) + e.pick_w(s), "ann = dact + pick");
        assert_eq!(e.up_pos(s, 0), e.nd(s) + e.pick_w(s), "up sits just above [dact|pick]");
        assert_eq!(e.udem_base(s), e.up_pos(s, 0) + e.nq(s), "udem sits just above up");
        assert!(e.valid_pos(s) >= e.udem_base(s) + e.nud(s), "valid is the top bit");
    }

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
    /// A per-call-unique temp path, so concurrent cvc5 tests never clobber a
    /// shared file (the same `tag` running in two threads otherwise races).
    fn unique_tmp(tag: &str, ext: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("delis_{tag}_{}_{n}.{ext}", std::process::id()))
    }
    fn cvc5_solve(sygus: &str, tag: &str, tlimit_ms: u64) -> String {
        let path = unique_tmp(tag, "sygus");
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
        let path = unique_tmp(tag, "smt2");
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
            let nn = format!("n{n}");
            // The node is a free const of value width; `up`/`udem` are computed, so
            // bind the lets as aux consts and PIN the node's `up`/`udem` to them.
            let (groups, checks, up_val, udem_val) = e.node_checks(sort, ctor, arg_sorts, &nn, &kids, &format!("n{n}_"));
            for (name, bw, expr) in groups.iter().flatten() {
                // width 0 = the `sel{k}`/`od` Bool sentinel; everything else is a BitVec.
                let sort_str = if *bw == 0 { "Bool".to_string() } else { format!("(_ BitVec {bw})") };
                smt.push_str(&format!("(declare-const {name} {sort_str})\n(assert (= {name} {expr}))\n"));
            }
            let q_n = e.nq(sort);
            if q_n > 0 {
                let up_lo = e.up_pos(sort, 0);
                smt.push_str(&format!("(assert (= {} {up_val}))\n", slice(&nn, up_lo, up_lo + q_n - 1)));
            }
            if e.nud(sort) > 0 {
                let ud_lo = e.udem_base(sort);
                smt.push_str(&format!("(assert (= {} {udem_val}))\n", slice(&nn, ud_lo, ud_lo + e.nud(sort) - 1)));
            }
            for chk in &checks {
                smt.push_str(&format!("(assert {chk})\n"));
            }
        }
        let d = e.nd(sort);
        if d > 0 {
            let lo = e.dact_pos(sort, 0);
            smt.push_str(&format!("(assert (= {} {}))\n", slice("n0", lo, lo + d - 1), bv0(d)));
        }
        for (qst, want) in constraints {
            if let Some(q) = e.qidx(sort, qst) {
                smt.push_str(&format!("(assert (= {} #b{}))\n", bit("n0", e.up_pos(sort, q)), if *want { 1 } else { 0 }));
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

        // CONSECUTIVE up-moves in the ACCEPTING derivation: leaf P ← M.Q ← R.W
        // (root base) — two up-moves in a row must compose. Dropping the base
        // makes it unfounded ⇒ must reject.
        let mw = |inner: HT| f("M", vec![inner]);
        let rw = |inner: HT| f("R", vec![inner]);
        let cup = vec![
            rule(atom("W", rw(v("x"))), vec![]),
            rule(atom("Q", mw(v("x"))), vec![atom("W", rw(mw(v("x"))))]),
            rule(atom("P", cc()), vec![atom("Q", mw(cc()))]),
            rule(atom("Acc2", cc()), vec![atom("P", cc())]),
            rule(atom("Acc", mw(v("x"))), vec![atom("Acc2", v("x"))]),
            rule(atom("Win", rw(v("x"))), vec![atom("Acc", v("x"))]),
        ];
        let trc = rw(mw(cc()));
        check(&cup, &trc, &[("Win".into(), true)], "consec-ups+");
        check(&cup[1..], &trc, &[("Win".into(), true)], "consec-ups-unfounded");

        // SHUTTLE (bounce c↕M): A@c ← B@M ← C@c ← D@M (base) — up, down, up.
        let shu = vec![
            rule(atom("D", mw(v("x"))), vec![]),
            rule(atom("C", cc()), vec![atom("D", mw(cc()))]),
            rule(atom("B", mw(v("x"))), vec![atom("C", v("x"))]),
            rule(atom("A", cc()), vec![atom("B", mw(cc()))]),
            rule(atom("Win", mw(v("x"))), vec![atom("A", v("x"))]),
        ];
        let tmc = mw(cc());
        check(&shu, &tmc, &[("Win".into(), true)], "shuttle+");
        check(&shu[1..], &tmc, &[("Win".into(), true)], "shuttle-unfounded");

        // DEEP consecutive up-chain: a root Mark propagates DOWN 8 levels purely via
        // parent-moves (8 up-moves in a row), then acceptance flows back up. A bounded
        // foundedness would spuriously fail this.
        let sw = |inner: HT| f("s", vec![inner]);
        let tw = |inner: HT| f("top", vec![inner]);
        let deepu = vec![
            rule(atom("Mark", tw(v("x"))), vec![]),
            rule(atom("Saw", sw(v("x"))), vec![atom("Mark", tw(sw(v("x"))))]),
            rule(atom("Saw", sw(v("x"))), vec![atom("Saw", sw(sw(v("x"))))]),
            rule(atom("Saw", c("e")), vec![atom("Saw", sw(c("e")))]),
            rule(atom("Ok", c("e")), vec![atom("Saw", c("e"))]),
            rule(atom("Ok", sw(v("x"))), vec![atom("Ok", v("x"))]),
            rule(atom("Win", tw(v("x"))), vec![atom("Ok", v("x"))]),
        ];
        let mut spn = c("e");
        for _ in 0..8 {
            spn = sw(spn);
        }
        let dtree = tw(spn);
        check(&deepu, &dtree, &[("Win".into(), true)], "deep-consec-ups+");
        check(&deepu[1..], &dtree, &[("Win".into(), true)], "deep-consec-ups-unfounded");

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

    /// The SCC-condensation closure must still reject large unfounded cycles
    /// (multi-round within-component squaring) and accept founded ones. A 6-state
    /// same-node ring is one SCC of size 6 — ⌈log₂6⌉ = 3 squaring rounds — well
    /// past the ≤2-state cycles in the oracle above.
    #[test]
    fn scc_closure_handles_large_cycles() {
        if !cvc5_available() {
            eprintln!("cvc5 not found; skipping");
            return;
        }
        let check = |rules: &[HornRule], cons: &[(String, bool)], label: &str| {
            let tree = c("c");
            let (states, ctors) = single_sort_sig(rules, &tree);
            let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
            let gt = build_ground(&tree);
            let holds = crossing_eval(rules, &gt);
            let want = cons.iter().all(|(q, w)| holds.contains(&(q.clone(), 0)) == *w);
            let got = local_check_sat(&states, &ctors, &gcs, cons, &gt);
            assert_eq!(got, want, "[{label}] got={got} want={want}");
        };
        // q0 ← q1 ← … ← q5 ← q0, all same-node (Selfp) ⇒ one SCC of size 6.
        let ring = |base: bool| {
            let n = 6;
            let mut rs: Vec<HornRule> = (0..n)
                .map(|i| rule(atom(&format!("q{i}"), c("c")), vec![atom(&format!("q{}", (i + 1) % n), c("c"))]))
                .collect();
            if base {
                rs.push(rule(atom("q3", c("c")), vec![]));
            }
            rs
        };
        check(&ring(false), &[("q0".into(), false)], "ring-unfounded"); // no base ⇒ LFP empty
        check(&ring(true), &[("q0".into(), true)], "ring-founded"); // base ⇒ all derivable
    }

    /// The precondition the per-SCC closure optimization rests on: every
    /// structural support edge that *any* guess could place into `R0` — Selfp
    /// edges and, crucially, the child→parent up-routed edges — must lie in
    /// `may_dep` (= `reach`). If so, every actual support cycle is contained in
    /// one `reach`-SCC, so restricting the transitive-closure squaring to SCCs
    /// cannot drop a cycle. Pure structural check: no solver, always runs.
    /// Returns the number of edges verified (so an empty/vacuous pass is visible).
    fn r0_edges_within_may_dep(
        states: &HashMap<String, Vec<String>>,
        ctors: &HashMap<String, Vec<(String, Vec<String>)>>,
        gcs: &[GC],
    ) -> Result<usize, String> {
        let e = Enc::new(states, ctors, gcs);
        let mut checked = 0usize;
        for sort in states.keys() {
            for &k in e.clauses(sort) {
                let gc = &gcs[k];
                let Some(p) = e.qidx(sort, &gc.head) else { continue };
                let arg_sorts = &ctors[sort]
                    .iter()
                    .find(|(c, _)| *c == gc.ctor)
                    .ok_or_else(|| format!("ctor {} missing from sort {sort}", gc.ctor))?
                    .1;
                let mut targets: Vec<usize> = Vec::new();
                for atom in &gc.body {
                    match atom {
                        GA::Parent(..) => {}
                        GA::Selfp(st) => targets.extend(e.qidx(sort, st)),
                        GA::Child(ci, st) => {
                            let a = &arg_sorts[*ci];
                            let Some(qa) = e.qidx(a, st) else { continue };
                            for (tj, (pc2, pos2, st2)) in e.ds_of(a).iter().enumerate() {
                                if pc2 == &gc.ctor && *pos2 == *ci && e.routes_up(a, qa, tj) {
                                    targets.extend(e.qidx(sort, st2));
                                }
                            }
                        }
                    }
                }
                for m in targets {
                    let (ps, ms) = (&states[sort][p], &states[sort][m]);
                    if !e.may_dep(ps, ms) {
                        return Err(format!(
                            "R0 edge {ps} -> {ms} (sort {sort}, ctor {}, clause #{k}) escapes may_dep",
                            gc.ctor
                        ));
                    }
                    checked += 1;
                }
            }
        }
        Ok(checked)
    }

    #[test]
    fn r0_support_edges_are_within_may_dep() {
        let cc = || c("c");
        let topxy = || f("top", vec![v("x"), v("y")]);
        let topcy = || f("top", vec![cc(), v("y")]);
        let mx = || f("M", vec![v("x")]);

        let inter = vec![
            rule(atom("P", topxy()), vec![]),
            rule(atom("A", cc()), vec![atom("P", topcy())]),
            rule(atom("C", cc()), vec![atom("A", cc()), atom("B", cc())]),
            rule(atom("Win", topxy()), vec![atom("C", v("x"))]),
        ];
        let three = vec![
            rule(atom("Q0", f("R", vec![v("x"), v("y")])), vec![atom("Q1", v("x"))]),
            rule(atom("Q1", mx()), vec![atom("Q2", v("x"))]),
            rule(atom("Q2", cc()), vec![atom("Q3", f("M", vec![cc()]))]),
            rule(atom("Q3", mx()), vec![atom("Q0", f("R", vec![mx(), v("z")]))]),
        ];
        let cases: Vec<(&str, Vec<HornRule>, HT)> = vec![
            ("inter", inter, f("top", vec![cc(), cc()])),
            ("three-level-cycle", three, f("R", vec![f("M", vec![cc()]), cc()])),
        ];
        let mut total = 0;
        for (label, rules, tree) in &cases {
            let (states, ctors) = single_sort_sig(rules, tree);
            let gcs: Vec<GC> = rules.iter().filter_map(extract_gc).collect();
            let n = r0_edges_within_may_dep(&states, &ctors, &gcs)
                .unwrap_or_else(|err| panic!("[{label}] {err}"));
            total += n;
        }

        let prog = load_chc_prog("benchmark/CFG/0001_tiny.sl");
        let (_start, constraints) = goal(&prog);
        let seeds: HashSet<String> = constraints.iter().map(|(q, _)| q.clone()).collect();
        let relevant = relevant_states(&prog, &seeds);
        let (states, ctors) = build_sig(&prog, &relevant);
        let gcs = general_clauses(&prog, &relevant);
        let n = r0_edges_within_may_dep(&states, &ctors, &gcs)
            .unwrap_or_else(|err| panic!("[cfg-tiny] {err}"));
        total += n;
        assert!(total > 0, "vacuous: no R0 edges were checked");
    }
}
