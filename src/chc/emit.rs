use crate::chc::ast::{Atom, Decl, Program, Query, Rule};
use sygus_parser::{
    Attribute, AttributeValue, Identifier, Index, Literal, SExpr, Sort, SortedVar, SyGuSTerm,
    VarBinding,
};

fn sexpr_reserved(name: &str) -> SExpr {
    SExpr::Reserved(name.to_string())
}

fn sexpr_symbol(name: &str) -> SExpr {
    SExpr::Symbol(name.to_string())
}

fn sexpr_list(items: Vec<SExpr>) -> SExpr {
    SExpr::SExprList(items)
}

fn sexpr_from_index(index: &Index) -> SExpr {
    match index {
        Index::Numeral(num) => SExpr::SpecConstant(Literal::Numeral(*num)),
        Index::Symbol(sym) => SExpr::Symbol(sym.clone()),
    }
}

fn sexpr_from_identifier(id: &Identifier) -> SExpr {
    match id {
        Identifier::Symbol(sym) => SExpr::Symbol(sym.clone()),
        Identifier::Indexed(sym, indices) => {
            let mut items = Vec::new();
            items.push(SExpr::Symbol(sym.clone()));
            items.extend(indices.iter().map(sexpr_from_index));
            sexpr_list(items)
        }
    }
}

fn sexpr_from_sort(sort: &Sort) -> SExpr {
    match sort {
        Sort::Simple(id) => sexpr_from_identifier(id),
        Sort::Parameterized(id, subsorts) => {
            let mut items = Vec::new();
            items.push(sexpr_from_identifier(id));
            items.extend(subsorts.iter().map(sexpr_from_sort));
            sexpr_list(items)
        }
    }
}

fn sexpr_from_sorted_var(var: &SortedVar) -> SExpr {
    sexpr_list(vec![sexpr_symbol(&var.name), sexpr_from_sort(&var.sort)])
}

fn sexpr_from_var_binding(binding: &VarBinding) -> SExpr {
    sexpr_list(vec![
        sexpr_symbol(&binding.name),
        sexpr_from_term(&binding.term),
    ])
}

fn sexpr_from_attribute_value(val: &AttributeValue) -> SExpr {
    match val {
        AttributeValue::SpecConstant(lit) => SExpr::SpecConstant(lit.clone()),
        AttributeValue::Symbol(sym) => SExpr::Symbol(sym.clone()),
        AttributeValue::SExprList(exprs) => sexpr_list(exprs.clone()),
    }
}

fn sexpr_from_attribute(attr: &Attribute) -> SExpr {
    let mut items = vec![SExpr::Keyword(attr.keyword.clone())];
    if let Some(val) = &attr.value {
        items.push(sexpr_from_attribute_value(val));
    }
    sexpr_list(items)
}

fn sexpr_from_term(term: &SyGuSTerm) -> SExpr {
    match term {
        SyGuSTerm::Identifier(id) => sexpr_from_identifier(id),
        SyGuSTerm::Literal(lit) => SExpr::SpecConstant(lit.clone()),
        SyGuSTerm::Application(id, args) => {
            let mut items = Vec::new();
            items.push(sexpr_from_identifier(id));
            items.extend(args.iter().map(sexpr_from_term));
            sexpr_list(items)
        }
        SyGuSTerm::Annotated(term, attrs) => {
            let mut items = Vec::new();
            items.push(sexpr_symbol("!"));
            items.push(sexpr_from_term(term));
            items.extend(attrs.iter().map(sexpr_from_attribute));
            sexpr_list(items)
        }
        SyGuSTerm::Exists(vars, body) => {
            let vars_list = sexpr_list(vars.iter().map(sexpr_from_sorted_var).collect());
            sexpr_list(vec![sexpr_symbol("exists"), vars_list, sexpr_from_term(body)])
        }
        SyGuSTerm::Forall(vars, body) => {
            let vars_list = sexpr_list(vars.iter().map(sexpr_from_sorted_var).collect());
            sexpr_list(vec![sexpr_symbol("forall"), vars_list, sexpr_from_term(body)])
        }
        SyGuSTerm::Let(bindings, body) => {
            let binds_list = sexpr_list(bindings.iter().map(sexpr_from_var_binding).collect());
            sexpr_list(vec![sexpr_symbol("let"), binds_list, sexpr_from_term(body)])
        }
    }
}

fn atom_to_term(atom: &Atom) -> SyGuSTerm {
    if atom.args.is_empty() {
        SyGuSTerm::Identifier(Identifier::Symbol(atom.pred.clone()))
    } else {
        SyGuSTerm::Application(Identifier::Symbol(atom.pred.clone()), atom.args.clone())
    }
}

fn body_to_term(body_atoms: &[Atom], constraints: &[SyGuSTerm]) -> Option<SyGuSTerm> {
    let mut terms = Vec::new();
    for atom in body_atoms {
        terms.push(atom_to_term(atom));
    }
    for term in constraints {
        terms.push(term.clone());
    }
    match terms.len() {
        0 => None,
        1 => Some(terms.remove(0)),
        _ => Some(SyGuSTerm::Application(
            Identifier::Symbol("and".to_string()),
            terms,
        )),
    }
}

fn rule_to_sexpr(rule: &Rule) -> SExpr {
    let head = atom_to_term(&rule.head);
    match body_to_term(&rule.body_atoms, &rule.constraints) {
        None => sexpr_list(vec![sexpr_reserved("rule"), sexpr_from_term(&head)]),
        Some(body) => {
            let implication = SyGuSTerm::Application(
                Identifier::Symbol("=>".to_string()),
                vec![body, head],
            );
            sexpr_list(vec![sexpr_reserved("rule"), sexpr_from_term(&implication)])
        }
    }
}

fn query_to_sexpr(query: &Query) -> SExpr {
    let body = body_to_term(&query.body_atoms, &query.constraints)
        .unwrap_or_else(|| SyGuSTerm::Literal(Literal::Bool(true)));
    sexpr_list(vec![sexpr_reserved("query"), sexpr_from_term(&body)])
}

fn decl_to_sexpr(decl: &Decl) -> SExpr {
    match decl {
        Decl::SetLogic(logic) => sexpr_list(vec![sexpr_reserved("set-logic"), sexpr_symbol(logic)]),
        Decl::DeclareRel { name, args } => {
            let args_list = sexpr_list(args.iter().map(sexpr_from_sort).collect());
            sexpr_list(vec![
                sexpr_reserved("declare-rel"),
                sexpr_symbol(name),
                args_list,
            ])
        }
        Decl::DeclareVar { name, sort } => sexpr_list(vec![
            sexpr_reserved("declare-var"),
            sexpr_symbol(name),
            sexpr_from_sort(sort),
        ]),
        Decl::DeclareConst { name, sort } => {
            let args_list = sexpr_list(Vec::new());
            sexpr_list(vec![
                sexpr_reserved("declare-fun"),
                sexpr_symbol(name),
                args_list,
                sexpr_from_sort(sort),
            ])
        }
        Decl::DeclareDatatypes { sorts, decls } => {
            let sorts_list = sexpr_list(
                sorts
                    .iter()
                    .map(|s| {
                        sexpr_list(vec![
                            sexpr_symbol(&s.symbol),
                            SExpr::SpecConstant(Literal::Numeral(s.arity)),
                        ])
                    })
                    .collect(),
            );
            let decls_list = sexpr_list(
                decls
                    .iter()
                    .map(|dt| {
                        sexpr_list(
                            dt.iter()
                                .map(|cons| {
                                    let mut items = Vec::new();
                                    items.push(sexpr_symbol(&cons.name));
                                    for field in &cons.args {
                                        items.push(sexpr_list(vec![
                                            sexpr_symbol(&field.name),
                                            sexpr_from_sort(&field.sort),
                                        ]));
                                    }
                                    sexpr_list(items)
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            );
            sexpr_list(vec![
                sexpr_reserved("declare-datatypes"),
                sorts_list,
                decls_list,
            ])
        }
    }
}

pub fn to_smt(program: &Program) -> Vec<SExpr> {
    let mut out = Vec::new();
    for decl in &program.decls {
        out.push(decl_to_sexpr(decl));
    }
    for rule in &program.rules {
        out.push(rule_to_sexpr(rule));
    }
    for query in &program.queries {
        out.push(query_to_sexpr(query));
    }
    out
}
