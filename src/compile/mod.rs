use crate::backends::smt2::ast::HornProg;
use crate::parser::DelisProg;
use std::time::Instant;
use sygus_parser::SyGuSCmd;

mod bv;
mod chc;
mod closure;
mod context;
pub(crate) mod emit;
mod expr;
mod finite;
mod native;
mod semantic;
mod syntax;
mod types;

use context::CompileContext;
pub use syntax::{TermConstructor, TermSignature};
pub use types::AspectKey;

pub fn solve_by_closure(problem: &DelisProg, optimize: bool) -> Result<Option<String>, String> {
    closure::solve(problem, optimize)
}

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    closure::failing_constraint_indices(cmds, solution)
}

pub fn bv_sz(problem: &DelisProg) -> Result<usize, String> {
    let ctx = CompileContext::new(problem)?;
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
    Ok(width)
}

pub fn to_sygus(problem: &DelisProg, optimize: bool) -> Result<Vec<SyGuSCmd>, String> {
    let semantic_start = Instant::now();
    let ctx = CompileContext::new(problem)?;
    log::info!(
        "compile_semantic_program completed in {:?}",
        semantic_start.elapsed()
    );
    log::info!("using native SyGuS BV lowering");
    let native_start = Instant::now();
    let result = bv::compile(&ctx, optimize);
    log::info!(
        "native SyGuS BV lowering completed in {:?}",
        native_start.elapsed()
    );
    result
}

pub fn to_chc_ast(problem: &DelisProg) -> Result<HornProg, String> {
    let ctx = CompileContext::new(problem)?;
    chc::compile_with_context(problem, ctx.semantic_program, ctx.sig, ctx.semantic_cases)
}

/// Render the SyGuS-IF program as bytes.
pub fn to_sygus_bytes(problem: &DelisProg, optimize: bool) -> Result<Vec<u8>, String> {
    let cmds = to_sygus(problem, optimize)?;
    let mut out = Vec::new();
    sygus_parser::writer::write_cmds(&mut out, &cmds).map_err(|e| e.to_string())?;
    use std::io::Write as _;
    writeln!(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests;
