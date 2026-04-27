use crate::backends::smt2::ast::HornProg;
use crate::backends::smt2::twa::saturate_program;
use crate::parser::{DelisCmd, DelisProg};
use lexpr::Value;
use std::time::Instant;
use sygus_parser::{Sort, SyGuSCmd};

mod bv;
mod chc;
mod closure;
mod expr;
mod finite;
mod native;
mod semantic;
mod syntax;
mod types;

pub use finite::compile;
use syntax::synth_name;
pub use syntax::{TermConstructor, TermSignature};
pub use types::{AspectKey, BitMapping, ConstraintBit, Encoding};

pub fn solve_by_closure(problem: &DelisProg, optimize: bool) -> Result<Option<String>, String> {
    closure::solve(problem, optimize)
}

pub fn bv_sz(problem: &DelisProg) -> Result<usize, String> {
    let semantic_program = finite::compile_semantic_program(problem)?;
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("SyGuS compilation requires declare-term-types".to_string());
    };
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    if supports_native_sygus(&semantic_program) {
        let width = semantic_program
            .domains
            .iter()
            .filter(|domain| domain.term_sort == sig.start_sort())
            .map(|domain| domain.bits.len())
            .sum::<usize>();
        if width == 0 {
            return Err("SyGuS compilation requires at least one semantic state".to_string());
        }
        return Ok(width);
    }
    let mut program = to_chc_ast(problem)?;
    if semantic_program.needs_saturation {
        program = saturate_program(&program, &sig)?;
    }
    let widths = complete_sort_widths(
        &sig,
        &semantic_program,
        crate::backends::sygus::bv::sort_widths(&program),
    )?;
    let width = widths.get(sig.start_sort()).copied().unwrap_or(0);
    if width == 0 {
        return Err("SyGuS compilation requires at least one semantic state".to_string());
    }
    Ok(width)
}

pub fn to_sygus(problem: &DelisProg, optimize: bool) -> Result<Vec<SyGuSCmd>, String> {
    let semantic_start = Instant::now();
    let semantic_program = finite::compile_semantic_program(problem)?;
    log::info!(
        "compile_semantic_program completed in {:?}",
        semantic_start.elapsed()
    );
    if supports_native_sygus(&semantic_program) {
        log::info!("using native SyGuS BV lowering");
        let native_start = Instant::now();
        let result = bv::compile(problem, semantic_program, optimize);
        log::info!("native SyGuS BV lowering completed in {:?}", native_start.elapsed());
        return result;
    }
    log::info!("using CHC-to-BV SyGuS lowering");
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("SyGuS compilation requires declare-term-types".to_string());
    };
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    let chc_start = Instant::now();
    let mut program = to_chc_ast(problem)?;
    if semantic_program.needs_saturation {
        program = saturate_program(&program, &sig)?;
    }
    log::info!("CHC lowering completed in {:?}", chc_start.elapsed());
    let layouts = complete_layouts(
        &sig,
        &semantic_program,
        crate::backends::sygus::bv::layouts_by_sort(&program),
    )?;
    let sort_widths = layouts
        .iter()
        .map(|(sort, layout)| (sort.clone(), Sort::bitvec(layout.states.len().max(1))))
        .collect::<std::collections::HashMap<_, _>>();
    let grammar = sig.bv_grammar_with_sorts(declare_term_types, &sort_widths)?;
    let bv_start = Instant::now();
    let result = crate::backends::sygus::bv::compile_one_way(
        &program,
        &synth_name(problem)?,
        grammar,
        &sig,
        sig.start_sort(),
        Some(layouts),
        optimize,
    )
    .into_iter()
    .map(Ok)
    .collect();
    log::info!("CHC BV emission completed in {:?}", bv_start.elapsed());
    result
}

fn complete_layouts(
    sig: &TermSignature,
    program: &types::SemanticProgram,
    mut layouts: std::collections::HashMap<String, crate::backends::sygus::bv::SortLayout>,
) -> Result<std::collections::HashMap<String, crate::backends::sygus::bv::SortLayout>, String> {
    let fallback_template = if layouts.len() == 1 {
        layouts.values().next().cloned()
    } else {
        None
    };
    let cons_sorts = sig
        .constructors
        .iter()
        .map(|cons| (cons.name.clone(), cons.result_sort.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    for spec in &program.semantics {
        let Some(template) = layouts
            .get(&spec.term_sort)
            .cloned()
            .or_else(|| fallback_template.clone())
        else {
            continue;
        };
        layouts
            .entry(spec.term_sort.clone())
            .or_insert_with(|| template.clone());
        let cases = semantic::parse_cases(&spec.body)?;
        for cons in cases.keys() {
            if let Some(sort) = cons_sorts.get(cons) {
                layouts
                    .entry(sort.clone())
                    .or_insert_with(|| template.clone());
            }
        }
    }
    Ok(layouts)
}

fn complete_sort_widths(
    sig: &TermSignature,
    program: &types::SemanticProgram,
    widths: std::collections::HashMap<String, usize>,
) -> Result<std::collections::HashMap<String, usize>, String> {
    complete_layouts(
        sig,
        program,
        widths
            .into_iter()
            .map(|(sort, width)| {
                let states = (0..width)
                    .map(|idx| format!("state#{idx}"))
                    .collect::<Vec<_>>();
                let index = states
                    .iter()
                    .enumerate()
                    .map(|(idx, state)| (state.clone(), idx))
                    .collect::<std::collections::HashMap<_, _>>();
                (
                    sort,
                    crate::backends::sygus::bv::SortLayout { states, index },
                )
            })
            .collect(),
    )
    .map(|layouts| {
        layouts
            .into_iter()
            .map(|(sort, layout)| (sort, layout.states.len()))
            .collect()
    })
}

fn supports_native_sygus(program: &types::SemanticProgram) -> bool {
    if program.needs_saturation {
        return false;
    }
    !program.semantics.iter().any(|spec| {
        contains_unsupported_native_feature(&spec.body)
            || contains_current_term_recursion(&spec.relation, &spec.body)
    })
}

fn contains_unsupported_native_feature(expr: &Value) -> bool {
    if let Some(items) = expr.to_ref_vec() {
        if let Some(op) = items.first().and_then(|item| item.as_symbol()) {
            if op == "parent" {
                return true;
            }
        }
        return items
            .iter()
            .any(|item| contains_unsupported_native_feature(item));
    }
    false
}

fn contains_current_term_recursion(relation: &str, body: &Value) -> bool {
    let Ok(cases) = semantic::parse_cases(body) else {
        return true;
    };
    cases
        .values()
        .any(|case| contains_self_call(relation, &case.self_var, &case.body))
}

fn contains_self_call(relation: &str, self_var: &str, expr: &Value) -> bool {
    let Some(items) = expr.to_ref_vec() else {
        return false;
    };
    if items.first().and_then(|item| item.as_symbol()) == Some(relation)
        && items.get(1).and_then(|item| item.as_symbol()) == Some(self_var)
    {
        return true;
    }
    items
        .iter()
        .any(|item| contains_self_call(relation, self_var, item))
}

pub fn uses_semantic_domains(problem: &DelisProg) -> Result<bool, String> {
    let semantic_program = finite::compile_semantic_program(problem)?;
    Ok(semantic_program
        .domains
        .iter()
        .any(|domain| !matches!(domain.kind, types::SemanticKind::Bool)))
}

pub fn to_chc_ast(problem: &DelisProg) -> Result<HornProg, String> {
    let semantic_program = finite::compile_semantic_program(problem)?;
    chc::compile(problem, semantic_program)
}

#[cfg(test)]
mod tests;
