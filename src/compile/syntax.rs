use crate::{
    backends::smt2::ast::HornTerm,
    compile::{types::SemCase, AspectKey},
    parser::{ast::TermTypes, DelisCmd, DelisProg},
};
use std::collections::HashMap;
use sygus_parser::{
    DTConsDecl, GrammarDef, GroupedRuleList, Identifier, Sort, SortedVar, SyGuSCmd, SyGuSGTerm,
    SyGuSTerm,
};

pub fn synth_name(problem: &DelisProg) -> Result<String, String> {
    problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::As(SyGuSCmd::SynthFun(name, _, _, _)) => Some(name.clone()),
            _ => None,
        })
        .ok_or_else(|| "SyGuS compilation requires synth-fun".to_string())
}

pub fn term_sort() -> Sort {
    Sort::Simple(Identifier::Symbol("Term".to_string()))
}

pub fn state_name(aspect: &AspectKey) -> String {
    let mut parts = vec![aspect.relation.clone(), aspect.model.clone()];
    for (field, value) in &aspect.fields {
        parts.push(field.clone());
        parts.push(value.display());
    }
    parts.join(".")
}

#[derive(Debug, Clone)]
pub struct TermConstructor {
    pub name: String,
    pub result_sort: String,
    pub arg_sorts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TermSignature {
    pub constructors: Vec<TermConstructor>,
    start_sort: String,
}

impl TermSignature {
    pub fn from_declare_term_types(decl: &TermTypes) -> Result<Self, String> {
        let mut constructors = Vec::new();
        for (sort_decl, term_type_decl) in decl.sort_decls.iter().zip(&decl.term_type_decls) {
            for cons in &term_type_decl.constructors {
                constructors.push(TermConstructor {
                    name: cons.symbol.to_string(),
                    result_sort: sort_decl.symbol.clone(),
                    arg_sorts: cons.sorts.iter().map(|sort| sort.to_string()).collect(),
                });
            }
        }
        let start_sort = decl
            .sort_decls
            .first()
            .map(|decl| decl.symbol.clone())
            .ok_or_else(|| "declare-term-types is empty".to_string())?;
        Ok(Self {
            constructors,
            start_sort,
        })
    }

    pub fn datatype_constructors(&self) -> Vec<DTConsDecl> {
        self.constructors
            .iter()
            .map(|cons| DTConsDecl {
                name: cons.name.clone(),
                args: cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(idx, _sort)| SortedVar {
                        name: format!("{}_arg{idx}", &cons.name),
                        sort: term_sort(),
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn start_sort(&self) -> &str {
        &self.start_sort
    }

    pub fn grammar_symbol_for_sort(&self, sort_name: &str) -> String {
        self.grammar_symbol(sort_name)
    }

    pub fn all_match_vars(&self, cases: &HashMap<String, SemCase>) -> Vec<String> {
        let mut vars = std::collections::BTreeSet::new();
        for case in cases.values() {
            vars.extend(case.vars.iter().cloned());
        }
        for cons in &self.constructors {
            for idx in 0..cons.arg_sorts.len() {
                vars.insert(format!("__up{idx}"));
            }
        }
        vars.into_iter().collect()
    }

    pub fn parent_contexts(&self, child_sort: &str, child: HornTerm) -> Vec<HornTerm> {
        let mut out = Vec::new();
        for cons in &self.constructors {
            for (idx, arg_sort) in cons.arg_sorts.iter().enumerate() {
                if arg_sort != child_sort {
                    continue;
                }
                let args = cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(arg_idx, _)| {
                        if arg_idx == idx {
                            child.clone()
                        } else {
                            HornTerm::var(format!("__up{arg_idx}"))
                        }
                    })
                    .collect();
                out.push(HornTerm::app(&cons.name, args));
            }
        }
        out
    }

    pub fn bv_grammar(&self, decl: &TermTypes, bv_sort: &Sort) -> Result<GrammarDef, String> {
        let sorted_vars = decl
            .sort_decls
            .iter()
            .map(|sort_decl| SortedVar {
                name: self.grammar_symbol(&sort_decl.symbol),
                sort: bv_sort.clone(),
            })
            .collect::<Vec<_>>();
        let grouped_rule_lists = decl
            .sort_decls
            .iter()
            .zip(decl.term_type_decls.iter())
            .map(|(sort_decl, term_type_decl)| {
                let symbol = self.grammar_symbol(&sort_decl.symbol);
                let terms = term_type_decl
                    .constructors
                    .iter()
                    .map(|cons| {
                        let name = cons.symbol.to_string();
                        if cons.sorts.is_empty() {
                            self.gterm(&name, Vec::new())
                        } else {
                            self.gterm(
                                &name,
                                cons.sorts
                                    .iter()
                                    .map(|sort| self.grammar_symbol(&sort.to_string()))
                                    .collect(),
                            )
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(GroupedRuleList {
                    symbol,
                    sort: bv_sort.clone(),
                    terms,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(GrammarDef {
            sorted_vars,
            grouped_rule_lists,
        })
    }

    fn grammar_symbol(&self, sort_name: &str) -> String {
        if sort_name == self.start_sort {
            "Start".to_string()
        } else {
            sort_name.to_string()
        }
    }

    fn gterm(&self, name: &str, args: Vec<String>) -> Result<SyGuSGTerm, String> {
        let id = Identifier::from_str(name)
            .map_err(|e| format!("invalid grammar identifier {name}: {e}"))?;
        if args.is_empty() {
            return Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Identifier(id)));
        }
        Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(
            id,
            args.into_iter()
                .map(|arg| {
                    Identifier::from_str(&arg)
                        .map(SyGuSTerm::Identifier)
                        .map_err(|e| format!("invalid grammar argument {arg}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )))
    }
}

pub fn parse_semantic_cases(problem: &DelisProg) -> Result<HashMap<String, SemCase>, String> {
    let semantics = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::Semantics(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();
    if semantics.len() != 1 {
        return Err(format!(
            "CHC compilation currently supports exactly one define-semantics declaration, got {}",
            semantics.len()
        ));
    }
    let semantics = semantics
        .into_iter()
        .next()
        .ok_or_else(|| "SyGuS compilation requires define-semantics".to_string())?;
    let body = lexpr::from_str(&semantics.body)
        .map_err(|e| format!("failed to parse define-semantics body: {e}"))?;
    let outer = body
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must be a list: {body}"))?;
    let match_expr = match outer.as_slice() {
        [single] => single,
        _ => &body,
    };
    let items = match_expr
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must contain a match expression: {body}"))?;
    if items.first().and_then(|v| v.as_symbol()) != Some("match") {
        return Err(format!(
            "define-semantics body must start with match: {body}"
        ));
    }
    let self_var = items
        .get(1)
        .and_then(|v| v.as_symbol())
        .ok_or_else(|| format!("match must bind a scrutinee variable: {body}"))?
        .to_string();
    let mut cases = HashMap::new();
    for case in items.iter().skip(2) {
        let case_items = case
            .to_ref_vec()
            .ok_or_else(|| format!("match case must be a list: {case}"))?;
        if case_items.len() != 2 {
            return Err(format!("match case must have pattern and body: {case}"));
        }
        let pattern = case_items[0]
            .to_ref_vec()
            .ok_or_else(|| format!("match pattern must be a list: {}", case_items[0]))?;
        let cons = pattern
            .first()
            .and_then(|v| v.as_symbol())
            .ok_or_else(|| {
                format!(
                    "match pattern constructor must be a symbol: {}",
                    case_items[0]
                )
            })?
            .to_string();
        let vars = pattern
            .iter()
            .skip(1)
            .map(|v| {
                v.as_symbol()
                    .map(str::to_string)
                    .ok_or_else(|| format!("match pattern variable must be a symbol: {v}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        cases.insert(
            cons.clone(),
            SemCase {
                self_var: self_var.clone(),
                vars,
                body: case_items[1].clone(),
            },
        );
    }
    Ok(cases)
}
