use derive_more::Display;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sygus_parser::{DTConsDecl, Identifier, Sort, SortDecl, SyGuSTerm};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum HornTerm {
    #[display("{}", _0)]
    Var(String),
    #[display(
        "({} {})",
        fun,
        args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ")
    )]
    App { fun: String, args: Vec<HornTerm> },
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display("({} {})", pred, arg)]
pub struct HornAtom {
    pub pred: String,
    pub arg: HornTerm,
}

#[derive(Debug, Clone, Display)]
#[display(
    "(rule (=> (and {}) {}))",
    body_atoms.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" "),
    head
)]
pub struct HornRule {
    pub body_atoms: Vec<HornAtom>,
    pub head: HornAtom,
}

#[derive(Debug, Clone, Display)]
#[display(
    "(query (and {}))",
    body_atoms.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ")
)]
pub struct HornQuery {
    pub body_atoms: Vec<HornAtom>,
}

#[derive(Debug, Clone, Display)]
pub enum HornDecl {
    #[display("(set-logic {})", _0)]
    SetLogic(String),
    #[display("(set-option {} {})", _0, _1)]
    SetOption(String, SyGuSTerm), // Uses SyGuSTerm for literals etc.
    #[display(
        "(declare-rel {} {})",
        _0,
        _1.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
    )]
    DeclareRel(String, Vec<Sort>),
    #[display("(declare-var {} {})", _0, _1)]
    DeclareVar(String, Sort),
    #[display("(declare-const {} {})", _0, _1)]
    DeclareConst(String, Sort),
    #[display("(declare-datatypes {} {})", 
    _0.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "),
    _1.iter().map(|d| d.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" "))
    ]
    DeclareDatatypes(Vec<SortDecl>, Vec<Vec<DTConsDecl>>),
    #[display("get-proof")]
    GetProof,
}

#[derive(Debug, Clone, Default)]
pub struct HornProg {
    pub decls: Vec<HornDecl>,
    pub rules: Vec<HornRule>,
    pub queries: Vec<HornQuery>,
    pub tail: Vec<HornDecl>,
    pub predicates: HashMap<String, PredicateInfo>,
    pub aspect_state: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateInfo {
    pub state: String,
    pub polarity: PredicatePolarity,
    pub sort: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicatePolarity {
    Positive,
    Negative,
}

impl HornProg {
    pub fn register_predicate(
        &mut self,
        name: impl Into<String>,
        state: impl Into<String>,
        polarity: PredicatePolarity,
        sort: impl Into<String>,
    ) {
        self.predicates.insert(
            name.into(),
            PredicateInfo {
                state: state.into(),
                polarity,
                sort: sort.into(),
            },
        );
    }

    pub fn live_predicates(&self) -> HashSet<String> {
        let mut live = HashSet::default();
        for rule in &self.rules {
            live.insert(rule.head.pred.clone());
            for atom in &rule.body_atoms {
                live.insert(atom.pred.clone());
            }
        }
        for query in &self.queries {
            for atom in &query.body_atoms {
                live.insert(atom.pred.clone());
            }
        }
        live
    }

    /// Merge predicate states that are bisimilar — identical least-model
    /// behaviour — and rename each class to a representative. This is the exact
    /// coarsest partition: refine to a true fixpoint, seeded only on the
    /// intrinsic (polarity, sort) of each predicate; a predicate absent from
    /// `predicates` (e.g. `Goal`) is seeded uniquely so it never merges. Two
    /// predicates land in the same class iff, for every rule with that predicate
    /// as head, the rule's term shape and the *classes* of its body predicates
    /// coincide (variables alpha-canonicalised, fresh/up siblings anonymised by
    /// position). Bisimilar predicates are mutually substitutable, so renaming
    /// to a representative preserves the least model — and hence every accept/
    /// reject verdict of the downstream encoding. Returns the class count.
    pub fn quotient_bisim(&mut self) -> usize {
        use std::collections::BTreeSet;
        let preds: Vec<String> = {
            let mut v: Vec<String> = self.live_predicates().into_iter().collect();
            v.sort();
            v
        };
        let pidx: HashMap<String, usize> = preds
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, p)| (p, i))
            .collect();

        // Per-rule canonical structure, computed once.
        struct RS {
            head: usize,
            hshape: String,
            body: Vec<(usize, String)>,
        }
        let rstructs: Vec<RS> = self
            .rules
            .iter()
            .map(|r| {
                let mut vmap: HashMap<String, usize> = HashMap::default();
                number_vars(&r.head.arg, &mut vmap);
                let mut fresh: HashMap<String, usize> = HashMap::default();
                let hshape = canon_shape(&r.head.arg, &vmap, &mut fresh);
                let body = r
                    .body_atoms
                    .iter()
                    .map(|b| (pidx[&b.pred], canon_shape(&b.arg, &vmap, &mut fresh)))
                    .collect();
                RS {
                    head: pidx[&r.head.pred],
                    hshape,
                    body,
                }
            })
            .collect();

        // Seed: (polarity, sort), or a unique singleton when unknown.
        let mut class: Vec<usize> = {
            let mut m: HashMap<String, usize> = HashMap::default();
            preds
                .iter()
                .map(|p| {
                    let key = match self.predicates.get(p) {
                        Some(info) => format!("S{:?}.{}", info.polarity, info.sort),
                        None => format!("U.{p}"),
                    };
                    let n = m.len();
                    *m.entry(key).or_insert(n)
                })
                .collect()
        };
        let mut ncur = class.iter().copied().max().map_or(0, |x| x + 1);

        type Sig = BTreeSet<(String, Vec<(usize, String)>)>;
        loop {
            let mut sigs: Vec<Sig> = vec![BTreeSet::new(); preds.len()];
            for rs in &rstructs {
                let mut body: Vec<(usize, String)> =
                    rs.body.iter().map(|(pi, sh)| (class[*pi], sh.clone())).collect();
                body.sort();
                sigs[rs.head].insert((rs.hshape.clone(), body));
            }
            let mut m: HashMap<(usize, Sig), usize> = HashMap::default();
            let mut next = vec![0usize; preds.len()];
            for i in 0..preds.len() {
                let key = (class[i], std::mem::take(&mut sigs[i]));
                let n = m.len();
                next[i] = *m.entry(key).or_insert(n);
            }
            if m.len() == ncur {
                break;
            }
            class = next;
            ncur = m.len();
        }

        // Representative = lexicographically-smallest name in the class.
        let mut rep: HashMap<usize, String> = HashMap::default();
        for (i, p) in preds.iter().enumerate() {
            rep.entry(class[i])
                .and_modify(|r| {
                    if p < r {
                        *r = p.clone();
                    }
                })
                .or_insert_with(|| p.clone());
        }
        let rename: HashMap<String, String> = preds
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), rep[&class[i]].clone()))
            .collect();

        for r in &mut self.rules {
            r.head.pred = rename[&r.head.pred].clone();
            for b in &mut r.body_atoms {
                b.pred = rename[&b.pred].clone();
            }
            // A conjunction: drop duplicate body atoms exposed by the rename.
            let mut seen = HashSet::default();
            r.body_atoms.retain(|a| seen.insert((a.pred.clone(), a.arg.clone())));
        }
        for q in &mut self.queries {
            for b in &mut q.body_atoms {
                b.pred = rename[&b.pred].clone();
            }
        }
        let mut newpreds: HashMap<String, PredicateInfo> = HashMap::default();
        for (name, info) in &self.predicates {
            if let Some(r) = rename.get(name) {
                newpreds.entry(r.clone()).or_insert_with(|| info.clone());
            }
        }
        self.predicates = newpreds;

        // Drop rules made structurally identical by the rename.
        let mut seen: HashSet<String> = HashSet::default();
        self.rules.retain(|r| seen.insert(r.to_string()));

        ncur
    }
}

/// Number a term's variables by first (left-to-right) occurrence.
fn number_vars(term: &HornTerm, vmap: &mut HashMap<String, usize>) {
    match term {
        HornTerm::Var(n) => {
            let next = vmap.len();
            vmap.entry(n.clone()).or_insert(next);
        }
        HornTerm::App { args, .. } => {
            for a in args {
                number_vars(a, vmap);
            }
        }
    }
}

/// Alpha-canonical string shape of a term: head variables become `v{i}` by the
/// `vmap` numbering; any other variable (a fresh/up sibling) becomes `f{i}` by
/// order of appearance within the rule (`fresh` is shared across the rule's
/// atoms so the same sibling aligns). Constructor names are kept verbatim.
fn canon_shape(term: &HornTerm, vmap: &HashMap<String, usize>, fresh: &mut HashMap<String, usize>) -> String {
    match term {
        HornTerm::Var(n) => match vmap.get(n) {
            Some(i) => format!("v{i}"),
            None => {
                let next = fresh.len();
                let id = *fresh.entry(n.clone()).or_insert(next);
                format!("f{id}")
            }
        },
        HornTerm::App { fun, args } => {
            let inner: Vec<String> = args.iter().map(|a| canon_shape(a, vmap, fresh)).collect();
            format!("({} {})", fun, inner.join(" "))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomRole {
    Pop,
    Intersection,
}

impl HornTerm {
    pub fn var(name: impl Into<String>) -> Self {
        HornTerm::Var(name.into())
    }

    pub fn app(fun: impl Into<String>, args: Vec<HornTerm>) -> Self {
        HornTerm::App {
            fun: fun.into(),
            args,
        }
    }

    pub fn substitute(&self, subst: &HashMap<String, HornTerm>) -> Self {
        match self {
            HornTerm::Var(name) => subst.get(name).cloned().unwrap_or_else(|| self.clone()),
            HornTerm::App { fun, args } => HornTerm::App {
                fun: fun.clone(),
                args: args.iter().map(|a| a.substitute(subst)).collect(),
            },
        }
    }

    /// Insert every variable appearing in `self` into `out`.
    pub fn collect_vars(&self, out: &mut HashSet<String>) {
        match self {
            HornTerm::Var(name) => {
                out.insert(name.clone());
            }
            HornTerm::App { args, .. } => {
                for arg in args {
                    arg.collect_vars(out);
                }
            }
        }
    }

    /// Variables appearing in `self`.
    pub fn vars(&self) -> HashSet<String> {
        let mut out = HashSet::default();
        self.collect_vars(&mut out);
        out
    }

    /// True iff `self` contains a variable not in `allowed`.
    pub fn has_var_outside(&self, allowed: &HashSet<String>) -> bool {
        match self {
            HornTerm::Var(name) => !allowed.contains(name),
            HornTerm::App { args, .. } => args.iter().any(|a| a.has_var_outside(allowed)),
        }
    }

    /// Fully apply a substitution, dereferencing variable chains transitively
    /// (so `x↦y, y↦a` maps `x` to `a`). Used to build resolvents under an mgu.
    pub fn apply(&self, subst: &HashMap<String, HornTerm>) -> HornTerm {
        match self {
            HornTerm::Var(name) => match subst.get(name) {
                Some(t) => t.apply(subst),
                None => self.clone(),
            },
            HornTerm::App { fun, args } => HornTerm::App {
                fun: fun.clone(),
                args: args.iter().map(|a| a.apply(subst)).collect(),
            },
        }
    }

    /// Replace every occurrence of the subterm `target` by `with`.
    pub fn replace_subterm(&self, target: &HornTerm, with: &HornTerm) -> HornTerm {
        if self == target {
            return with.clone();
        }
        match self {
            HornTerm::Var(_) => self.clone(),
            HornTerm::App { fun, args } => HornTerm::App {
                fun: fun.clone(),
                args: args.iter().map(|a| a.replace_subterm(target, with)).collect(),
            },
        }
    }

    /// Term height: a variable or nullary constructor is 0; `f(t..)` is 1+max.
    pub fn depth(&self) -> usize {
        match self {
            HornTerm::Var(_) => 0,
            HornTerm::App { args, .. } if args.is_empty() => 0,
            HornTerm::App { args, .. } => 1 + args.iter().map(HornTerm::depth).max().unwrap_or(0),
        }
    }

    /// Occurs-check: does the variable `var` appear in `self`? (Call on a term
    /// already `apply`-resolved under the current substitution.)
    pub fn occurs(&self, var: &str) -> bool {
        match self {
            HornTerm::Var(name) => name == var,
            HornTerm::App { args, .. } => args.iter().any(|a| a.occurs(var)),
        }
    }

    /// One-way match: extend `subst` so that `self·subst == subject`. Only
    /// `self`'s variables are bound (the subject is treated as ground/fixed).
    /// Used for subsumption — it is invariant under renaming the subject.
    pub fn matches(&self, subject: &HornTerm, subst: &mut HashMap<String, HornTerm>) -> bool {
        match self {
            HornTerm::Var(name) => match subst.get(name) {
                Some(bound) => bound == subject,
                None => {
                    subst.insert(name.clone(), subject.clone());
                    true
                }
            },
            HornTerm::App { fun: pf, args: pa } => match subject {
                HornTerm::App { fun: sf, args: sa } => {
                    pf == sf
                        && pa.len() == sa.len()
                        && pa.iter().zip(sa).all(|(p, s)| p.matches(s, subst))
                }
                HornTerm::Var(_) => false,
            },
        }
    }

    /// Robinson most-general-unifier, extending `subst` in place. Returns
    /// false (and leaves `subst` partially extended) if not unifiable. Binds
    /// variables on BOTH sides — this is a real mgu, not one-way matching.
    pub fn unify(&self, other: &HornTerm, subst: &mut HashMap<String, HornTerm>) -> bool {
        let a = self.apply(subst);
        let b = other.apply(subst);
        match (&a, &b) {
            (HornTerm::Var(x), HornTerm::Var(y)) if x == y => true,
            (HornTerm::Var(x), _) => {
                if b.occurs(x) {
                    false
                } else {
                    subst.insert(x.clone(), b);
                    true
                }
            }
            (_, HornTerm::Var(y)) => {
                if a.occurs(y) {
                    false
                } else {
                    subst.insert(y.clone(), a);
                    true
                }
            }
            (
                HornTerm::App { fun: f, args: fa },
                HornTerm::App { fun: g, args: ga },
            ) => {
                if f != g || fa.len() != ga.len() {
                    return false;
                }
                fa.iter().zip(ga.iter()).all(|(x, y)| x.unify(y, subst))
            }
        }
    }

    /// Converts to SyGuSTerm for SMT emission.
    pub fn to_sygus(&self) -> SyGuSTerm {
        match self {
            HornTerm::Var(name) => SyGuSTerm::Identifier(Identifier::Symbol(name.clone())),
            HornTerm::App { fun, args } if args.is_empty() => {
                SyGuSTerm::Identifier(Identifier::Symbol(fun.clone()))
            }
            HornTerm::App { fun, args } => SyGuSTerm::Application(
                Identifier::Symbol(fun.clone()),
                args.iter().map(HornTerm::to_sygus).collect(),
            ),
        }
    }
}

impl HornAtom {
    /// Substitutes variables in the atom's arg.
    pub fn substitute(&self, subst: &HashMap<String, HornTerm>) -> Self {
        HornAtom {
            pred: self.pred.clone(),
            arg: self.arg.substitute(subst),
        }
    }

    /// Transitively apply a substitution (see [`HornTerm::apply`]).
    pub fn apply(&self, subst: &HashMap<String, HornTerm>) -> Self {
        HornAtom {
            pred: self.pred.clone(),
            arg: self.arg.apply(subst),
        }
    }

    /// Unify two atoms (same predicate + unifiable arguments), extending
    /// `subst`. See [`HornTerm::unify`].
    pub fn unify(&self, other: &HornAtom, subst: &mut HashMap<String, HornTerm>) -> bool {
        self.pred == other.pred && self.arg.unify(&other.arg, subst)
    }

    /// Classify `self` as a body atom against the head-bound variables
    /// `head_vars`. See [`AtomRole`].
    pub fn role_against(&self, head_vars: &HashSet<String>) -> AtomRole {
        if self.arg.has_var_outside(head_vars) {
            AtomRole::Pop
        } else {
            AtomRole::Intersection
        }
    }
}

impl HornRule {
    /// Variables bound by the rule's head.
    pub fn head_vars(&self) -> HashSet<String> {
        self.head.arg.vars()
    }

    pub fn is_pop_clause(&self) -> bool {
        if self.head.pred.starts_with("__guard") {
            return false;
        }
        let hv = self.head_vars();
        self.body_atoms
            .iter()
            .any(|a| matches!(&a.arg, HornTerm::App { .. }) && a.role_against(&hv) == AtomRole::Pop)
    }

    /// A copy of this rule with every variable renamed to a fresh name, so it
    /// shares no variable with any other clause (required before unifying its
    /// head with another clause's body atom). `fresh` is a running counter.
    pub fn rename_apart(&self, fresh: &mut usize) -> HornRule {
        let mut vars: HashSet<String> = HashSet::default();
        self.head.arg.collect_vars(&mut vars);
        for atom in &self.body_atoms {
            atom.arg.collect_vars(&mut vars);
        }
        let mut map: HashMap<String, HornTerm> = HashMap::default();
        for v in vars {
            let name = format!("_r{fresh}");
            *fresh += 1;
            map.insert(v, HornTerm::Var(name));
        }
        HornRule {
            head: self.head.substitute(&map),
            body_atoms: self.body_atoms.iter().map(|a| a.substitute(&map)).collect(),
        }
    }

    /// True iff `self` subsumes `other`: there is a substitution θ with
    /// `self.head·θ = other.head` and `self.body·θ ⊆ other.body` (as a set).
    /// This is the standard clause subsumption — invariant under renaming the
    /// variables of either clause AND under reordering body atoms, so it is
    /// not fooled by the fresh variable names that flattening introduces.
    pub fn subsumes(&self, other: &HornRule) -> bool {
        let mut subst: HashMap<String, HornTerm> = HashMap::default();
        if self.head.pred != other.head.pred || !self.head.arg.matches(&other.head.arg, &mut subst) {
            return false;
        }
        Self::match_body(&self.body_atoms, &other.body_atoms, subst)
    }

    /// Backtracking: can every atom of `pattern` be matched (extending `subst`)
    /// to some atom of `target`?
    fn match_body(
        pattern: &[HornAtom],
        target: &[HornAtom],
        subst: HashMap<String, HornTerm>,
    ) -> bool {
        let Some((first, rest)) = pattern.split_first() else {
            return true;
        };
        for t in target {
            if first.pred != t.pred {
                continue;
            }
            let mut s = subst.clone();
            if first.arg.matches(&t.arg, &mut s) && Self::match_body(rest, target, s) {
                return true;
            }
        }
        false
    }

    /// Returns a key for deduplication. Body is sorted for stable hashing.
    pub fn rule_key(&self) -> (HornAtom, Vec<HornAtom>) {
        let mut body: Vec<_> = self.body_atoms.clone();
        body.sort();
        (self.head.clone(), body)
    }

    pub fn is_one_way_compatible(&self) -> bool {
        match &self.head.arg {
            HornTerm::Var(head_var) => self
                .body_atoms
                .iter()
                .all(|atom| matches!(&atom.arg, HornTerm::Var(name) if name == head_var)),
            HornTerm::App { args, .. } => {
                let head_vars = args
                    .iter()
                    .map(|arg| match arg {
                        HornTerm::Var(name) => Some(name.as_str()),
                        HornTerm::App { .. } => None,
                    })
                    .collect::<Option<Vec<_>>>();
                let Some(head_vars) = head_vars else {
                    return false;
                };
                self.body_atoms.iter().all(|atom| match &atom.arg {
                    HornTerm::Var(name) => head_vars.iter().any(|head| *head == name),
                    other => other == &self.head.arg,
                })
            }
        }
    }
}

pub fn predicate_count(rules: &[HornRule]) -> usize {
    let mut preds = HashSet::default();
    for rule in rules {
        preds.insert(rule.head.pred.clone());
        for atom in &rule.body_atoms {
            preds.insert(atom.pred.clone());
        }
    }
    preds.len()
}

pub fn remove_subsumed_rules(mut rules: Vec<HornRule>) -> Vec<HornRule> {
    let mut keep = vec![true; rules.len()];
    for i in 0..rules.len() {
        if !keep[i] {
            continue;
        }
        for j in 0..rules.len() {
            if i == j || !keep[j] {
                continue;
            }
            if rules[i].subsumes(&rules[j]) {
                keep[j] = false;
            }
        }
    }
    let mut out = Vec::new();
    for (rule, keep_rule) in rules.drain(..).zip(keep.into_iter()) {
        if keep_rule {
            out.push(rule);
        }
    }
    out
}
