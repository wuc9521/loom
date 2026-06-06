use crate::backends::smt2::ast::HornProg;
use crate::parser::DelisProg;
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::Instant;
use sygus_parser::SyGuSCmd;

mod bisim;
mod compiler;
mod chc;
mod closure;
mod context;
pub mod emit;
mod eval;
mod expr;
mod aspects;
mod eval_ctx;
pub mod optim;
mod semantics;
mod syntax;
pub mod types;
mod verifier;

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

thread_local! {
    static OPTS_OVERRIDE: Cell<Option<CompileOptions>> = const { Cell::new(None) };
}

pub fn init_options(opts: CompileOptions) {
    COMPILE_OPTS.set(opts).ok();
}

pub fn opts() -> CompileOptions {
    if let Some(o) = OPTS_OVERRIDE.with(|c| {
        let v = c.take();
        c.set(v.clone());
        v
    }) {
        return o;
    }
    COMPILE_OPTS
        .get()
        .cloned()
        .unwrap_or_else(CompileOptions::default)
}

/// Run `f` with `opts` as the effective `CompileOptions` for this thread only.
/// Restores the previous override on return.
pub fn with_options<F, R>(opts: CompileOptions, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = OPTS_OVERRIDE.with(|c| c.replace(Some(opts)));
    let result = f();
    OPTS_OVERRIDE.with(|c| c.set(previous));
    result
}

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    let verifier = verifier::Verifier::new(cmds)?;
    verifier.failing_constraint_indices(solution)
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
        let program = self.semantic_program()?;
        log::info!(
            "semantic program built in {:?}",
            semantic_start.elapsed()
        );
        self.to_sygus_with(program)
    }

    /// Like `to_sygus`, but reuses an already-built `SemanticProgram` so
    /// bisim isn't re-run. Use from `main::solve` to share one compile
    /// between the parallel closure and sygus backends.
    pub fn to_sygus_with(&self, program: types::SemanticProgram) -> Result<Vec<SyGuSCmd>, String> {
        let ctx = CompileContext::with_program(self, program)?;
        let encode_start = Instant::now();
        let result = ctx.compile();
        log::info!("encoded to SyGuS BV in {:?}", encode_start.elapsed());
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
