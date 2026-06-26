use crate::backends::smt2::ast::HornProg;
use crate::parser::DelisProg;
use std::sync::OnceLock;
use std::time::Instant;
use sygus_parser::{SyGuSCmd, SyGuSFile};

mod aspects;
mod bexpr;
mod bisim;
mod chc;
mod closure;
mod compiler;
mod context;
mod crossing;
mod dual;
pub mod emit;
mod eval;
mod eval_ctx;
mod expr;
mod layout;
pub mod optim;
mod saturate;
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
        let program = self.semantic_program()?;
        log::info!("semantic program built in {:?}", semantic_start.elapsed());
        self.to_sygus_with(program)
    }

    /// Like `to_sygus`, but reuses an already-built `SemanticProgram` so
    /// bisim isn't re-run. Use from `main::solve` to share one compile
    /// between the parallel closure and sygus backends.
    pub fn to_sygus_with(&self, program: types::SemanticProgram) -> Result<Vec<SyGuSCmd>, String> {
        let parent_free = !program
            .semantics
            .iter()
            .any(|spec| aspects::contains_parent_navigation(&spec.body));
        let use_direct = opts().optimize && parent_free;
        let ctx = CompileContext::with_program(self, program)?;
        let encode_start = Instant::now();
        let result = if use_direct {
            self.with_axiom_assume_cmds(ctx.compile()?)
        } else if !parent_free && std::env::var("DELIS_LEGACY").is_err() {
            self.faithful_sygus_cmds_from_ctx(ctx)
        } else {
            self.with_axiom_assume_cmds(emit::compile_via_horn(ctx)?)
        };
        log::info!("encoded to SyGuS BV in {:?}", encode_start.elapsed());
        result
    }

    pub fn to_chc_ast(&self) -> Result<HornProg, String> {
        CompileContext::new(self)?.compile_chc()
    }

    pub fn to_chc_ast_saturated(&self) -> Result<HornProg, String> {
        let mut prog = self.to_chc_ast()?;
        saturate::saturate(&mut prog);
        Ok(prog)
    }

    pub fn to_sygus_bytes(&self) -> Result<Vec<u8>, String> {
        let cmds = self.to_sygus()?;
        let mut out = Vec::new();
        sygus_parser::writer::write_cmds(&mut out, &cmds).map_err(|e| e.to_string())?;
        use std::io::Write as _;
        writeln!(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Sound & complete crossing encoding for a parent-bearing program. Returns
    /// `(sygus-text, synth-target sort, synth-fun name)`; solve the text with an
    /// SMT-backed SyGuS solver (it contains `Constant` holes) and decode with
    /// [`decode_faithful_term`].
    pub fn faithful_sygus(&self) -> Result<(String, String, String), String> {
        let ctx = CompileContext::new(self)?;
        let synth_name = ctx.synth_name.clone();
        let horn = ctx.compile_chc()?;
        let (sygus, sort) = crossing::faithful_sygus(&horn, &synth_name);
        let sygus = self.with_axiom_assumes(sygus);
        Ok((sygus, sort, synth_name))
    }

    /// Counterexample crossing encoding for a parent-bearing program. For a fixed
    /// skeleton, SAT means the skeleton violates some root demand; UNSAT means no
    /// violation certificate exists.
    pub fn faithful_refuter_sygus(&self) -> Result<(String, String, String), String> {
        let ctx = CompileContext::new(self)?;
        let synth_name = ctx.synth_name.clone();
        let horn = ctx.compile_chc()?;
        let (sygus, sort) = crossing::faithful_refuter_sygus(&horn, &synth_name);
        let sygus = self.with_axiom_assumes(sygus);
        Ok((sygus, sort, synth_name))
    }

    fn faithful_sygus_cmds_from_ctx(
        &self,
        ctx: CompileContext<'_>,
    ) -> Result<Vec<SyGuSCmd>, String> {
        let synth_name = ctx.synth_name.clone();
        let horn = ctx.compile_chc()?;
        let (sygus, _) = crossing::faithful_sygus(&horn, &synth_name);
        let sygus = self.with_axiom_assumes(sygus);
        SyGuSFile::from_str(&sygus)
            .map(|file| file.cmds)
            .map_err(|e| format!("crossing SyGuS emission produced unparsable output: {e}"))
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

    pub fn crossing_eval_bench(&self, max_size: usize) -> Result<(), String> {
        let ctx = CompileContext::new(self)?;
        let horn = ctx.compile_chc()?;
        crossing::crossing_eval_bench(&horn, max_size);
        Ok(())
    }
}

/// Decode a SyGuS solver's solution for the crossing encoding into a plain delis
/// term (the synth-fun named `synth_name`, with annotation args stripped).
pub fn decode_faithful_term(stdout: &str, synth_name: &str) -> Option<String> {
    crossing::decode_term(stdout, synth_name)
}

#[cfg(test)]
mod tests;
