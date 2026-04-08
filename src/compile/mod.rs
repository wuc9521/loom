use crate::backends::smt2::ast::{
    HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm, PredicatePolarity,
};
use crate::backends::smt2::pred::{dualize_dnf, emit_dnf_rules, Pred, PredAtom};
use crate::backends::smt2::saturate::saturate;
use crate::parser::{DelisCmd, DelisProg};
use std::collections::HashMap;
use sygus_parser::{Sort, SortDecl, SyGuSCmd};

mod expr;
mod finite;
mod syntax;
mod types;

use expr::eval_constructor_case;
pub use finite::compile;
use syntax::{parse_semantic_cases, state_name, synth_name, term_sort, TermSignature};
pub use types::{AspectKey, BitMapping, ConstraintBit, Encoding};

#[derive(Clone, Debug)]
struct StatePredicates {
    state: String,
    positive: String,
    negative: String,
}

pub fn to_sygus(problem: &DelisProg) -> Result<Vec<SyGuSCmd>, String> {
    let summary = compile(problem)?;
    if summary.bit_mappings.is_empty() {
        return Err("SyGuS compilation requires at least one semantic aspect bit".to_string());
    }

    let bv_sort = Sort::bitvec(summary.bit_mappings.len());
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("SyGuS compilation requires declare-term-types".to_string());
    };
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    let grammar = sig.bv_grammar(declare_term_types, &bv_sort)?;
    let mut program = compile_chc(problem, &summary, DualRules::Omit)?;
    if summary.needs_saturation {
        program.rules = saturate(&program.rules);
    }
    crate::backends::sygus::bv::compile_one_way(&program, &synth_name(problem)?, grammar)
        .into_iter()
        .map(Ok)
        .collect()
}

pub fn to_chc_ast(problem: &DelisProg) -> Result<HornProg, String> {
    let summary = compile(problem)?;
    compile_chc(problem, &summary, DualRules::Emit)
}

#[derive(Clone, Copy)]
enum DualRules {
    Emit,
    Omit,
}

fn compile_chc(
    file: &DelisProg,
    summary: &Encoding,
    dual_rules: DualRules,
) -> Result<HornProg, String> {
    let declare_term_types = file.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("CHC compilation requires declare-term-types".to_string());
    };
    let cases = parse_semantic_cases(file)?;
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    let mut program = HornProg::default();
    program.decls.push(HornDecl::SetLogic("HORN".to_string()));
    program.decls.push(HornDecl::DeclareDatatypes(
        vec![SortDecl {
            symbol: "Term".to_string(),
            arity: 0,
        }],
        vec![sig.datatype_constructors()],
    ));
    program
        .decls
        .push(HornDecl::DeclareVar("r".to_string(), term_sort()));
    for var in sig.all_match_vars(&cases) {
        program.decls.push(HornDecl::DeclareVar(var, term_sort()));
    }
    let state_preds = summary
        .bit_mappings
        .iter()
        .map(|mapping| StatePredicates {
            state: state_name(&mapping.aspect),
            positive: format!("q{}", mapping.bit),
            negative: format!("nq{}", mapping.bit),
        })
        .collect::<Vec<_>>();
    let predicate_names = summary
        .bit_mappings
        .iter()
        .map(|mapping| {
            let preds = &state_preds[mapping.bit];
            (
                mapping.aspect.clone(),
                (preds.positive.clone(), preds.negative.clone()),
            )
        })
        .collect::<HashMap<_, _>>();
    for preds in &state_preds {
        rel_decl(&mut program, &preds.positive);
        program.register_predicate(
            preds.positive.clone(),
            preds.state.clone(),
            PredicatePolarity::Positive,
        );
        rel_decl(&mut program, &preds.negative);
        program.register_predicate(
            preds.negative.clone(),
            preds.state.clone(),
            PredicatePolarity::Negative,
        );
    }
    rel_decl(&mut program, "Goal");

    for cons in &sig.constructors {
        let Some(case) = cases.get(&cons.name) else {
            return Err(format!(
                "missing define-semantics match case for constructor {}",
                cons.name
            ));
        };
        if case.vars.len() != cons.arg_sorts.len() {
            return Err(format!(
                "match case for constructor {} binds {} variables, expected {}",
                cons.name,
                case.vars.len(),
                cons.arg_sorts.len()
            ));
        }
        let head_arg = HornTerm::app(
            &cons.name,
            case.vars.iter().map(HornTerm::var).collect::<Vec<_>>(),
        );
        let child_vars = case
            .vars
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        for mapping in &summary.bit_mappings {
            let preds = &state_preds[mapping.bit];
            let dnf = eval_constructor_case(
                file,
                &case.body,
                &mapping.aspect,
                &predicate_names,
                &summary.semantic_relations,
                &child_vars,
                &sig,
                cons,
                &head_arg,
                &case.self_var,
            )?;
            let head = PredAtom::new(
                Pred::new(preds.positive.clone(), preds.negative.clone()),
                head_arg.clone(),
            );
            emit_dnf_rules(&mut program.rules, head.clone(), dnf.clone());
            if matches!(dual_rules, DualRules::Emit) {
                if let Some((dual_head, dual_dnf)) = dualize_dnf(head, &dnf) {
                    emit_dnf_rules(&mut program.rules, dual_head, dual_dnf);
                }
            }
        }
    }

    let goal_body = summary
        .constraint_bits
        .iter()
        .map(|constraint| {
            let aspect = summary
                .bit_mappings
                .iter()
                .find(|mapping| mapping.bit == constraint.bit)
                .map(|mapping| &state_preds[mapping.bit])
                .ok_or_else(|| format!("constraint references unknown bit {}", constraint.bit))?;
            Ok(HornAtom {
                pred: if constraint.expected {
                    aspect.positive.clone()
                } else {
                    aspect.negative.clone()
                },
                arg: HornTerm::var("r"),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    program.rules.push(HornRule {
        body_atoms: goal_body,
        head: HornAtom {
            pred: "Goal".to_string(),
            arg: HornTerm::var("r"),
        },
    });
    program.queries.push(HornQuery {
        body_atoms: vec![HornAtom {
            pred: "Goal".to_string(),
            arg: HornTerm::var("r"),
        }],
    });
    Ok(program)
}

fn rel_decl(program: &mut HornProg, name: impl Into<String>) {
    program
        .decls
        .push(HornDecl::DeclareRel(name.into(), vec![term_sort()]));
}

#[cfg(test)]
mod tests;
