use crate::sorts::SortEnv;
use crate::fxmap::HashMap;
use sygus_parser::{Literal, SortedVar, SyGuSCmd, SyGuSFile, SyGuSTerm};

use super::eval::{
    is_closure_constraint, term_mentions_symbol, Bits, DefineFun, EvalContext, EvalEnv, EvalValue,
};

pub struct Verifier {
    synth_name: String,
    synth_args: Vec<String>,
    synth_width: usize,
    funcs: HashMap<String, DefineFun>,
    constraints: Vec<SyGuSTerm>,
}

impl Verifier {
    pub fn new(cmds: &[SyGuSCmd]) -> Result<Self, String> {
        let sort_env = SortEnv::from_cmds(cmds);
        let mut funcs = HashMap::default();
        let mut synth = None;
        let mut constraints = Vec::new();
        for cmd in cmds {
            match cmd {
                SyGuSCmd::DefineFun(name, args, sort, body) => {
                    let width = sort_env.bv_width(sort)?;
                    funcs.insert(
                        name.clone(),
                        DefineFun {
                            args: args.iter().map(|arg| arg.name.clone()).collect(),
                            body: body.clone(),
                            width,
                        },
                    );
                }
                SyGuSCmd::SynthFun(name, args, sort, grammar) => {
                    let width = sort_env.bv_width(sort)?;
                    if grammar.as_ref().is_some_and(|g| is_constant_grammar(g)) {
                        funcs.insert(
                            name.clone(),
                            DefineFun {
                                args: Vec::new(),
                                body: SyGuSTerm::Literal(Literal::BinConst("0".repeat(width))),
                                width,
                            },
                        );
                    } else if synth.is_none() {
                        synth = Some((
                            name.clone(),
                            args.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                            width,
                        ));
                    }
                }
                SyGuSCmd::Constraint(term) => {
                    if is_closure_constraint(term, &synth.as_ref().map(|(n, _, _)| n.clone())) {
                        continue;
                    }
                    constraints.push(term.clone());
                }
                _ => {}
            }
        }
        let (synth_name, synth_args, synth_width) =
            synth.ok_or_else(|| "missing synth-fun command".to_string())?;
        Ok(Self {
            synth_name,
            synth_args,
            synth_width,
            funcs,
            constraints,
        })
    }

    pub fn failing_constraint_indices(&self, solution: &str) -> Result<Vec<usize>, String> {
        let define_funs = parse_solution_define_funs(solution)?;
        let synth_def = define_funs
            .iter()
            .find(|(name, _, _)| name == &self.synth_name)
            .or_else(|| define_funs.first())
            .ok_or_else(|| "solver response did not contain a define-fun".to_string())?;
        let (_, _, body) = synth_def.clone();
        let has_args = !self.synth_args.is_empty();

        let mut funcs = self.funcs.clone();
        if has_args {
            funcs.insert(
                self.synth_name.clone(),
                DefineFun {
                    args: self.synth_args.clone(),
                    body: body.clone(),
                    width: self.synth_width,
                },
            );
        }

        let mut base = HashMap::default();
        let evaluator = EvalContext { funcs: &funcs };
        let mut pending = define_funs
            .iter()
            .filter(|(name, args, _)| args.is_empty() && name != &self.synth_name)
            .cloned()
            .collect::<Vec<_>>();
        let mut progress = true;
        while progress && !pending.is_empty() {
            progress = false;
            let mut next_pending = Vec::new();
            for (name, _args, def_body) in pending {
                let mut local_env = EvalEnv::with_base(base.clone());
                match evaluator.eval(&def_body, &mut local_env) {
                    Ok(value) => {
                        base.insert(name, value);
                        progress = true;
                    }
                    Err(_) => next_pending.push((name, Vec::new(), def_body)),
                }
            }
            pending = next_pending;
        }

        if !has_args {
            let requires_self_fixpoint = term_mentions_symbol(&body, &self.synth_name)
                || pending
                    .iter()
                    .any(|(_, _, def_body)| term_mentions_symbol(def_body, &self.synth_name))
                || funcs.iter().any(|(name, func)| {
                    name != &self.synth_name && term_mentions_symbol(&func.body, &self.synth_name)
                });
            let bits = if requires_self_fixpoint {
                let mut current = Bits::zero(self.synth_width);
                let mut converged = None;
                for _ in 0..=self.synth_width {
                    let mut iter_base = base.clone();
                    iter_base.insert(
                        self.synth_name.clone(),
                        EvalValue::Bv(current.clone(), self.synth_width),
                    );
                    let mut remaining = pending.clone();
                    let mut progress = true;
                    while progress && !remaining.is_empty() {
                        progress = false;
                        let mut next_remaining = Vec::new();
                        for (name, _args, def_body) in remaining {
                            let mut local_env = EvalEnv::with_base(iter_base.clone());
                            match evaluator.eval(&def_body, &mut local_env) {
                                Ok(value) => {
                                    iter_base.insert(name, value);
                                    progress = true;
                                }
                                Err(_) => next_remaining.push((name, Vec::new(), def_body)),
                            }
                        }
                        remaining = next_remaining;
                    }
                    let next = evaluator.eval_bv(
                        &body,
                        &mut EvalEnv::with_base(iter_base),
                        self.synth_width,
                    )?;
                    if next == current {
                        converged = Some(next);
                        break;
                    }
                    current = next;
                }
                converged.ok_or_else(|| {
                    "verifier: self-closure candidate did not converge".to_string()
                })?
            } else {
                evaluator.eval_bv(
                    &body,
                    &mut EvalEnv::with_base(base.clone()),
                    self.synth_width,
                )?
            };
            log::debug!(
                "verifier: synth_width={} bits={}",
                self.synth_width,
                bits.0.len()
            );
            base.insert(
                self.synth_name.clone(),
                EvalValue::Bv(bits, self.synth_width),
            );
        }

        let mut env = EvalEnv::with_base(base);
        let mut failing = Vec::new();
        for (idx, constraint) in self.constraints.iter().enumerate() {
            if !evaluator.eval_bool(constraint, &mut env)? {
                failing.push(idx);
            }
        }
        Ok(failing)
    }
}

fn is_constant_grammar(grammar: &sygus_parser::GrammarDef) -> bool {
    grammar.grouped_rule_lists.len() == 1
        && grammar.grouped_rule_lists[0].terms.len() == 1
        && matches!(
            &grammar.grouped_rule_lists[0].terms[0],
            sygus_parser::SyGuSGTerm::Constant(_)
        )
}

fn parse_solution_define_funs(
    solution: &str,
) -> Result<Vec<(String, Vec<SortedVar>, SyGuSTerm)>, String> {
    let trimmed = solution.trim();
    let content = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let file = SyGuSFile::from_str(content)
        .map_err(|_| format!("failed to parse solver response: {trimmed}"))?;
    let define_funs = file
        .cmds
        .into_iter()
        .filter_map(|cmd| match cmd {
            SyGuSCmd::DefineFun(name, args, _sort, body) => Some((name, args, body)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if define_funs.is_empty() {
        return Err("solver response did not contain a define-fun".to_string());
    }
    Ok(define_funs)
}
