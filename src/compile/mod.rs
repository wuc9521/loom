use crate::backends::smt2::ast::HornProg;
use crate::backends::smt2::twa::saturate_program;
use crate::parser::DelisProg;
use lexpr::Value;
use std::time::Instant;
use sygus_parser::{Sort, SyGuSCmd};

mod bv;
mod chc;
mod closure;
mod context;
mod expr;
mod finite;
mod native;
mod semantic;
mod syntax;
mod types;

pub use finite::compile;
pub use syntax::{TermConstructor, TermSignature};
pub use types::{AspectKey, BitMapping, ConstraintBit, Encoding};
use context::CompileContext;

pub fn solve_by_closure(problem: &DelisProg, optimize: bool) -> Result<Option<String>, String> {
    closure::solve(problem, optimize)
}

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    closure::failing_constraint_indices(cmds, solution)
}

pub fn bv_sz(problem: &DelisProg) -> Result<usize, String> {
    let ctx = CompileContext::new(problem)?;
    if supports_native_sygus(&ctx.semantic_program) {
        let width = ctx
            .semantic_program
            .domains
            .iter()
            .filter(|domain| domain.term_sort == ctx.sig.start_sort())
            .map(|domain| domain.bits.len())
            .sum::<usize>();
        if width == 0 {
            return Err("SyGuS compilation requires at least one semantic state".to_string());
        }
        return Ok(width);
    }
    let mut program = chc::compile_with_context(
        ctx.problem,
        ctx.semantic_program.clone(),
        ctx.sig.clone(),
        ctx.semantic_cases.clone(),
    )?;
    if ctx.semantic_program.needs_saturation {
        program = saturate_program(&program, &ctx.sig)?;
    }
    let widths = complete_sort_widths(
        &ctx.sig,
        &ctx.semantic_program,
        &ctx.semantic_cases,
        crate::backends::sygus::bv::sort_widths(&program),
    )?;
    let width = widths.get(ctx.sig.start_sort()).copied().unwrap_or(0);
    if width == 0 {
        return Err("SyGuS compilation requires at least one semantic state".to_string());
    }
    Ok(width)
}

pub fn to_sygus(problem: &DelisProg, optimize: bool) -> Result<Vec<SyGuSCmd>, String> {
    let semantic_start = Instant::now();
    let ctx = CompileContext::new(problem)?;
    log::info!(
        "compile_semantic_program completed in {:?}",
        semantic_start.elapsed()
    );
    if supports_native_sygus(&ctx.semantic_program) {
        log::info!("using native SyGuS BV lowering");
        let native_start = Instant::now();
        let result = bv::compile(&ctx, optimize);
        log::info!(
            "native SyGuS BV lowering completed in {:?}",
            native_start.elapsed()
        );
        return result;
    }
    log::info!("using CHC-to-BV SyGuS lowering");
    let chc_start = Instant::now();
    let mut program = chc::compile_with_context(
        ctx.problem,
        ctx.semantic_program.clone(),
        ctx.sig.clone(),
        ctx.semantic_cases.clone(),
    )?;
    if ctx.semantic_program.needs_saturation {
        program = saturate_program(&program, &ctx.sig)?;
    }
    log::info!("CHC lowering completed in {:?}", chc_start.elapsed());
    let layouts = complete_layouts(
        &ctx.sig,
        &ctx.semantic_program,
        &ctx.semantic_cases,
        crate::backends::sygus::bv::layouts_by_sort(&program),
    )?;
    let sort_alias_cmds = layouts
        .iter()
        .map(|(sort, layout)| {
            SyGuSCmd::DefineSort(
                sort.clone(),
                Vec::new(),
                Sort::bitvec(layout.states.len().max(1)),
            )
        })
        .collect::<Vec<_>>();
    let sort_aliases = layouts
        .keys()
        .map(|sort| (sort.clone(), Sort::from_name(sort)))
        .collect::<std::collections::HashMap<_, _>>();
    let grammar = ctx.sig.bv_grammar_with_sorts(ctx.term_types, &sort_aliases)?;
    let bv_start = Instant::now();
    let mut result = crate::backends::sygus::bv::compile_one_way(
        &program,
        &ctx.synth_name,
        grammar,
        &ctx.sig,
        ctx.sig.start_sort(),
        Some(layouts),
        optimize,
    );
    result.splice(1..1, sort_alias_cmds);
    log::info!("CHC BV emission completed in {:?}", bv_start.elapsed());
    Ok(result)
}

fn complete_layouts(
    sig: &TermSignature,
    program: &types::SemanticProgram,
    semantic_cases: &context::SemanticCases,
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
        let cases = semantic_cases
            .get(&spec.relation)
            .ok_or_else(|| format!("missing parsed cases for {}", spec.relation))?;
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
    semantic_cases: &context::SemanticCases,
    widths: std::collections::HashMap<String, usize>,
) -> Result<std::collections::HashMap<String, usize>, String> {
    complete_layouts(
        sig,
        program,
        semantic_cases,
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
    let ctx = CompileContext::new(problem)?;
    Ok(ctx
        .semantic_program
        .domains
        .iter()
        .any(|domain| !matches!(domain.kind, types::SemanticKind::Bool)))
}

pub fn to_chc_ast(problem: &DelisProg) -> Result<HornProg, String> {
    let ctx = CompileContext::new(problem)?;
    chc::compile_with_context(problem, ctx.semantic_program, ctx.sig, ctx.semantic_cases)
}

#[cfg(test)]
mod tests;
