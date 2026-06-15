use derive_more::Display;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sygus_parser::{DTConsDecl, Identifier, Sort, SortDecl, SyGuSTerm};

/// Minimal term representation for CHC: variables and constructor applications only.
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
}

/// Classification of a body atom against its rule's head.
///
/// `Pop` atoms contain a variable not bound by the head — they originate
/// from `(parent n)` lowering and saturation must eliminate them.
/// `Intersection` atoms are closed in the head's variables — alternating
/// clauses (Comon 2008 §7.6.1) that must be preserved by saturation,
/// otherwise an AND-of-OR body distributes to DNF.
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

    /// True iff some body atom is a [`AtomRole::Pop`] — i.e., a constructor
    /// application carrying a variable not bound by the head. Saturation
    /// must close these.
    ///
    /// Note: intersection clauses (helper-pred bodies) also have App-shaped
    /// body atoms, but their variables are all head-bound; they are not
    /// pop clauses and survive saturation.
    pub fn is_pop_clause(&self) -> bool {
        let hv = self.head_vars();
        self.body_atoms
            .iter()
            .any(|a| matches!(&a.arg, HornTerm::App { .. }) && a.role_against(&hv) == AtomRole::Pop)
    }

    /// Returns true if self subsumes other (same head, self has subset of other's body).
    pub fn subsumes(&self, other: &HornRule) -> bool {
        if self.head != other.head {
            return false;
        }
        let other_body: HashSet<_> = other.body_atoms.iter().collect();
        self.body_atoms.iter().all(|a| other_body.contains(a))
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
