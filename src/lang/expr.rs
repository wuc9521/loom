// Module: expr
use crate::{
    fol::{model::Model, signature::PredicateSpec},
    lang::symbol::*,
};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Var(usize),
    Const(String),
    Func(Symbol, Vec<Term>),
    ITE(Box<Form>, Box<Term>, Box<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Form {
    Pred(
        Symbol,    // name
        Vec<Term>, // args
    ),
    Eq(
        Box<Term>, // lhs
        Box<Term>, // rhs
    ),
    Val(bool), // true or false
    Not(Box<Form>),
    Bop(Symbol, Box<Form>, Box<Form>),
    Qop(Symbol, Symbol, Box<Form>),
    Dummy(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    SubExpr(Vec<Symbol>),
    Form(Box<Form>),
    Term(Box<Term>),
}

impl Form {
    /// get all (free and bounded) variables in the formula
    pub fn vars(&self) -> Vec<usize> {
        match self {
            Form::Pred(_, args) => args.iter().flat_map(|arg| arg.vars()).collect(),
            Form::Val(_) => vec![],
            Form::Not(e) => e.vars(),
            Form::Bop(_, lhs, rhs) => {
                let mut vars = lhs.vars();
                vars.extend(rhs.vars());
                vars
            }
            Form::Eq(lhs, rhs) => {
                let mut vars = lhs.vars();
                vars.extend(rhs.vars());
                vars
            }
            Form::Qop(_, v, e) => {
                let mut vars = e.vars();
                vars.retain(|&x| x != v.idx().unwrap());
                vars
            }
            _ => vec![],
        }
    }

    /// get all free variables in the formula
    pub fn fv(&self) -> Vec<usize> {
        match self {
            Form::Pred(_, args) => args.iter().flat_map(|arg| arg.fv()).collect(),
            Form::Val(_) => vec![],
            Form::Not(e) => e.fv(),
            Form::Bop(_, lhs, rhs) => {
                let mut vars = lhs.fv();
                vars.extend(rhs.fv());
                vars
            }
            Form::Eq(lhs, rhs) => {
                let mut vars = lhs.fv();
                vars.extend(rhs.fv());
                vars
            }
            Form::Qop(_, v, e) => {
                let mut vars = e.fv();
                vars.retain(|&x| x != v.idx().unwrap());
                vars
            }
            _ => vec![],
        }
    }

    /// Evaluate the formula with the given model and assignment
    pub fn eval(&self, model: &Model, assignment: &HashMap<usize, i32>) -> bool {
        match &self {
            Form::Val(v) => *v,
            Form::Pred(symbol, args) => {
                let arg_values: Vec<i32> =
                    args.iter().map(|arg| arg.eval(model, assignment)).collect();
                model
                    .predicate(symbol.as_str().unwrap())
                    .unwrap()
                    .holds(&arg_values)
            }
            Form::Not(e) => !e.eval(model, assignment),
            Form::Bop(op, lhs, rhs) => {
                return match op {
                    Symbol::SAnd => lhs.eval(model, assignment) && rhs.eval(model, assignment),
                    Symbol::SOr => lhs.eval(model, assignment) || rhs.eval(model, assignment),
                    _ => unimplemented!("Operator not supported: {}", op),
                }
            }
            Form::Eq(lhs, rhs) => lhs.eval(model, assignment) == rhs.eval(model, assignment),
            Form::Qop(op, var, e) => {
                return match op {
                    Symbol::SExists => {
                        let vars = e.vars();
                        let assignments = generate_assignments(&vars, model.domain.len());
                        for assignment in assignments {
                            let mut new_assignment = assignment.clone();
                            new_assignment.insert(var.idx().unwrap(), 0);
                            if e.eval(model, &new_assignment) {
                                return true;
                            }
                        }
                        false
                    }
                    Symbol::SForall => {
                        let vars = e.vars();
                        let assignments = generate_assignments(&vars, model.domain.len());
                        for assignment in assignments {
                            let mut new_assignment = assignment.clone();
                            new_assignment.insert(var.idx().unwrap(), 0);
                            if !e.eval(model, &new_assignment) {
                                return false;
                            }
                        }
                        true
                    }
                    _ => unimplemented!("Operator not supported: {}", op),
                }
            }
            _ => {
                unimplemented!("Formula not supported: {}", self);
            }
        }
    }


    pub fn size(&self) -> usize {
        match &self {
            Form::Pred(_, args) => 1 + args.iter().map(|arg| arg.size()).sum::<usize>(),
            Form::Val(_) => 1,
            Form::Not(e) => 1 + e.size(),
            Form::Bop(_, lhs, rhs) => 1 + lhs.size() + rhs.size(),
            Form::Eq(lhs, rhs) => 1 + lhs.size() + rhs.size(),
            Form::Qop(_, _, e) => 1 + e.size(),
            _ => 1,
        }
    }

    pub fn name(&self) -> String {
        match &self {
            Form::Pred(name, _) => name.as_str().unwrap().to_string(),
            _ => unimplemented!("Form name not supported: {}", self),
        }
    }
}

impl Term {
    /// get all (free and bounded) variables in the term
    pub fn vars(&self) -> Vec<usize> {
        match self {
            Term::Var(idx) => vec![*idx],
            Term::Const(_) => vec![],
            Term::Func(_, args) => args.iter().flat_map(|arg| arg.vars()).collect(),
            Term::ITE(cond, then, els) => {
                let mut vars = cond.vars();
                vars.extend(then.vars());
                vars.extend(els.vars());
                vars
            }
        }
    }

    /// get all free variables in the term
    pub fn fv(&self) -> Vec<usize> {
        match self {
            Term::Var(idx) => vec![*idx],
            Term::Const(_) => vec![],
            Term::Func(_, args) => args.iter().flat_map(|arg| arg.fv()).collect(),
            Term::ITE(cond, then, els) => {
                let mut vars = cond.fv();
                vars.extend(then.fv());
                vars.extend(els.fv());
                vars
            }
        }
    }

    /// Evaluate the term with the given model and assignment
    pub fn eval(&self, model: &Model, assignment: &HashMap<usize, i32>) -> i32 {
        match &self {
            Term::Const(name) => model.constant(name).unwrap().value,
            Term::Var(idx) => *assignment.get(idx).unwrap_or(&-1),
            Term::Func(symbol, args) => {
                let arg_values: Vec<i32> =
                    args.iter().map(|arg| arg.eval(model, assignment)).collect();
                model
                    .function(symbol.as_str().unwrap())
                    .unwrap()
                    .apply(&arg_values)
                    .unwrap_or(-1)
            }
            Term::ITE(cond, then, r#else) => {
                if cond.eval(model, assignment) {
                    then.eval(model, assignment)
                } else {
                    r#else.eval(model, assignment)
                }
            }
        }
    }

    /// name of a term (partially implemented)
    pub fn size(&self) -> usize {
        match &self {
            Term::Var(_) | Term::Const(_) => 1,
            Term::Func(_, args) => 1 + args.iter().map(|arg| arg.size()).sum::<usize>(),
            Term::ITE(cond, then, els) => 3 + cond.size() + then.size() + els.size(),
        }
    }

    /// name of a term (partially implemented)
    pub fn name(&self) -> String {
        match &self {
            Term::Var(idx) => format!("x{}", idx),
            Term::Const(name) => name.clone(),
            Term::Func(name, _) => name.as_str().unwrap().to_string(),
            _ => unimplemented!("Term name not supported: {}", self),
        }
    }
}

impl Expr {
    /// size of a expr.
    pub fn size(&self) -> usize {
        match &self {
            Expr::Form(f) => f.size(),
            Expr::Term(t) => t.size(),
            Expr::SubExpr(vec) => vec.len(),
        }
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Term {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size().cmp(&other.size())
    }
}

impl Ord for Expr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size().cmp(&other.size())
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Term::Var(idx) => write!(f, "x{}", idx),
            Term::Const(name) => write!(f, "{}", name),
            Term::Func(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(
                        f,
                        "{}({})",
                        name,
                        args.iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                }
            }
            Term::ITE(cond, then, els) => write!(
                f,
                "{} ({}, {}, {})",
                Symbol::SITE,
                cond,
                then,
                els
            ),
        }
    }
}

impl std::fmt::Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Form::Pred(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(
                        f,
                        "{}({})",
                        name,
                        args.iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                }
            }
            Form::Eq(lhs, rhs) => write!(f, "{}{}{}", lhs, Symbol::SEqual, rhs),
            Form::Val(op) => write!(f, "{}", if *op { Symbol::STrue } else { Symbol::SFalse }),
            Form::Not(e) => write!(f, "{}({})", Symbol::SNeg, e),
            Form::Bop(op, lhs, rhs) => {
                write!(f, "{}{}{}", lhs, op, rhs)
            }
            Form::Qop(op, var, e) => {
                write!(f, "{}{}({})", op, var, e)
            }
            Form::Dummy(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Expr::Form(form) => write!(f, "{}", form),
            Expr::Term(term) => write!(f, "{}", term),
            Expr::SubExpr(symbols) => write!(
                f,
                "{}",
                symbols
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            ),
        }
    }
}

/// Convert the assignment to an array representation
fn assignment_to_array(assignment: &HashMap<usize, i32>, k: usize) -> Vec<usize> {
    let mut result = vec![0; k];
    for (var_idx, &value) in assignment {
        result[*var_idx] = value as usize;
    }
    result
}

/// generate all possible assignments for the given variables
fn generate_assignments(vars: &Vec<usize>, domain_size: usize) -> Vec<HashMap<usize, i32>> {
    let mut assignments = Vec::new();
    let vars: Vec<_> = vars.iter().collect();

    fn generate_recursive(
        vars: &[&usize],
        current: &mut HashMap<usize, i32>,
        assignments: &mut Vec<HashMap<usize, i32>>,
        domain_size: usize,
        index: usize,
    ) {
        if index == vars.len() {
            assignments.push(current.clone());
            return;
        }

        for value in 0..domain_size {
            current.insert(vars[index].clone(), value as i32);
            generate_recursive(vars, current, assignments, domain_size, index + 1);
        }
    }

    let mut current = HashMap::new();
    generate_recursive(&vars, &mut current, &mut assignments, domain_size, 0);
    assignments
}

/// Generate all possible atoms from the given terms and predicates
pub fn generate_all_atoms(
    terms: &[Term],
    predicates: &HashMap<String, PredicateSpec>,
) -> Vec<Form> {
    let mut atoms = Vec::new();
    atoms.push(Form::Val(true));
    atoms.push(Form::Val(false));
    for i in 0..terms.len() {
        for j in i + 1..terms.len() {
            atoms.push(Form::Eq(
                Box::new(terms[i].clone()),
                Box::new(terms[j].clone()),
            ));
        }
    }
    for (pred_name, pred_spec) in predicates {
        let term_combinations = generate_term_combinations(terms, pred_spec.arity);
        for args in term_combinations {
            atoms.push(Form::Pred(Symbol::SPred(pred_name.clone()), args.clone()));
        }
    }
    atoms
}

/// generate all possible combinations of terms with the given arity
fn generate_term_combinations(terms: &[Term], arity: usize) -> Vec<Vec<Term>> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    fn generate_recursive(
        terms: &[Term],
        current: &mut Vec<Term>,
        results: &mut Vec<Vec<Term>>,
        arity: usize,
    ) {
        if current.len() == arity {
            results.push(current.clone());
            return;
        }
        for term in terms {
            current.push(term.clone());
            generate_recursive(terms, current, results, arity);
            current.pop();
        }
    }
    generate_recursive(terms, &mut current, &mut results, arity);
    results
}
