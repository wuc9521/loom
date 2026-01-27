use crate::chc::ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule};
use sygus_parser::{Bool, Identifier, Literal, SExpr, SyGuSTerm, Term};

fn query() -> SExpr {
    SExpr::Reserved("query".to_string())
}

fn rule() -> SExpr {
    SExpr::Reserved("rule".to_string())
}

impl HornAtom {
    pub fn to_term(&self) -> SyGuSTerm {
        if self.args.is_empty() {
            SyGuSTerm::Identifier(Identifier::Symbol(self.pred.clone()))
        } else {
            Term::app(&self.pred, self.args.clone())
        }
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
        SExpr::SExprList(vec![query(), body.to_sexpr()])
    }
}

impl HornRule {
    pub fn to_sexpr(&self) -> SExpr {
        let head = self.head.to_term();
        match body_to_term(&self.body_atoms) {
            None => SExpr::SExprList(vec![rule(), head.to_sexpr()]),
            Some(body) => {
                let implication = Bool::implies(body, head);
                SExpr::SExprList(vec![rule(), implication.to_sexpr()])
            }
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
                    SExpr::Reserved("declare-rel".to_string()),
                    SExpr::Symbol(name.to_string()),
                    args_list,
                ])
            }
            HornDecl::DeclareVar(name, sort) => SExpr::SExprList(vec![
                SExpr::Reserved("declare-var".to_string()),
                SExpr::Symbol(name.to_string()),
                sort.to_sexpr(),
            ]),
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
    out
}
