use crate::backends::smt2::ast::HornProg;
use crate::parser::DelisProg;
use std::sync::OnceLock;
use std::time::Instant;
use sygus_parser::SyGuSCmd;

mod bisim;
mod compiler;
mod chc;
mod closure;
mod context;
pub(crate) mod emit;
mod expr;
mod finite;
mod eval_ctx;
pub(crate) mod optim;
mod semantic;
mod syntax;
mod types;

use context::CompileContext;
pub use syntax::{TermConstructor, TermSignature};
pub use types::AspectKey;

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub optimize: bool,
    pub bisim: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimize: true,
            bisim: true,
        }
    }
}

static COMPILE_OPTS: OnceLock<CompileOptions> = OnceLock::new();

pub fn init_options(opts: CompileOptions) {
    COMPILE_OPTS.set(opts).ok();
}

pub fn opts() -> &'static CompileOptions {
    COMPILE_OPTS.get().expect("compile options not initialized")
}

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    closure::failing_constraint_indices(cmds, solution)
}

impl DelisProg {
    pub fn bv_sz(&self) -> Result<usize, String> {
        let ctx = CompileContext::new(self)?;
        let width = ctx
            .semantic_program
            .domains
            .iter()
            .filter(|domain| domain.term_sort == ctx.sig.start_sort())
            .map(|domain| domain.width())
            .sum::<usize>();
        if width == 0 {
            return Err("SyGuS compilation requires at least one semantic state".to_string());
        }
        Ok(width)
    }

    pub fn to_sygus(&self) -> Result<Vec<SyGuSCmd>, String> {
        let semantic_start = Instant::now();
        let ctx = CompileContext::new(self)?;
        log::info!(
            "compile_semantic_program completed in {:?}",
            semantic_start.elapsed()
        );
        log::info!("using native SyGuS BV lowering");
        let native_start = Instant::now();
        let result = ctx.compile();
        log::info!(
            "native SyGuS BV lowering completed in {:?}",
            native_start.elapsed()
        );
        result
    }

    pub fn to_chc_ast(&self) -> Result<HornProg, String> {
        CompileContext::new(self)?.compile_chc()
    }

    pub fn to_sygus_bytes(&self) -> Result<Vec<u8>, String> {
        let cmds = self.to_sygus()?;
        let mut out = Vec::new();
        sygus_parser::writer::write_cmds(&mut out, &cmds).map_err(|e| e.to_string())?;
        use std::io::Write as _;
        writeln!(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests;
