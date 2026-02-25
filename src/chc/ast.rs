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
}

/// Pattern for P(ctor(x,y,...)) — used for push/pop matching.
#[derive(Debug, Clone)]
pub struct CtorPat {
    pub pred: String,
    pub ctor: String,
    pub vars: Vec<String>,
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

    pub fn substitute(&self, subst: &HashMap<String, String>) -> Self {
        match self {
            HornTerm::Var(name) => subst
                .get(name)
                .cloned()
                .map(HornTerm::Var)
                .unwrap_or_else(|| self.clone()),
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
            HornTerm::App { fun, args } => SyGuSTerm::Application(
                Identifier::Symbol(fun.clone()),
                args.iter().map(HornTerm::to_sygus).collect(),
            ),
        }
    }
}

impl CtorPat {
    pub fn matches(&self, other: &CtorPat) -> bool {
        self.pred == other.pred && self.ctor == other.ctor && self.vars.len() == other.vars.len()
    }
}

impl HornAtom {
    /// Substitutes variables in the atom's arg.
    pub fn substitute(&self, subst: &HashMap<String, String>) -> Self {
        HornAtom {
            pred: self.pred.clone(),
            arg: self.arg.substitute(subst),
        }
    }

    /// Returns `Some(CtorPat)` when arg is `ctor(x,y,...)`.
    pub fn as_ctor_pat(&self) -> Option<CtorPat> {
        match &self.arg {
            HornTerm::App { fun, args } => {
                let mut vars = Vec::with_capacity(args.len());
                for a in args {
                    if let HornTerm::Var(name) = a {
                        vars.push(name.clone());
                    } else {
                        return None;
                    }
                }
                Some(CtorPat {
                    pred: self.pred.clone(),
                    ctor: fun.clone(),
                    vars,
                })
            }
            _ => None,
        }
    }
}

impl HornRule {
    /// Returns `Some(CtorPat)` when the head has constructor-app form (push rule).
    pub fn push_head(&self) -> Option<CtorPat> {
        self.head.as_ctor_pat()
    }

    /// Returns body atoms that are constructor-apps, with their indices (pop sites).
    pub fn pop_sites(&self) -> Vec<(usize, CtorPat)> {
        self.body_atoms
            .iter()
            .enumerate()
            .filter_map(|(idx, atom)| atom.as_ctor_pat().map(|p| (idx, p)))
            .collect()
    }

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
