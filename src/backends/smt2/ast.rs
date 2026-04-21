use derive_more::Display;
use std::collections::{HashMap, HashSet};
use sygus_parser::{DTConsDecl, Identifier, Sort, SortDecl, SyGuSTerm};

/// Minimal term representation for CHC: variables and constructor applications only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum HornTerm {
    #[display(fmt = "{}", _0)]
    Var(String),
    #[display(
        fmt = "({} {})",
        fun,
        "args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    App { fun: String, args: Vec<HornTerm> },
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "({} {})", pred, arg)]
pub struct HornAtom {
    pub pred: String,
    pub arg: HornTerm,
}

#[derive(Debug, Clone, Display)]
#[display(
    fmt = "(rule (=> (and {}) {}))",
    "body_atoms.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")",
    head
)]
pub struct HornRule {
    pub body_atoms: Vec<HornAtom>,
    pub head: HornAtom,
}

#[derive(Debug, Clone, Display)]
#[display(
    fmt = "(query (and {}))",
    "body_atoms.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct HornQuery {
    pub body_atoms: Vec<HornAtom>,
}

#[derive(Debug, Clone, Display)]
pub enum HornDecl {
    #[display(fmt = "(set-logic {})", _0)]
    SetLogic(String),
    #[display(fmt = "(set-option {} {})", _0, _1)]
    SetOption(String, SyGuSTerm), // Uses SyGuSTerm for literals etc.
    #[display(
        fmt = "(declare-rel {} {})",
        _0,
        "_1.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    DeclareRel(String, Vec<Sort>),
    #[display(fmt = "(declare-var {} {})", _0, _1)]
    DeclareVar(String, Sort),
    #[display(fmt = "(declare-const {} {})", _0, _1)]
    DeclareConst(String, Sort),
    #[display(fmt = "(declare-datatypes {} {})", 
    "_0.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(\" \")",
    "_1.iter().map(|d| d.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\" \")).collect::<Vec<_>>().join(\" \")")
    ]
    DeclareDatatypes(Vec<SortDecl>, Vec<Vec<DTConsDecl>>),
    #[display(fmt = "get-proof")]
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
}

impl HornRule {
    /// Returns true if the rule has a body atom that is a constructor application (pop clause).
    pub fn is_pop_clause(&self) -> bool {
        self.body_atoms
            .iter()
            .any(|a| matches!(&a.arg, HornTerm::App { .. }))
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
}
