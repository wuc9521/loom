use crate::backends::smt2::ast::HornProg;
use crate::parser::DelisProg;
use std::sync::OnceLock;
use std::time::Instant;
use sygus_parser::SyGuSCmd;

mod aspects;
mod bexpr;
mod bisim;
mod chc;
mod closure;
mod compiler;
mod context;
mod dual;
mod parent;
mod eval;
mod eval_ctx;
mod expr;
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

pub fn init_options(opts: CompileOptions) {
    COMPILE_OPTS.set(opts).ok();
}

pub fn opts() -> CompileOptions {
    COMPILE_OPTS
        .get()
        .cloned()
        .unwrap_or_else(CompileOptions::default)
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
        let ctx = CompileContext::new(self)?;
        log::info!("semantic program built in {:?}", semantic_start.elapsed());
        let encode_start = Instant::now();
        let cmds = ctx.compile()?;
        log::info!("encoded to SyGuS BV in {:?}", encode_start.elapsed());
        self.with_axiom_assume_cmds(cmds)
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

    /// Counterexample (refuter) encoding for a parent-bearing program. For a fixed
    /// skeleton, SAT means the skeleton violates some root demand; UNSAT means no
    /// violation certificate exists.
    pub fn parent_refuter_sygus(&self) -> Result<(String, String, String), String> {
        let ctx = CompileContext::new(self)?;
        let synth_name = ctx.synth_name.clone();
        let horn = ctx.compile_chc()?;
        let (sygus, sort) = parent::parent_sygus(&horn, &synth_name, parent::QueryKind::Refuter);
        let sygus = self.with_axiom_assumes(sygus);
        Ok((sygus, sort, synth_name))
    }

    fn with_axiom_assumes(&self, sygus: String) -> String {
        // Carry the program's `declare-axiom` operator laws to the solver as
        // `(assume ...)` commands: the synthesis backend uses them to recognize
        // semantically-equivalent candidates and dedup the search.
        let assumes: String = self
            .axioms()
            .iter()
            .map(|a| format!("(assume {a})\n"))
            .collect();
        if assumes.is_empty() {
            sygus
        } else {
            sygus.replace("(check-synth)\n", &format!("{assumes}(check-synth)\n"))
        }
    }

    fn with_axiom_assume_cmds(&self, mut cmds: Vec<SyGuSCmd>) -> Result<Vec<SyGuSCmd>, String> {
        let assumes = self.axioms().into_iter().map(SyGuSCmd::Assume);
        let insert_at = cmds
            .iter()
            .position(|cmd| matches!(cmd, SyGuSCmd::CheckSynth))
            .unwrap_or(cmds.len());
        cmds.splice(insert_at..insert_at, assumes);
        Ok(cmds)
    }

    pub fn parent_eval_bench(&self, max_size: usize) -> Result<(), String> {
        let ctx = CompileContext::new(self)?;
        let horn = ctx.compile_chc()?;
        parent::parent_eval_bench(&horn, max_size);
        Ok(())
    }
}

/// Decode a SyGuS solver's solution for the parent encoding into a plain delis
/// term (the synth-fun named `synth_name`, with annotation args stripped).
pub fn decode_parent_term(stdout: &str, synth_name: &str) -> Option<String> {
    parent::decode_term(stdout, synth_name)
}

#[cfg(test)]
mod tests;
