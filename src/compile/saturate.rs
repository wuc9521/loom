use crate::backends::smt2::ast::{HornAtom, HornDecl, HornProg, HornRule, HornTerm};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::BTreeSet;

type Term = HornTerm;
type Subst = HashMap<String, Term>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Atom {
    pred: String,
    arg: Option<Term>,
}

#[derive(Clone)]
struct Clause {
    head: Atom,
    body: Vec<Atom>,
    origin: String,
}

struct Sig {
    cons: HashMap<String, (Vec<String>, String)>,
}

impl Sig {
    fn arg_sorts(&self, f: &str) -> Option<&[String]> {
        self.cons.get(f).map(|(a, _)| a.as_slice())
    }
    fn result_sort(&self, f: &str) -> Option<&str> {
        self.cons.get(f).map(|(_, r)| r.as_str())
    }
}

// ===========================================================================
// term helpers
// ===========================================================================
fn is_var(t: &Term) -> bool {
    matches!(t, HornTerm::Var(_))
}

fn term_vars(t: &Term) -> HashSet<String> {
    t.vars()
}

// ===========================================================================
// atom / clause helpers
// ===========================================================================
fn atom_vars(a: &Atom) -> HashSet<String> {
    match &a.arg {
        None => HashSet::default(),
        Some(t) => term_vars(t),
    }
}

fn atom_apply(s: &Subst, a: &Atom) -> Atom {
    Atom {
        pred: a.pred.clone(),
        arg: a.arg.as_ref().map(|t| t.apply(s)),
    }
}

fn head_vars(c: &Clause) -> HashSet<String> {
    atom_vars(&c.head)
}

fn floating_vars(c: &Clause) -> HashSet<String> {
    let hv = head_vars(c);
    let mut bv = HashSet::default();
    for b in &c.body {
        bv.extend(atom_vars(b));
    }
    bv.retain(|v| !hv.contains(v));
    bv
}

fn normalize_clause(c: &Clause) -> Clause {
    // dedup body atoms preserving order
    let mut seen: Vec<Atom> = Vec::new();
    for b in &c.body {
        if !seen.contains(b) {
            seen.push(b.clone());
        }
    }
    // canonical variable renaming by first occurrence
    let mut mapping: HashMap<String, String> = HashMap::default();
    let mut order = 0usize;
    fn canon(t: &Term, mapping: &mut HashMap<String, String>, order: &mut usize) -> Term {
        match t {
            HornTerm::Var(name) => {
                let n = mapping.entry(name.clone()).or_insert_with(|| {
                    let v = format!("x{order}");
                    *order += 1;
                    v
                });
                HornTerm::Var(n.clone())
            }
            HornTerm::App { fun, args } => HornTerm::App {
                fun: fun.clone(),
                args: args.iter().map(|a| canon(a, mapping, order)).collect(),
            },
        }
    }
    let canon_atom = |a: &Atom, mapping: &mut HashMap<String, String>, order: &mut usize| Atom {
        pred: a.pred.clone(),
        arg: a.arg.as_ref().map(|t| canon(t, mapping, order)),
    };
    let head = canon_atom(&c.head, &mut mapping, &mut order);
    let body = seen
        .iter()
        .map(|b| canon_atom(b, &mut mapping, &mut order))
        .collect();
    Clause {
        head,
        body,
        origin: c.origin.clone(),
    }
}

// ---------------------------------------------------------------------------
// classification (delis-adapted: a body atom equal to the head term is an
// inert self-loop / intersection edge; only deep non-head atoms are "pop")
// ---------------------------------------------------------------------------
fn shallow_ok(b: &Atom, head_term: Option<&Term>, hv: &HashSet<String>) -> bool {
    match &b.arg {
        None => false,
        Some(arg) => {
            if let HornTerm::Var(v) = arg {
                if hv.contains(v) {
                    return true;
                }
            }
            Some(arg) == head_term
        }
    }
}

fn is_guard_def(c: &Clause) -> bool {
    c.head.arg.is_none()
}

fn is_automaton_clause(c: &Clause) -> bool {
    if c.head.arg.is_none() {
        return false;
    }
    if !floating_vars(c).is_empty() {
        return false;
    }
    if c.body.iter().any(|b| b.arg.is_none()) {
        return false;
    }
    let hv = head_vars(c);
    let ht = c.head.arg.as_ref();
    c.body.iter().all(|b| shallow_ok(b, ht, &hv))
}

// ===========================================================================
// LPO ordering
// ===========================================================================
struct Prec {
    rank: HashMap<String, i64>,
}

impl Prec {
    fn rank(&self, sym: &str) -> i64 {
        if let Some(r) = self.rank.get(sym) {
            return *r;
        }
        if sym.starts_with("__guard[") {
            return -1;
        }
        0
    }
}

fn occurs(v: &str, t: &Term) -> bool {
    match t {
        HornTerm::Var(name) => name == v,
        HornTerm::App { args, .. } => args.iter().any(|a| occurs(v, a)),
    }
}

fn lpo_gt(s: &Term, t: &Term, prec: &Prec) -> bool {
    if s == t {
        return false;
    }
    if let HornTerm::Var(tv) = t {
        return !is_var(s) && occurs(tv, s);
    }
    if is_var(s) {
        return false;
    }
    let (sf, sa) = match s {
        HornTerm::App { fun, args } => (fun, args),
        _ => unreachable!(),
    };
    let (tf, ta) = match t {
        HornTerm::App { fun, args } => (fun, args),
        _ => unreachable!(),
    };
    // (1) some immediate subterm s_i >= t
    for si in sa {
        if si == t || lpo_gt(si, t, prec) {
            return true;
        }
    }
    let (ps, pt) = (prec.rank(sf), prec.rank(tf));
    // (2) root(s) > root(t) and s > every t_j
    if ps > pt {
        return ta.iter().all(|tj| lpo_gt(s, tj, prec));
    }
    // (3) same root: s > every t_j and args lex-greater
    if ps == pt && sf == tf {
        if !ta.iter().all(|tj| lpo_gt(s, tj, prec)) {
            return false;
        }
        for (a, b) in sa.iter().zip(ta.iter()) {
            if a == b {
                continue;
            }
            return lpo_gt(a, b, prec);
        }
        return false;
    }
    false
}

fn atom_term(a: &Atom) -> Term {
    match &a.arg {
        None => HornTerm::App {
            fun: a.pred.clone(),
            args: Vec::new(),
        },
        Some(arg) => HornTerm::App {
            fun: a.pred.clone(),
            args: vec![arg.clone()],
        },
    }
}

fn atom_gt(a: &Atom, b: &Atom, prec: &Prec) -> bool {
    lpo_gt(&atom_term(a), &atom_term(b), prec)
}

fn is_maximal(idx: usize, atoms: &[Atom], prec: &Prec) -> bool {
    !atoms
        .iter()
        .enumerate()
        .any(|(i, x)| i != idx && atom_gt(x, &atoms[idx], prec))
}

fn is_strictly_maximal(idx: usize, atoms: &[Atom], prec: &Prec) -> bool {
    atoms
        .iter()
        .enumerate()
        .all(|(i, x)| i == idx || atom_gt(&atoms[idx], x, prec))
}

fn build_precedence(clauses: &[Clause], sig: &Sig) -> Prec {
    // states = all predicates with a non-None arg, minus data symbols
    let mut states: BTreeSet<String> = BTreeSet::new();
    for c in clauses {
        if c.head.arg.is_some() && !sig.cons.contains_key(&c.head.pred) {
            states.insert(c.head.pred.clone());
        }
        for b in &c.body {
            if b.arg.is_some() && !sig.cons.contains_key(&b.pred) {
                states.insert(b.pred.clone());
            }
        }
    }
    let states: Vec<String> = states.into_iter().collect();
    let state_set: HashSet<String> = states.iter().cloned().collect();

    // constraints head -> body for shallow (var-headed) input clauses
    let mut succ: HashMap<String, BTreeSet<String>> = HashMap::default();
    let mut indeg: HashMap<String, usize> = states.iter().map(|s| (s.clone(), 0)).collect();
    for c in clauses {
        let Some(HornTerm::Var(_)) = c.head.arg.as_ref() else {
            continue;
        };
        for b in &c.body {
            if let Some(arg) = &b.arg {
                if is_var(arg)
                    && b.pred != c.head.pred
                    && state_set.contains(&b.pred)
                {
                    succ.entry(c.head.pred.clone())
                        .or_default()
                        .insert(b.pred.clone());
                }
            }
        }
    }
    for bs in succ.values() {
        for b in bs {
            if let Some(d) = indeg.get_mut(b) {
                *d += 1;
            }
        }
    }
    // Kahn: indeg-0 heads first
    let mut order: Vec<String> = Vec::new();
    let mut avail: Vec<String> = states
        .iter()
        .filter(|s| indeg[*s] == 0)
        .cloned()
        .collect();
    avail.sort();
    let mut indeg2 = indeg.clone();
    while let Some(s) = (!avail.is_empty()).then(|| avail.remove(0)) {
        order.push(s.clone());
        if let Some(bs) = succ.get(&s) {
            for b in bs {
                let e = indeg2.get_mut(b).unwrap();
                *e -= 1;
                if *e == 0 {
                    avail.push(b.clone());
                    avail.sort();
                }
            }
        }
    }
    for s in &states {
        if !order.contains(s) {
            order.push(s.clone());
        }
    }

    // ranks: guards (lowest) < states (reverse topo) < data symbols
    let mut rank: HashMap<String, i64> = HashMap::default();
    let base = 0i64;
    for (i, s) in order.iter().rev().enumerate() {
        rank.insert(s.clone(), base + 1 + i as i64);
    }
    let state_top = base + order.len() as i64;
    let mut data: Vec<String> = sig.cons.keys().cloned().collect();
    data.sort();
    for (i, f) in data.iter().enumerate() {
        rank.insert(f.clone(), state_top + 1 + i as i64);
    }
    Prec { rank }
}

// ===========================================================================
// flatten (only fires on nested heads; delis heads are depth<=1 so identity)
// ===========================================================================
struct Fresh(usize);
impl Fresh {
    fn var(&mut self, base: &str) -> Term {
        self.0 += 1;
        HornTerm::Var(format!("{base}#{}", self.0))
    }
    fn state(&mut self) -> String {
        self.0 += 1;
        format!("__N{}", self.0)
    }
}

fn flatten(clauses: &[Clause], fresh: &mut Fresh) -> Vec<Clause> {
    let mut out = Vec::new();
    for c in clauses {
        match &c.head.arg {
            // guard def or variable head pass through unchanged
            None => out.push(c.clone()),
            Some(t) if is_var(t) => out.push(c.clone()),
            // every App head (incl. nullary) goes through _flatten, which keeps
            // only body atoms whose arg is a variable of the (sub)term
            Some(t) => flatten_into(&c.head.pred, t, &c.body, fresh, &mut out),
        }
    }
    out
}


/// Keep body atoms that are child-var refs of `node` or self-loops on `node`
/// (a body atom whose arg is the whole node term — a delis intersection edge).
/// Deep non-head atoms (pop / parent-navigation) are dropped.
fn kept_for(body: &[Atom], node: &Term) -> Vec<Atom> {
    let vars = term_vars(node);
    body.iter()
        .filter(|b| match &b.arg {
            Some(HornTerm::Var(v)) => vars.contains(v),
            Some(other) => other == node,
            None => false,
        })
        .cloned()
        .collect()
}

fn flatten_into(pred: &str, term: &Term, body: &[Atom], fresh: &mut Fresh, out: &mut Vec<Clause>) {
    // base case: variable or nullary constructor
    let args = match term {
        HornTerm::App { args, .. } if !args.is_empty() => args,
        _ => {
            out.push(normalize_clause(&Clause {
                head: Atom { pred: pred.to_string(), arg: Some(term.clone()) },
                body: kept_for(body, term),
                origin: "flatten".into(),
            }));
            return;
        }
    };
    let fun = match term {
        HornTerm::App { fun, .. } => fun,
        _ => unreachable!(),
    };
    let mut new_args = Vec::new();
    let mut top_body: Vec<Atom> = Vec::new();
    for a in args {
        let avars = term_vars(a);
        let atoms_here: Vec<Atom> = body
            .iter()
            .filter(|b| match &b.arg {
                Some(HornTerm::Var(v)) => avars.contains(v),
                _ => false,
            })
            .cloned()
            .collect();
        if is_var(a) || matches!(a, HornTerm::App { args, .. } if args.is_empty()) {
            new_args.push(a.clone());
            top_body.extend(atoms_here);
        } else {
            let v = fresh.var("y");
            let st = fresh.state();
            flatten_into(&st, a, &atoms_here, fresh, out);
            if let HornTerm::Var(vn) = &v {
                top_body.push(Atom { pred: st, arg: Some(HornTerm::Var(vn.clone())) });
            }
            new_args.push(v);
        }
    }
    // keep self-loop atoms on the (reconstructed) node term
    let new_head = HornTerm::App { fun: fun.clone(), args: new_args };
    for b in body {
        if (b.arg.as_ref() == Some(&new_head) || b.arg.as_ref() == Some(term))
            && !top_body.contains(b)
        {
            top_body.push(b.clone());
        }
    }
    out.push(normalize_clause(&Clause {
        head: Atom { pred: pred.to_string(), arg: Some(new_head) },
        body: top_body,
        origin: "flatten".into(),
    }));
}

// ===========================================================================
// eager epsilon-splitting into product-state guards
// ===========================================================================
fn guard_name(preds: &BTreeSet<String>) -> String {
    let parts: Vec<&str> = preds.iter().map(|s| s.as_str()).collect();
    format!("__guard[{}]", parts.join("&"))
}

fn eps_split(c: &Clause) -> Vec<Clause> {
    let floats = floating_vars(c);
    if c.head.arg.is_none() || floats.is_empty() {
        return vec![normalize_clause(c)];
    }
    // floating vars that occur in a deep (non-var) body atom are left alone
    let mut deep_vars: HashSet<String> = HashSet::default();
    for b in &c.body {
        if let Some(arg) = &b.arg {
            if !is_var(arg) {
                for v in term_vars(arg) {
                    if floats.contains(&v) {
                        deep_vars.insert(v);
                    }
                }
            }
        }
    }
    let pure: HashSet<String> = floats.difference(&deep_vars).cloned().collect();
    if pure.is_empty() {
        return vec![normalize_clause(c)];
    }
    let mut kept: Vec<Atom> = Vec::new();
    let mut by_var: HashMap<String, Vec<Atom>> = HashMap::default();
    let mut var_order: Vec<String> = Vec::new();
    for b in &c.body {
        match &b.arg {
            Some(HornTerm::Var(v)) if pure.contains(v) => {
                if !by_var.contains_key(v) {
                    var_order.push(v.clone());
                }
                by_var.entry(v.clone()).or_default().push(b.clone());
            }
            _ => kept.push(b.clone()),
        }
    }
    let mut guards: Vec<Clause> = Vec::new();
    for v in &var_order {
        let atoms = &by_var[v];
        let preds: BTreeSet<String> = atoms.iter().map(|a| a.pred.clone()).collect();
        let q = guard_name(&preds);
        kept.push(Atom { pred: q.clone(), arg: None });
        let body: Vec<Atom> = preds
            .iter()
            .map(|p| Atom { pred: p.clone(), arg: Some(HornTerm::Var(v.clone())) })
            .collect();
        guards.push(normalize_clause(&Clause {
            head: Atom { pred: q, arg: None },
            body,
            origin: "eps-split".into(),
        }));
    }
    let reduced = normalize_clause(&Clause {
        head: c.head.clone(),
        body: kept,
        origin: format!("{}+split", c.origin),
    });
    let mut out = vec![reduced];
    out.extend(guards);
    out
}

// ===========================================================================
// binary ordered resolution
// ===========================================================================
fn rename_apart(c: &Clause, fresh: &mut Fresh) -> Clause {
    let mut m: HashMap<String, String> = HashMap::default();
    fn rt(t: &Term, m: &mut HashMap<String, String>, fresh: &mut Fresh) -> Term {
        match t {
            HornTerm::Var(name) => {
                let n = m.entry(name.clone()).or_insert_with(|| {
                    fresh.0 += 1;
                    format!("{name}#{}", fresh.0)
                });
                HornTerm::Var(n.clone())
            }
            HornTerm::App { fun, args } => HornTerm::App {
                fun: fun.clone(),
                args: args.iter().map(|a| rt(a, m, fresh)).collect(),
            },
        }
    }
    let ra = |a: &Atom, m: &mut HashMap<String, String>, fresh: &mut Fresh| Atom {
        pred: a.pred.clone(),
        arg: a.arg.as_ref().map(|t| rt(t, m, fresh)),
    };
    let head = ra(&c.head, &mut m, fresh);
    let body = c.body.iter().map(|b| ra(b, &mut m, fresh)).collect();
    Clause { head, body, origin: c.origin.clone() }
}

fn resolve(main: &Clause, idx: usize, side: &Clause, fresh: &mut Fresh) -> Option<Clause> {
    let lit = &main.body[idx];
    let lit_arg = match (&lit.arg, &side.head.arg) {
        (Some(l), Some(_)) if lit.pred == side.head.pred => l,
        _ => return None,
    };
    let side = rename_apart(side, fresh);
    let side_head_arg = side.head.arg.as_ref().unwrap();
    let mut s: Subst = HashMap::default();
    if !lit_arg.unify(side_head_arg, &mut s) {
        return None;
    }
    let new_head = atom_apply(&s, &main.head);
    let mut new_body: Vec<Atom> = main
        .body
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != idx)
        .map(|(_, b)| atom_apply(&s, b))
        .collect();
    new_body.extend(side.body.iter().map(|b| atom_apply(&s, b)));
    let res = normalize_clause(&Clause {
        head: new_head,
        body: new_body,
        origin: format!("{} x {}", main.origin, side.origin),
    });
    if res.body.contains(&res.head) {
        return None; // tautology
    }
    Some(res)
}

// ---------------------------------------------------------------------------
// subsumption
// ---------------------------------------------------------------------------
fn atom_match(p: &Atom, t: &Atom, s: &Subst) -> Option<Subst> {
    if p.pred != t.pred {
        return None;
    }
    match (&p.arg, &t.arg) {
        (None, None) => Some(s.clone()),
        (Some(pa), Some(ta)) => {
            let mut s2 = s.clone();
            if pa.matches(ta, &mut s2) {
                Some(s2)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn subsumes(c: &Clause, d: &Clause) -> bool {
    let Some(s0) = atom_match(&c.head, &d.head, &HashMap::default()) else {
        return false;
    };
    fn search(i: usize, cbody: &[Atom], dbody: &[Atom], s: &Subst) -> bool {
        if i == cbody.len() {
            return true;
        }
        for datom in dbody {
            if let Some(s2) = atom_match(&cbody[i], datom, s) {
                if search(i + 1, cbody, dbody, &s2) {
                    return true;
                }
            }
        }
        false
    }
    search(0, &c.body, &d.body, &s0)
}

// ===========================================================================
// exact guard discharge: reachable types (sort-aware subset construction)
// ===========================================================================
fn reachable_types(automaton: &[Clause], sig: &Sig) -> HashSet<(String, BTreeSet<String>)> {
    let mut const_seed: HashMap<String, BTreeSet<String>> = HashMap::default();
    // pushes: (head_pred, f, [(child_pred, child_pos)], [self_loop_pred])
    // self_loop preds are node-level intersection conditions: the head fires
    // only when they already hold in this node's own type.
    let mut pushes: Vec<(String, String, Vec<(String, usize)>, Vec<String>)> = Vec::new();
    let mut eps: Vec<(String, String)> = Vec::new();
    let mut inter: Vec<(String, Vec<String>)> = Vec::new();

    for c in automaton {
        let Some(harg) = &c.head.arg else { continue };
        if c.body.is_empty() {
            if let HornTerm::App { fun, args } = harg {
                if args.is_empty() {
                    const_seed.entry(fun.clone()).or_default().insert(c.head.pred.clone());
                }
            }
            continue;
        }
        if is_var(harg) {
            let preds: Vec<String> = c
                .body
                .iter()
                .filter_map(|b| b.arg.as_ref().and(Some(b.pred.clone())))
                .collect();
            if preds.len() == 1 {
                eps.push((c.head.pred.clone(), preds[0].clone()));
            } else if !preds.is_empty() {
                inter.push((c.head.pred.clone(), preds));
            }
            continue;
        }
        if let HornTerm::App { fun, args } = harg {
            // position of each head variable
            let mut pos: HashMap<String, usize> = HashMap::default();
            for (i, v) in args.iter().enumerate() {
                if let HornTerm::Var(n) = v {
                    pos.insert(n.clone(), i);
                }
            }
            let mut obligations: Vec<(String, usize)> = Vec::new();
            let mut self_preds: Vec<String> = Vec::new();
            let mut ok = true;
            for b in &c.body {
                if b.arg.as_ref() == Some(harg) {
                    // self-loop: a condition on this node's own type
                    self_preds.push(b.pred.clone());
                } else if let Some(HornTerm::Var(v)) = &b.arg {
                    if let Some(&p) = pos.get(v) {
                        obligations.push((b.pred.clone(), p));
                    } else {
                        ok = false;
                        break;
                    }
                } else {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            if args.is_empty() {
                // nullary constructor: head fires at the constant iff its
                // self-loop conditions hold — a plain node-level intersection.
                if !self_preds.is_empty() {
                    inter.push((c.head.pred.clone(), self_preds));
                }
            } else {
                pushes.push((c.head.pred.clone(), fun.clone(), obligations, self_preds));
            }
        }
    }

    let close = |seed: BTreeSet<String>| -> BTreeSet<String> {
        let mut typ = seed;
        let mut changed = true;
        while changed {
            changed = false;
            for (p, q) in &eps {
                if typ.contains(q) && !typ.contains(p) {
                    typ.insert(p.clone());
                    changed = true;
                }
            }
            for (p, ps) in &inter {
                if ps.iter().all(|pi| typ.contains(pi)) && !typ.contains(p) {
                    typ.insert(p.clone());
                    changed = true;
                }
            }
        }
        typ
    };

    let mut types: HashSet<(String, BTreeSet<String>)> = HashSet::default();
    for (c, seed) in &const_seed {
        if let Some(rs) = sig.result_sort(c) {
            types.insert((rs.to_string(), close(seed.clone())));
        }
    }

    // rules grouped by constructor, computed once (mirrors prototype's rules_f)
    type Rule = (String, Vec<(String, usize)>, Vec<String>); // head, child obligations, self-loop preds
    let mut rules_by_f: HashMap<String, Vec<Rule>> = HashMap::default();
    for (p, f, obs, sp) in &pushes {
        rules_by_f.entry(f.clone()).or_default().push((p.clone(), obs.clone(), sp.clone()));
    }

    // Node-level type for a constructor node, given its children's types: fire
    // a push head when its child obligations are met by the combo AND its
    // self-loop conditions already hold in this node's type, interleaved with
    // the global eps / intersection edges, to a fixpoint.
    let node_type = |combo: &[&BTreeSet<String>], rules: &[Rule]| -> BTreeSet<String> {
        let mut typ: BTreeSet<String> = BTreeSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for (p, obs, sp) in rules {
                if !typ.contains(p)
                    && obs.iter().all(|(qi, idx)| combo[*idx].contains(qi))
                    && sp.iter().all(|s| typ.contains(s))
                {
                    typ.insert(p.clone());
                    changed = true;
                }
            }
            for (p, q) in &eps {
                if typ.contains(q) && !typ.contains(p) {
                    typ.insert(p.clone());
                    changed = true;
                }
            }
            for (p, ps) in &inter {
                if ps.iter().all(|pi| typ.contains(pi)) && !typ.contains(p) {
                    typ.insert(p.clone());
                    changed = true;
                }
            }
        }
        typ
    };

    let mut changed = true;
    while changed {
        changed = false;
        // group current type sets by sort (by reference, no clone)
        let mut by_sort: HashMap<&str, Vec<&BTreeSet<String>>> = HashMap::default();
        for (s, t) in &types {
            by_sort.entry(s.as_str()).or_default().push(t);
        }
        let mut fresh_types: Vec<(String, BTreeSet<String>)> = Vec::new();
        for (f, rules) in &rules_by_f {
            let Some(arg_sorts) = sig.arg_sorts(f) else { continue };
            let Some(result_sort) = sig.result_sort(f) else { continue };
            if arg_sorts.is_empty() {
                continue;
            }
            let per_pos: Vec<&Vec<&BTreeSet<String>>> =
                match arg_sorts.iter().map(|s| by_sort.get(s.as_str())).collect() {
                    Some(v) => v,
                    None => continue, // some position has no inhabited type yet
                };
            cartesian(&per_pos, &mut |combo: &[&BTreeSet<String>]| {
                let t = node_type(combo, rules);
                if !t.is_empty() && !types.contains(&(result_sort.to_string(), t.clone())) {
                    fresh_types.push((result_sort.to_string(), t));
                }
            });
        }
        for entry in fresh_types {
            if types.insert(entry) {
                changed = true;
            }
        }
    }
    types
}

/// Visit every tuple of the cartesian product of `per_pos`, calling `f` on each.
fn cartesian<'a>(
    per_pos: &[&Vec<&'a BTreeSet<String>>],
    f: &mut impl FnMut(&[&'a BTreeSet<String>]),
) {
    let mut combo: Vec<&'a BTreeSet<String>> = Vec::with_capacity(per_pos.len());
    fn go<'a>(
        per_pos: &[&Vec<&'a BTreeSet<String>>],
        i: usize,
        combo: &mut Vec<&'a BTreeSet<String>>,
        f: &mut impl FnMut(&[&'a BTreeSet<String>]),
    ) {
        if i == per_pos.len() {
            f(combo);
            return;
        }
        for &t in per_pos[i] {
            combo.push(t);
            go(per_pos, i + 1, combo, f);
            combo.pop();
        }
    }
    go(per_pos, 0, &mut combo, f);
}

fn guard_true(qpred: &str, types: &HashSet<(String, BTreeSet<String>)>) -> bool {
    let inner = qpred
        .strip_prefix("__guard[")
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or("");
    let need: BTreeSet<String> = if inner.is_empty() {
        BTreeSet::new()
    } else {
        inner.split('&').map(|s| s.to_string()).collect()
    };
    types.iter().any(|(_, t)| need.is_subset(t))
}

// ===========================================================================
// selection / premise roles
// ===========================================================================
fn selected_literal(c: &Clause, prec: &Prec) -> Option<usize> {
    if is_guard_def(c) || is_automaton_clause(c) {
        return None;
    }
    let mut atoms: Vec<Atom> = vec![c.head.clone()];
    atoms.extend(c.body.iter().cloned());
    let mut best: Option<usize> = None;
    for (i, b) in c.body.iter().enumerate() {
        if b.arg.is_none() {
            continue;
        }
        if !is_maximal(i + 1, &atoms, prec) {
            continue;
        }
        match best {
            None => best = Some(i),
            Some(bi) => {
                if atom_gt(b, &c.body[bi], prec) {
                    best = Some(i);
                }
            }
        }
    }
    best
}

fn is_side_eligible(c: &Clause, prec: &Prec) -> bool {
    if !is_automaton_clause(c) {
        return false;
    }
    let mut atoms: Vec<Atom> = vec![c.head.clone()];
    atoms.extend(c.body.iter().cloned());
    is_strictly_maximal(0, &atoms, prec)
}

// ===========================================================================
// saturation loop (indexed given-clause loop)
// ===========================================================================
fn atom_repr(a: &Atom) -> String {
    match &a.arg {
        None => a.pred.clone(),
        Some(t) => format!("{}({t})", a.pred),
    }
}

fn clause_repr(c: &Clause) -> String {
    let mut bs: Vec<String> = c.body.iter().map(atom_repr).collect();
    bs.sort();
    format!("{} :- {}", atom_repr(&c.head), bs.join(","))
}

/// Database of clauses kept across the saturation, with role indexes so each
/// resolution step only meets the few clauses that can actually resolve.
struct Db {
    clauses: Vec<Clause>,
    active: Vec<bool>,
    by_head: HashMap<String, Vec<usize>>,        // for subsumption (same head pred)
    sides: HashMap<String, Vec<usize>>,          // side-eligible clauses by head pred
    mains: HashMap<String, Vec<(usize, usize)>>, // by selected-literal pred -> (clause, lit idx)
    seen: HashSet<String>,
}

impl Db {
    fn new() -> Self {
        Db {
            clauses: Vec::new(),
            active: Vec::new(),
            by_head: HashMap::default(),
            sides: HashMap::default(),
            mains: HashMap::default(),
            seen: HashSet::default(),
        }
    }

    /// Add `c`; returns its index, or None if it was a duplicate or subsumed.
    /// Backward-subsumes (deactivates) the clauses it makes redundant.
    fn add(&mut self, c: Clause, prec: &Prec) -> Option<usize> {
        if !self.seen.insert(clause_repr(&c)) {
            return None;
        }
        if let Some(cands) = self.by_head.get(&c.head.pred) {
            if cands.iter().any(|&i| self.active[i] && subsumes(&self.clauses[i], &c)) {
                return None;
            }
        }
        if let Some(cands) = self.by_head.get(&c.head.pred) {
            for &i in cands {
                if self.active[i] && subsumes(&c, &self.clauses[i]) {
                    self.active[i] = false;
                }
            }
        }
        let idx = self.clauses.len();
        self.by_head.entry(c.head.pred.clone()).or_default().push(idx);
        if let Some(li) = selected_literal(&c, prec) {
            self.mains.entry(c.body[li].pred.clone()).or_default().push((idx, li));
        } else if is_side_eligible(&c, prec) {
            self.sides.entry(c.head.pred.clone()).or_default().push(idx);
        }
        self.clauses.push(c);
        self.active.push(true);
        Some(idx)
    }
}

fn saturate_clauses(clauses: Vec<Clause>, sig: &Sig) -> Vec<Clause> {
    let mut fresh = Fresh(0);
    let normalized: Vec<Clause> = clauses.iter().map(normalize_clause).collect();
    let flat = flatten(&normalized, &mut fresh);
    let prec = build_precedence(&flat, sig);

    let mut db = Db::new();
    // generation 0 = the flattened, eps-split input
    let mut gen: Vec<Clause> = Vec::new();
    for c in &flat {
        gen.extend(eps_split(c));
    }

    let mut true_guards: HashSet<String> = HashSet::default();
    // Process one generation, then discharge guards on the resulting automaton
    // before the next generation's resolvents enlarge it (this keeps the subset
    // construction on the small early automaton, as the prototype does).
    loop {
        let mut next: Vec<Clause> = Vec::new();
        for given in gen.drain(..) {
            let mut given = normalize_clause(&given);
            if given.body.iter().any(|b| b.arg.is_none() && true_guards.contains(&b.pred)) {
                let body = given
                    .body
                    .iter()
                    .filter(|b| !(b.arg.is_none() && true_guards.contains(&b.pred)))
                    .cloned()
                    .collect();
                given = normalize_clause(&Clause { head: given.head.clone(), body, origin: given.origin });
            }
            let role_main = selected_literal(&given, &prec);
            let role_side = role_main.is_none() && is_side_eligible(&given, &prec);
            let mut resolvents: Vec<Clause> = Vec::new();
            if let Some(li) = role_main {
                let lp = given.body[li].pred.clone();
                if let Some(sis) = db.sides.get(&lp) {
                    for &si in sis {
                        if db.active[si] {
                            if let Some(r) = resolve(&given, li, &db.clauses[si], &mut fresh) {
                                resolvents.extend(eps_split(&r));
                            }
                        }
                    }
                }
            } else if role_side {
                if let Some(mis) = db.mains.get(&given.head.pred) {
                    for &(ci, li) in mis {
                        if db.active[ci] {
                            if let Some(r) = resolve(&db.clauses[ci], li, &given, &mut fresh) {
                                resolvents.extend(eps_split(&r));
                            }
                        }
                    }
                }
            }
            if db.add(given, &prec).is_some() {
                next.extend(resolvents);
            }
        }
        let discharged = discharge(&db, sig, &mut true_guards, &mut next);
        gen = next;
        if gen.is_empty() && !discharged {
            break;
        }
    }
    db.clauses
        .into_iter()
        .zip(db.active)
        .filter(|(c, a)| *a && is_automaton_clause(c))
        .map(|(c, _)| c)
        .collect()
}

/// Decide pending product-state guards exactly (subset construction), latch the
/// non-empty ones, and re-enqueue every clause that carried a now-true guard
/// with that atom erased.  Returns true iff a new guard became true.
fn discharge(db: &Db, sig: &Sig, true_guards: &mut HashSet<String>, work: &mut Vec<Clause>) -> bool {
    let pending: Vec<&Clause> = db
        .clauses
        .iter()
        .zip(db.active.iter())
        .filter(|(c, a)| **a && c.head.arg.is_none() && !c.body.is_empty() && !true_guards.contains(&c.head.pred))
        .map(|(c, _)| c)
        .collect();
    if pending.is_empty() {
        return false;
    }
    let automaton: Vec<Clause> = db
        .clauses
        .iter()
        .zip(db.active.iter())
        .filter(|(c, a)| **a && is_automaton_clause(c))
        .map(|(c, _)| c.clone())
        .collect();
    let types = reachable_types(&automaton, sig);
    let mut newly = false;
    for c in &pending {
        if guard_true(&c.head.pred, &types) && true_guards.insert(c.head.pred.clone()) {
            newly = true;
        }
    }
    if !newly {
        return false;
    }
    for (c, a) in db.clauses.iter().zip(db.active.iter()) {
        if *a && c.body.iter().any(|b| b.arg.is_none() && true_guards.contains(&b.pred)) {
            let body = c
                .body
                .iter()
                .filter(|b| !(b.arg.is_none() && true_guards.contains(&b.pred)))
                .cloned()
                .collect();
            work.push(Clause { head: c.head.clone(), body, origin: format!("{}/g", c.origin) });
        }
    }
    true
}

// ===========================================================================
// HornProg <-> Clause conversion + public entry
// ===========================================================================
impl HornAtom {
    fn to_atom(&self) -> Atom {
        Atom { pred: self.pred.clone(), arg: Some(self.arg.clone()) }
    }
}

impl HornRule {
    fn to_clause(&self) -> Clause {
        Clause {
            head: self.head.to_atom(),
            body: self.body_atoms.iter().map(HornAtom::to_atom).collect(),
            origin: "input".into(),
        }
    }
}

impl Clause {
    fn to_rule(&self) -> HornRule {
        let lower = |a: &Atom| HornAtom {
            pred: a.pred.clone(),
            arg: a.arg.clone().expect("automaton atom has an arg"),
        };
        HornRule {
            head: lower(&self.head),
            body_atoms: self.body.iter().map(lower).collect(),
        }
    }
}

impl HornProg {
    fn term_signature(&self) -> Sig {
        let mut cons: HashMap<String, (Vec<String>, String)> = HashMap::default();
        for decl in &self.decls {
            if let HornDecl::DeclareDatatypes(sorts, groups) = decl {
                for (sort_decl, group) in sorts.iter().zip(groups.iter()) {
                    for cd in group {
                        let arg_sorts =
                            cd.args.iter().map(|sv| sv.sort.to_string()).collect();
                        cons.insert(cd.name.clone(), (arg_sorts, sort_decl.symbol.clone()));
                    }
                }
            }
        }
        Sig { cons }
    }
}

pub fn saturate(prog: &mut HornProg) {
    let sig = prog.term_signature();
    let clauses: Vec<Clause> = prog.rules.iter().map(HornRule::to_clause).collect();
    let saturated = saturate_clauses(clauses, &sig);
    prog.rules = saturated.iter().map(Clause::to_rule).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(n: &str) -> Term { HornTerm::Var(n.into()) }
    fn c(n: &str) -> Term { HornTerm::App { fun: n.into(), args: vec![] } }
    fn f(n: &str, args: Vec<Term>) -> Term { HornTerm::App { fun: n.into(), args } }
    fn at(p: &str, t: Term) -> Atom { Atom { pred: p.into(), arg: Some(t) } }
    fn cl(head: Atom, body: Vec<Atom>) -> Clause { Clause { head, body, origin: "t".into() } }

    fn tata_762() -> Vec<Clause> {
        let (x1, x2, x3, x, y) = (v("x1"), v("x2"), v("x3"), v("x"), v("y"));
        vec![
            cl(at("P1", f("f", vec![f("f", vec![x1.clone(), x2.clone()]), x3.clone()])),
               vec![at("P2", x1.clone()), at("P2", x2.clone()), at("P2", x3.clone())]),
            cl(at("P2", c("a")), vec![]),
            cl(at("P1", f("f", vec![c("a"), x.clone()])), vec![at("P2", x.clone())]),
            cl(at("P3", f("f", vec![x.clone(), y.clone()])), vec![at("P1", x.clone()), at("P2", y.clone())]),
            cl(at("P4", x.clone()), vec![at("P3", x.clone()), at("P1", x.clone())]),
            cl(at("P2", x.clone()), vec![at("P4", f("f", vec![x.clone(), y.clone()]))]),
            cl(at("P1", y.clone()), vec![at("P4", f("f", vec![x.clone(), y.clone()]))]),
        ]
    }

    fn sig_af() -> Sig {
        let mut cons = HashMap::default();
        cons.insert("a".to_string(), (vec![], "T".to_string()));
        cons.insert("f".to_string(), (vec!["T".into(), "T".into()], "T".to_string()));
        Sig { cons }
    }

    fn universe(sig: &Sig, height: usize) -> Vec<Term> {
        let mut u: Vec<Term> = sig.cons.iter().filter(|(_, (a, _))| a.is_empty())
            .map(|(n, _)| c(n)).collect();
        for _ in 0..height {
            let prev = u.clone();
            for (name, (args, _)) in &sig.cons {
                if args.is_empty() { continue; }
                let mut combos: Vec<Vec<Term>> = vec![vec![]];
                for _ in 0..args.len() {
                    let mut next = vec![];
                    for cmb in &combos { for t in &prev {
                        let mut cc = cmb.clone(); cc.push(t.clone()); next.push(cc);
                    }}
                    combos = next;
                }
                for cmb in combos {
                    let t = f(name, cmb);
                    if !u.contains(&t) { u.push(t); }
                }
            }
        }
        u
    }

    fn solve_body(body: &[Atom], model: &HashMap<String, HashSet<Term>>,
                  nullary: &HashSet<String>, s: Subst) -> Vec<Subst> {
        let Some((first, rest)) = body.split_first() else { return vec![s]; };
        let mut out = vec![];
        if first.arg.is_none() {
            if nullary.contains(&first.pred) {
                out.extend(solve_body(rest, model, nullary, s));
            }
            return out;
        }
        let pat = first.arg.as_ref().unwrap();
        if let Some(ext) = model.get(&first.pred) {
            for g in ext {
                let mut s2 = s.clone();
                if pat.matches(g, &mut s2) {
                    out.extend(solve_body(rest, model, nullary, s2));
                }
            }
        }
        out
    }

    fn least_model(clauses: &[Clause], uni: &[Term]) -> (HashMap<String, HashSet<Term>>, HashSet<String>) {
        let uset: HashSet<Term> = uni.iter().cloned().collect();
        let mut model: HashMap<String, HashSet<Term>> = HashMap::default();
        let mut nullary: HashSet<String> = HashSet::default();
        let mut changed = true;
        while changed {
            changed = false;
            for cl in clauses {
                for sub in solve_body(&cl.body, &model, &nullary, HashMap::default()) {
                    match &cl.head.arg {
                        None => { if nullary.insert(cl.head.pred.clone()) { changed = true; } }
                        Some(t) => {
                            let g = t.apply(&sub);
                            let fv: Vec<String> = g.vars().into_iter().collect();
                            // free head vars (e.g. universal clause P(x) <= ) range over the universe
                            let mut combos: Vec<Subst> = vec![HashMap::default()];
                            for fvn in &fv {
                                let mut next = vec![];
                                for pre in &combos { for u in uni {
                                    let mut m = pre.clone();
                                    m.insert(fvn.clone(), u.clone());
                                    next.push(m);
                                }}
                                combos = next;
                            }
                            for m in &combos {
                                let gg = g.apply(m);
                                if uset.contains(&gg)
                                    && model.entry(cl.head.pred.clone()).or_default().insert(gg) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        (model, nullary)
    }

    #[test]
    fn tata_762_saturates_to_oneway_and_preserves_language() {
        let input = tata_762();
        let sig = sig_af();
        let out = saturate_clauses(input.clone(), &sig);
        // every output clause is a one-way automaton clause
        assert!(out.iter().all(is_automaton_clause), "projection left non-automaton clauses");
        assert!(out.iter().all(|c| c.head.arg.is_some()), "guard def survived projection");
        // Language preserved. The two-way input may need a taller witness term
        // to place a height-k term in a state, so models are computed with
        // headroom and compared only on terms well below the model height.
        let cutoff = 1;
        let uni = universe(&sig, cutoff + 2);
        let (mi, _) = least_model(&input, &uni);
        let (mo, _) = least_model(&out, &uni);
        let restrict = |m: &HashMap<String, HashSet<Term>>, p: &str| -> HashSet<Term> {
            m.get(p).cloned().unwrap_or_default().into_iter()
                .filter(|t| t.depth() <= cutoff).collect()
        };
        for p in ["P1", "P2", "P3", "P4"] {
            assert_eq!(restrict(&mi, p), restrict(&mo, p), "language differs for {p}");
        }
    }

    fn sig_upmove() -> Sig {
        let mut cons = HashMap::default();
        cons.insert("r".to_string(), (vec![], "T".to_string()));
        cons.insert("e1".to_string(), (vec![], "T".to_string()));
        cons.insert("e2".to_string(), (vec![], "T".to_string()));
        cons.insert("top".to_string(), (vec!["T".into(), "T".into()], "T".to_string()));
        Sig { cons }
    }

    /// Minimal model of how delis compiles a FACET up-move `(parent n)`: a PUSH
    /// clause whose parent context carries a FRESH sibling variable `y`
    /// (exactly what `syntax.rs::parent_contexts` emits).
    ///
    /// Intended (real-parent) semantics: `top(x, s)` is in `Win` only if `x`'s
    /// ACTUAL sibling `s` is `Good`. Here `e2` is Good, `e1` is not. So
    /// `Win(top(r, e1))` should be FALSE — r's real sibling e1 isn't Good.
    ///
    /// The fresh-sibling encoding lets the up-move from `r` borrow the Good
    /// sibling `e2`, so `B(r)` holds and `Win(top(r, e1))` is accepted anyway.
    ///
    /// The test pins down WHERE that over-acceptance lives: it is already in the
    /// least Herbrand model of the *input* (the encoding), and `saturate`
    /// PRESERVES it (VGL is language-preserving). So saturate is faithful; the
    /// unsoundness is the fresh-sibling push clause itself.
    #[test]
    fn upmove_fresh_sibling_overapproximates_but_saturate_preserves_language() {
        let sig = sig_upmove();
        let (x, y) = (v("x"), v("y"));
        let input = vec![
            cl(at("Good", c("e2")), vec![]),
            cl(at("A", f("top", vec![x.clone(), y.clone()])), vec![at("Good", y.clone())]),
            // PUSH: head over the subterm `r`, body over the parent `top(r, y)`
            // with y a FRESH sibling not constrained to r's real sibling.
            cl(at("B", c("r")), vec![at("A", f("top", vec![c("r"), y.clone()]))]),
            cl(at("Win", f("top", vec![x.clone(), y.clone()])), vec![at("B", x.clone())]),
        ];

        let cutoff = 1;
        let uni = universe(&sig, cutoff + 2);
        let target = f("top", vec![c("r"), c("e1")]); // r's real sibling e1 is NOT Good

        let (mi, _) = least_model(&input, &uni);
        let out = saturate_clauses(input.clone(), &sig);
        let (mo, _) = least_model(&out, &uni);

        let win_in = mi.get("Win").map_or(false, |s| s.contains(&target));
        let win_out = mo.get("Win").map_or(false, |s| s.contains(&target));

        // (1) The ENCODING over-accepts: top(r, e1) wins despite e1 not being Good.
        assert!(win_in, "expected the fresh-sibling encoding to over-accept Win(top(r,e1))");
        // (2) saturate is language-preserving (VGL faithful): same verdict, and
        //     more strongly the whole Win/B languages match below the cutoff.
        assert!(win_out, "saturate dropped the (over-)accepted Win(top(r,e1)) -- not language preserving");
        let restrict = |m: &HashMap<String, HashSet<Term>>, p: &str| -> HashSet<Term> {
            m.get(p).cloned().unwrap_or_default().into_iter().filter(|t| t.depth() <= cutoff).collect()
        };
        for p in ["Good", "A", "B", "Win"] {
            assert_eq!(restrict(&mi, p), restrict(&mo, p), "language differs for {p} -- saturate not faithful");
        }
    }
}
