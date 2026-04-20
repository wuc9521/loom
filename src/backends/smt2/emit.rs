use crate::backends::smt2::ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm};
use std::collections::BTreeSet;
use sygus_parser::{Bool, Literal, SExpr, Sort, SortedVar, SyGuSTerm, Term};

fn assert() -> SExpr {
    SExpr::Reserved("assert".to_string())
}

fn check_sat() -> SExpr {
    SExpr::Reserved("check-sat".to_string())
}

fn forall(vars: BTreeSet<String>, body: SyGuSTerm) -> SyGuSTerm {
    if vars.is_empty() {
        return body;
    }
    Term::forall(
        vars.into_iter()
            .map(|name| SortedVar {
                name,
                sort: Sort::from_name("Term"),
            })
            .collect(),
        body,
    )
}

fn collect_term_vars(term: &HornTerm, out: &mut BTreeSet<String>) {
    match term {
        HornTerm::Var(name) => {
            out.insert(name.clone());
        }
        HornTerm::App { args, .. } => {
            for arg in args {
                collect_term_vars(arg, out);
            }
        }
    }
}

fn collect_atom_vars(atom: &HornAtom, out: &mut BTreeSet<String>) {
    collect_term_vars(&atom.arg, out);
}

impl HornAtom {
    pub fn to_term(&self) -> SyGuSTerm {
        Term::app(&self.pred, vec![self.arg.to_sygus()])
    }
}

fn body_to_term(body_atoms: &[HornAtom]) -> Option<SyGuSTerm> {
    let mut terms = Vec::new();
    for atom in body_atoms {
        terms.push(atom.to_term());
    }
    match terms.len() {
        0 => None,
        1 => Some(terms.remove(0)),
        _ => Some(Bool::and(terms)),
    }
}

impl HornQuery {
    pub fn to_sexpr(&self) -> SExpr {
        let body = body_to_term(&self.body_atoms).unwrap_or_else(|| Bool::true_());
        let mut vars = BTreeSet::new();
        for atom in &self.body_atoms {
            collect_atom_vars(atom, &mut vars);
        }
        let query = Bool::implies(body, Bool::false_());
        SExpr::SExprList(vec![assert(), forall(vars, query).to_sexpr()])
    }
}

impl HornRule {
    pub fn to_sexpr(&self) -> SExpr {
        let head = self.head.to_term();
        let rule = match body_to_term(&self.body_atoms) {
            None => head,
            Some(body) => Bool::implies(body, head),
        };
        let mut vars = BTreeSet::new();
        collect_atom_vars(&self.head, &mut vars);
        for atom in &self.body_atoms {
            collect_atom_vars(atom, &mut vars);
        }
        SExpr::SExprList(vec![assert(), forall(vars, rule).to_sexpr()])
    }

    fn to_fixedpoint_string(&self) -> String {
        let head = self.head.to_term();
        match body_to_term(&self.body_atoms) {
            None => format!("(rule {head})"),
            Some(body) => format!("(rule {})", Bool::implies(body, head)),
        }
    }
}

impl HornDecl {
    pub fn to_sexpr(&self) -> SExpr {
        match self {
            HornDecl::SetOption(key, value) => SExpr::SExprList(vec![
                SExpr::Reserved("set-option".to_string()),
                SExpr::Keyword(key.to_string()),
                value.to_sexpr(),
            ]),
            HornDecl::SetLogic(logic) => SExpr::SExprList(vec![
                SExpr::Reserved("set-logic".to_string()),
                SExpr::Symbol(logic.to_string()),
            ]),
            HornDecl::DeclareRel(name, args) => {
                let args_list = SExpr::SExprList(args.iter().map(|s| s.to_sexpr()).collect());
                SExpr::SExprList(vec![
                    SExpr::Reserved("declare-fun".to_string()),
                    SExpr::Symbol(name.to_string()),
                    args_list,
                    Sort::bool().to_sexpr(),
                ])
            }
            HornDecl::DeclareVar(name, sort) => {
                let args_list = SExpr::SExprList(Vec::new());
                SExpr::SExprList(vec![
                    SExpr::Reserved("declare-fun".to_string()),
                    SExpr::Symbol(name.to_string()),
                    args_list,
                    sort.to_sexpr(),
                ])
            }
            HornDecl::DeclareConst(name, sort) => {
                let args_list = SExpr::SExprList(Vec::new());
                SExpr::SExprList(vec![
                    SExpr::Reserved("declare-fun".to_string()),
                    SExpr::Symbol(name.to_string()),
                    args_list,
                    sort.to_sexpr(),
                ])
            }
            HornDecl::DeclareDatatypes(sorts, decls) => {
                let sorts_list = SExpr::SExprList(
                    sorts
                        .iter()
                        .map(|s| {
                            SExpr::SExprList(vec![
                                SExpr::Symbol(s.symbol.to_string()),
                                SExpr::SpecConstant(Literal::Numeral(s.arity)),
                            ])
                        })
                        .collect(),
                );
                let decls_list = SExpr::SExprList(
                    decls
                        .iter()
                        .map(|dt| {
                            SExpr::SExprList(
                                dt.iter()
                                    .map(|cons| {
                                        let mut items = Vec::new();
                                        items.push(SExpr::Symbol(cons.name.to_string()));
                                        for field in &cons.args {
                                            items.push(SExpr::SExprList(vec![
                                                SExpr::Symbol(field.name.to_string()),
                                                field.sort.to_sexpr(),
                                            ]));
                                        }
                                        SExpr::SExprList(items)
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                );
                SExpr::SExprList(vec![
                    SExpr::Reserved("declare-datatypes".to_string()),
                    sorts_list,
                    decls_list,
                ])
            }
            HornDecl::GetProof => SExpr::SExprList(vec![SExpr::Reserved("get-proof".to_string())]),
        }
    }
}

pub fn to_smt(program: &HornProg) -> Vec<SExpr> {
    let mut out = Vec::new();
    for decl in &program.decls {
        if matches!(decl, HornDecl::DeclareVar(..)) {
            continue;
        }
        out.push(decl.to_sexpr());
    }
    for rule in &program.rules {
        out.push(rule.to_sexpr());
    }
    for query in &program.queries {
        out.push(query.to_sexpr());
    }
    for cmd in &program.tail {
        out.push(cmd.to_sexpr());
    }
    out.push(SExpr::SExprList(vec![check_sat()]));
    out
}

fn decl_to_fixedpoint_string(decl: &HornDecl) -> String {
    match decl {
        HornDecl::DeclareRel(name, args) => {
            let args = args
                .iter()
                .map(|sort| sort.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            format!("(declare-rel {name} ({args}))")
        }
        HornDecl::DeclareVar(name, sort) => format!("(declare-var {name} {sort})"),
        _ => decl.to_sexpr().to_string(),
    }
}

pub fn to_fixedpoint_query_text(program: &HornProg, relation: &str) -> String {
    let mut out = Vec::new();
    for decl in &program.decls {
        out.push(decl_to_fixedpoint_string(decl));
    }
    for rule in &program.rules {
        out.push(rule.to_fixedpoint_string());
    }
    out.push(format!("(query {relation} :print-answer true)"));
    out.join("\n")
}
