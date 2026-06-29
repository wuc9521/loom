use rustc_hash::FxHashMap as HashMap;

use sygus_parser::SortedVar;

use crate::parser::{ast::TermTypes, DelisCmd, DelisProg};

use super::{
    eval_ctx::SemanticContext,
    semantics::{self, Case},
    syntax::TermSignature,
    types::SemanticProgram,
};

pub type SemanticCases = HashMap<String, HashMap<String, Case>>;

pub struct CompileContext<'a> {
    pub problem: &'a DelisProg,
    pub term_types: &'a TermTypes,
    pub sig: TermSignature,
    pub semantic_program: SemanticProgram,
    pub semantic_cases: SemanticCases,
    pub synth_name: String,
    pub synth_args: Vec<SortedVar>,
}

impl<'a> CompileContext<'a> {
    pub fn new(problem: &'a DelisProg) -> Result<Self, String> {
        let semantic_program = problem.semantic_program()?;
        Self::with_program(problem, semantic_program)
    }

    /// Like `new`, but reuses a pre-built `SemanticProgram` instead of
    /// re-running `compile_semantic_program` (which includes bisim).
    /// Lets the two parallel backends share one compile.
    pub fn with_program(
        problem: &'a DelisProg,
        semantic_program: SemanticProgram,
    ) -> Result<Self, String> {
        let term_types = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::TermTypes(cmd) => Some(cmd),
                _ => None,
            })
            .ok_or_else(|| "SyGuS compilation requires declare-term-types".to_string())?;
        let (synth_name, synth_args, synth_sort) = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::As(sygus_parser::SyGuSCmd::SynthFun(name, args, sort, _)) => {
                    Some((name.clone(), args.clone(), sort.clone()))
                }
                _ => None,
            })
            .ok_or_else(|| "SyGuS compilation requires synth-fun".to_string())?;
        let mut sig = TermSignature::from_declare_term_types(term_types)?;
        // The synth-fun's declared return sort selects the grammar root: a
        // Bool-valued formula sort for separator synthesis, or a value sort
        // for term synthesis. Falls back to the first-declared sort.
        sig.set_start_sort(&synth_sort.to_string())?;
        // Merge cases across every spec for the same relation. Multiple
        // (define-semantics ((R (n S) ...))) blocks at different sorts S
        // share one cases map keyed by constructor name; the spec's
        // term_sort is recovered per constructor from sig.
        let mut semantic_cases: SemanticCases = HashMap::default();
        for spec in &semantic_program.semantics {
            let cases = semantics::parse_cases(&spec.body, &spec.aspect_var)?;
            let entry = semantic_cases.entry(spec.relation.clone()).or_default();
            for (cons, case) in cases {
                entry.insert(cons, case);
            }
        }
        Ok(Self {
            problem,
            term_types,
            sig,
            semantic_program,
            semantic_cases,
            synth_name,
            synth_args,
        })
    }

    pub fn semantic_context(&self) -> Result<SemanticContext<'a>, String> {
        SemanticContext::with_cases(self.problem, &self.semantic_program, &self.semantic_cases)
    }

    /// True if no semantic spec navigates to a node's parent. Parent-free
    /// programs compile via the direct bit encoder; parent-bearing ones use the
    /// crossing-relation certificate encoder. See [`CompileContext::compile`].
    pub fn is_parent_free(&self) -> bool {
        !self
            .semantic_program
            .semantics
            .iter()
            .any(|spec| super::aspects::contains_parent_navigation(&spec.body))
    }
}
