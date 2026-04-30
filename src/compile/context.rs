use std::collections::HashMap;

use sygus_parser::SortedVar;

use crate::parser::{ast::TermTypes, DelisCmd, DelisProg};

use super::{
    finite,
    native::SemanticContext,
    semantic::{self, Case},
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
        let term_types = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::TermTypes(cmd) => Some(cmd),
                _ => None,
            })
            .ok_or_else(|| "SyGuS compilation requires declare-term-types".to_string())?;
        let (synth_name, synth_args, _synth_sort) = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::As(sygus_parser::SyGuSCmd::SynthFun(name, args, sort, _)) => {
                    Some((name.clone(), args.clone(), sort.clone()))
                }
                _ => None,
            })
            .ok_or_else(|| "SyGuS compilation requires synth-fun".to_string())?;
        let sig = TermSignature::from_declare_term_types(term_types)?;
        let semantic_program = finite::compile_semantic_program(problem)?;
        let semantic_cases = semantic_program
            .semantics
            .iter()
            .map(|spec| {
                semantic::parse_cases(&spec.body).map(|cases| (spec.relation.clone(), cases))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
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
}
