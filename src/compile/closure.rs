use crate::{parser::DelisProg, sorts::SortEnv};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::time::Instant;
use sygus_parser::{GrammarDef, Identifier, Literal, Sort, SyGuSCmd, SyGuSGTerm, SyGuSTerm};

use super::eval::{
    is_closure_constraint, term_mentions_symbol, Bits, DefineFun, EvalContext, EvalEnv, EvalValue,
};

#[derive(Clone)]
struct Value {
    bits: Bits,
    width: usize,
    term: SyGuSTerm,
}

struct Nonterminal {
    width: usize,
    values: Vec<Value>,
    seen: HashSet<Bits>,
    productions: Vec<Production>,
    bool_ops: BoolOps,
}

#[derive(Clone)]
struct Production {
    ctor: String,
    args: Vec<String>,
}

#[derive(Default, Clone)]
struct BoolOps {
    true_ctor: Option<String>,
    false_ctor: Option<String>,
    not_ctor: Option<String>,
    and_ctor: Option<String>,
    or_ctor: Option<String>,
}

impl DelisProg {
    pub fn solve_by_closure(&self) -> Result<Option<String>, String> {
        let cmds = self.to_sygus()?;
        self.solve_by_closure_with_cmds(cmds)
    }

    /// Solve from pre-built SyGuS BV cmds. Used when the SyGuS lowering is
    /// shared with the parallel SyGuS backend — see `main::solve`.
    pub fn solve_by_closure_with_cmds(
        &self,
        cmds: Vec<SyGuSCmd>,
    ) -> Result<Option<String>, String> {
        // Multi-synth-fun problems (aux closure pattern emitted by NativeCompiler
        // when (parent ...) is used) aren't supported by this enumerative
        // backend — its eval expects every identifier to be a value, define-fun,
        // or env binding, not a synth-fun reference. Bail to None so the parallel
        // SyGuS backend handles it alone.
        let synth_fun_count = cmds
            .iter()
            .filter(|cmd| matches!(cmd, SyGuSCmd::SynthFun(..)))
            .count();
        if synth_fun_count > 1 {
            log::info!(
                "closure: skipping (program has {} synth-funs; closure backend supports 1)",
                synth_fun_count
            );
            return Ok(None);
        }
        let start = Instant::now();
        let Some(mut solver) = ClosureSolver::new(self, cmds)? else {
            return Ok(None);
        };
        log::debug!("closure: solver initialized in {:?}", start.elapsed());
        Ok(Some(solver.solve()?))
    }
}

fn is_constant_grammar(grammar: &GrammarDef) -> bool {
    grammar.grouped_rule_lists.len() == 1
        && grammar.grouped_rule_lists[0].terms.len() == 1
        && matches!(
            &grammar.grouped_rule_lists[0].terms[0],
            SyGuSGTerm::Constant(_)
        )
}

struct ClosureSolver<'a> {
    problem: &'a DelisProg,
    synth_name: String,
    synth_sort: Sort,
    start: String,
    funcs: HashMap<String, DefineFun>,
    nonterminals: Vec<Nonterminal>,
    nt_by_symbol: HashMap<String, usize>,
    constraints: Vec<SyGuSTerm>,
    processed: HashMap<(usize, usize), HashSet<Vec<usize>>>,
    /// True if any constructor's body references the synth-fun by name (e.g.
    /// `re.*`'s body extracts bits of `r`). Such constructors encode a
    /// finite-aspect least fixed point: their meaning at a given argument
    /// tuple is the LFP of `body(args, r)` iterated from `r = 0` (Krogmeier
    /// & Madhusudan 2023). The solver computes that LFP inline per
    /// evaluation instead of treating `r` as an extra free input.
    synth_self_ref: bool,
    /// Index of the start NT (the synth-fun's nonterminal).
    start_nt_idx: usize,
}

impl<'a> ClosureSolver<'a> {
    fn new(problem: &'a DelisProg, cmds: Vec<SyGuSCmd>) -> Result<Option<Self>, String> {
        let sort_env = SortEnv::from_cmds(&cmds);
        let mut funcs = HashMap::default();
        let mut synth = None;
        let mut constraints = Vec::new();
        for cmd in cmds {
            match cmd {
                SyGuSCmd::DefineFun(name, args, sort, body) => {
                    let width = sort_env.bv_width(&sort)?;
                    funcs.insert(
                        name,
                        DefineFun {
                            args: args.into_iter().map(|arg| arg.name).collect(),
                            body,
                            width,
                        },
                    );
                }
                SyGuSCmd::SynthFun(name, args, sort, grammar) => {
                    let width = sort_env.bv_width(&sort)?;
                    // Synth-funs are also available as zero-arity functions
                    // so that define-fun bodies can reference them (e.g. the
                    // aux closure placeholder).
                    if grammar.as_ref().is_some_and(|g| is_constant_grammar(g)) {
                        funcs.insert(
                            name.clone(),
                            DefineFun {
                                args: Vec::new(),
                                body: SyGuSTerm::Literal(Literal::BinConst("0".repeat(width))),
                                width,
                            },
                        );
                    } else if args.is_empty() {
                        let g =
                            grammar.ok_or_else(|| "closure requires SyGuS grammar".to_string())?;
                        synth = Some((name, sort, g));
                    }
                }
                SyGuSCmd::Constraint(term) => {
                    if is_closure_constraint(&term, &synth.as_ref().map(|(n, _, _)| n.clone())) {
                        continue;
                    }
                    constraints.push(term);
                }
                _ => {}
            }
        }
        let Some((synth_name, synth_sort, grammar)) = synth else {
            return Ok(None);
        };
        let (nonterminals, nt_by_symbol, start) = build_nonterminals(grammar, &sort_env)?;
        let start_nt_idx = nt_by_symbol
            .get(&start)
            .copied()
            .ok_or_else(|| format!("start nonterminal {start} not in grammar"))?;
        let synth_self_ref = funcs
            .values()
            .any(|f| term_mentions_symbol(&f.body, &synth_name));
        Ok(Some(Self {
            problem,
            synth_name,
            synth_sort,
            start,
            funcs,
            nonterminals,
            nt_by_symbol,
            constraints,
            processed: HashMap::default(),
            synth_self_ref,
            start_nt_idx,
        }))
    }

    fn solve(&mut self) -> Result<String, String> {
        let start = Instant::now();
        let mut iteration = 0usize;
        log::debug!("closure: solve started");
        loop {
            iteration += 1;
            let mut changed = false;
            for nt_idx in 0..self.nonterminals.len() {
                changed |= self.boolean_partition_close(nt_idx)?;
                let prod_count = self.nonterminals[nt_idx].productions.len();
                for prod_idx in 0..prod_count {
                    changed |= self.apply_production(nt_idx, prod_idx)?;
                    if let Some(term) = self.satisfying_start_term()? {
                        log::info!(
                            "closure: solved at iteration {} in {:?}",
                            iteration,
                            start.elapsed()
                        );
                        return Ok(self.define_fun(term));
                    }
                }
            }
            if let Some(term) = self.satisfying_start_term()? {
                log::info!(
                    "closure: solved at iteration-boundary {} in {:?}",
                    iteration,
                    start.elapsed()
                );
                return Ok(self.define_fun(term));
            }
            if !changed {
                log::info!(
                    "closure: infeasible (fixed point at iteration {} after {:?})",
                    iteration,
                    start.elapsed()
                );
                return Ok("infeasible".to_string());
            }
            if iteration <= 5 || iteration % 25 == 0 {
                log::debug!("closure: iteration {} in {:?}", iteration, start.elapsed());
            }
        }
    }

    fn define_fun(&self, body: SyGuSTerm) -> String {
        let (name, args, sort) = synth_signature_parts(self.problem)
            .unwrap_or_else(|| (self.synth_name.clone(), Vec::new(), self.synth_sort.clone()));
        let cmd = SyGuSCmd::DefineFun(name, args, sort, body);
        format!("(\n{}\n)", cmd)
    }

    fn satisfying_start_term(&self) -> Result<Option<SyGuSTerm>, String> {
        let start_idx = *self
            .nt_by_symbol
            .get(&self.start)
            .ok_or_else(|| format!("unknown start nonterminal {}", self.start))?;
        for value in &self.nonterminals[start_idx].values {
            if self.satisfies(&value.bits)? {
                return Ok(Some(value.term.clone()));
            }
        }
        Ok(None)
    }

    fn satisfies(&self, bits: &Bits) -> Result<bool, String> {
        let mut base = HashMap::default();
        base.insert(
            self.synth_name.clone(),
            EvalValue::Bv(
                bits.clone(),
                self.nonterminals[self.nt_by_symbol[&self.start]].width,
            ),
        );
        let mut env = EvalEnv::with_base(base);
        for constraint in &self.constraints {
            if !self.evaluator().eval_bool(constraint, &mut env)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn apply_production(&mut self, nt_idx: usize, prod_idx: usize) -> Result<bool, String> {
        let prod = self.nonterminals[nt_idx].productions[prod_idx].clone();
        if prod.args.is_empty() {
            let key = (nt_idx, prod_idx);
            let done = self.processed.entry(key).or_default();
            if !done.insert(Vec::new()) {
                return Ok(false);
            }
            let (bits, term) = self.eval_constructor(&prod, &[])?;
            return Ok(self.insert_value(nt_idx, bits, term));
        }
        let arg_indices = prod
            .args
            .iter()
            .map(|arg| {
                self.nt_by_symbol
                    .get(arg)
                    .copied()
                    .ok_or_else(|| format!("unknown grammar nonterminal {arg}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut tuples = Vec::new();
        collect_tuples(
            &self.nonterminals,
            &arg_indices,
            0,
            &mut Vec::new(),
            &mut tuples,
        );
        let mut changed = false;
        for tuple in tuples {
            let key = (nt_idx, prod_idx);
            if !self.processed.entry(key).or_default().insert(tuple.clone()) {
                continue;
            }
            let args = tuple
                .iter()
                .zip(&arg_indices)
                .map(|(value_idx, nt)| self.nonterminals[*nt].values[*value_idx].clone())
                .collect::<Vec<_>>();
            let (bits, term) = self.eval_constructor(&prod, &args)?;
            changed |= self.insert_value(nt_idx, bits, term);
        }
        Ok(changed)
    }

    fn eval_constructor(
        &self,
        prod: &Production,
        args: &[Value],
    ) -> Result<(Bits, SyGuSTerm), String> {
        let func = self
            .funcs
            .get(&prod.ctor)
            .ok_or_else(|| format!("unknown constructor function {}", prod.ctor))?;
        if func.args.len() != args.len() {
            return Err(format!(
                "constructor {} expects {} args, got {}",
                prod.ctor,
                func.args.len(),
                args.len()
            ));
        }
        let needs_lfp = self.synth_self_ref && term_mentions_symbol(&func.body, &self.synth_name);
        let bits = if needs_lfp {
            self.eval_constructor_lfp(func, args)?
        } else {
            self.eval_constructor_once(func, args, None)?
        };
        let term = if args.is_empty() {
            SyGuSTerm::Identifier(Identifier::Symbol(prod.ctor.clone()))
        } else {
            SyGuSTerm::Application(
                Identifier::Symbol(prod.ctor.clone()),
                args.iter().map(|arg| arg.term.clone()).collect(),
            )
        };
        Ok((bits, term))
    }

    /// Evaluate a constructor body once with the given args, optionally
    /// binding the synth-fun's name to a given iterate.
    fn eval_constructor_once(
        &self,
        func: &DefineFun,
        args: &[Value],
        self_binding: Option<&Bits>,
    ) -> Result<Bits, String> {
        let mut env = HashMap::default();
        if let Some(bits) = self_binding {
            let width = self.nonterminals[self.start_nt_idx].width;
            env.insert(self.synth_name.clone(), EvalValue::Bv(bits.clone(), width));
        }
        for (name, value) in func.args.iter().zip(args) {
            env.insert(name.clone(), EvalValue::Bv(value.bits.clone(), value.width));
        }
        self.evaluator()
            .eval_bv(&func.body, &mut EvalEnv::with_base(env), func.width)
    }

    /// Compute the least fixed point of `body(args, r)` starting from
    /// `r = 0`. The body is monotone in `r` under the finite-aspect
    /// encoding, so iteration converges in at most `width + 1` steps.
    fn eval_constructor_lfp(&self, func: &DefineFun, args: &[Value]) -> Result<Bits, String> {
        let width = self.nonterminals[self.start_nt_idx].width;
        let mut current = Bits::zero(width);
        for _ in 0..=width {
            let next = self.eval_constructor_once(func, args, Some(&current))?;
            if next == current {
                return Ok(next);
            }
            current = next;
        }
        Err(format!(
            "constructor {} LFP did not converge in {} steps",
            self.synth_name,
            width + 1
        ))
    }

    fn insert_value(&mut self, nt_idx: usize, bits: Bits, term: SyGuSTerm) -> bool {
        if self.nonterminals[nt_idx].seen.insert(bits.clone()) {
            let width = self.nonterminals[nt_idx].width;
            self.nonterminals[nt_idx]
                .values
                .push(Value { bits, width, term });
            true
        } else {
            false
        }
    }

    fn boolean_partition_close(&mut self, nt_idx: usize) -> Result<bool, String> {
        let ops = self.nonterminals[nt_idx].bool_ops.clone();
        let (Some(true_ctor), Some(false_ctor), Some(not_ctor), Some(and_ctor), Some(or_ctor)) = (
            ops.true_ctor,
            ops.false_ctor,
            ops.not_ctor,
            ops.and_ctor,
            ops.or_ctor,
        ) else {
            return Ok(false);
        };
        let width = self.nonterminals[nt_idx].width;
        if width > 24 || self.nonterminals[nt_idx].values.is_empty() {
            return Ok(false);
        }
        let generators = self.nonterminals[nt_idx].values.clone();
        let mut signatures: HashMap<Vec<bool>, Vec<usize>> = HashMap::default();
        for bit in 0..width {
            let sig = generators
                .iter()
                .map(|value| value.bits.get(bit))
                .collect::<Vec<_>>();
            signatures.entry(sig).or_default().push(bit);
        }
        let blocks = signatures.into_values().collect::<Vec<_>>();
        if blocks.len() > 16 {
            return Ok(false);
        }
        let false_term = SyGuSTerm::Identifier(Identifier::Symbol(false_ctor.clone()));
        let true_term = SyGuSTerm::Identifier(Identifier::Symbol(true_ctor));
        let mut changed = false;
        for mask in 0usize..(1usize << blocks.len()) {
            let mut bits = Bits::zero(width);
            for (block_idx, block) in blocks.iter().enumerate() {
                if ((mask >> block_idx) & 1) == 1 {
                    for bit in block {
                        bits.set(*bit);
                    }
                }
            }
            if self.nonterminals[nt_idx].seen.contains(&bits) {
                continue;
            }
            let term = if mask == 0 {
                false_term.clone()
            } else if mask + 1 == (1usize << blocks.len()) {
                true_term.clone()
            } else {
                let mut disjuncts = Vec::new();
                for (block_idx, block) in blocks.iter().enumerate() {
                    if ((mask >> block_idx) & 1) == 0 {
                        continue;
                    }
                    let sample = block[0];
                    let mut conjunct = None;
                    for gen in &generators {
                        let lit = if gen.bits.get(sample) {
                            gen.term.clone()
                        } else {
                            SyGuSTerm::Application(
                                Identifier::Symbol(not_ctor.clone()),
                                vec![gen.term.clone()],
                            )
                        };
                        conjunct = Some(match conjunct {
                            Some(prev) => SyGuSTerm::Application(
                                Identifier::Symbol(and_ctor.clone()),
                                vec![prev, lit],
                            ),
                            None => lit,
                        });
                    }
                    if let Some(term) = conjunct {
                        disjuncts.push(term);
                    }
                }
                disjuncts
                    .into_iter()
                    .reduce(|lhs, rhs| {
                        SyGuSTerm::Application(Identifier::Symbol(or_ctor.clone()), vec![lhs, rhs])
                    })
                    .unwrap_or(false_term.clone())
            };
            changed |= self.insert_value(nt_idx, bits, term);
        }
        Ok(changed)
    }

    fn evaluator(&self) -> EvalContext<'_> {
        EvalContext { funcs: &self.funcs }
    }
}

fn build_nonterminals(
    grammar: GrammarDef,
    sort_env: &SortEnv,
) -> Result<(Vec<Nonterminal>, HashMap<String, usize>, String), String> {
    let mut nonterminals = Vec::new();
    let mut nt_by_symbol = HashMap::default();
    let start = grammar
        .grouped_rule_lists
        .first()
        .map(|group| group.symbol.clone())
        .ok_or_else(|| "empty SyGuS grammar".to_string())?;
    for group in grammar.grouped_rule_lists {
        let width = sort_env.bv_width(&group.sort)?;
        let mut productions = Vec::new();
        let mut bool_ops = BoolOps::default();
        for term in group.terms {
            let prod = Production::from_gterm(term)?;
            bool_ops.classify(&prod);
            productions.push(prod);
        }
        let idx = nonterminals.len();
        nt_by_symbol.insert(group.symbol.clone(), idx);
        nonterminals.push(Nonterminal {
            width,
            values: Vec::new(),
            seen: HashSet::default(),
            productions,
            bool_ops,
        });
    }
    Ok((nonterminals, nt_by_symbol, start))
}

impl Production {
    fn from_gterm(term: SyGuSGTerm) -> Result<Self, String> {
        let SyGuSGTerm::SyGuSTerm(term) = term else {
            return Err("closure does not support Constant/Variable grammar terms".to_string());
        };
        match term {
            SyGuSTerm::Identifier(Identifier::Symbol(name)) => Ok(Production {
                ctor: name,
                args: Vec::new(),
            }),
            SyGuSTerm::Application(Identifier::Symbol(name), args) => Ok(Production {
                ctor: name,
                args: args
                    .into_iter()
                    .map(|arg| match arg {
                        SyGuSTerm::Identifier(Identifier::Symbol(name)) => Ok(name),
                        _ => Err(format!(
                            "grammar constructor argument must be a nonterminal: {arg}"
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            other => Err(format!("unsupported grammar term {other}")),
        }
    }
}

impl BoolOps {
    fn classify(&mut self, prod: &Production) {
        if !prod.args.is_empty() {
            if prod.args.len() == 1 && prod.ctor.ends_with(".not") {
                self.not_ctor = Some(prod.ctor.clone());
            } else if prod.args.len() == 2 && prod.ctor.ends_with(".and") {
                self.and_ctor = Some(prod.ctor.clone());
            } else if prod.args.len() == 2 && prod.ctor.ends_with(".or") {
                self.or_ctor = Some(prod.ctor.clone());
            }
            return;
        }
        if prod.ctor.ends_with(".true") {
            self.true_ctor = Some(prod.ctor.clone());
        } else if prod.ctor.ends_with(".false") {
            self.false_ctor = Some(prod.ctor.clone());
        }
    }
}

fn collect_tuples(
    nonterminals: &[Nonterminal],
    arg_indices: &[usize],
    pos: usize,
    current: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    if pos == arg_indices.len() {
        out.push(current.clone());
        return;
    }
    let nt = &nonterminals[arg_indices[pos]];
    for idx in 0..nt.values.len() {
        current.push(idx);
        collect_tuples(nonterminals, arg_indices, pos + 1, current, out);
        current.pop();
    }
}

fn synth_signature_parts(
    problem: &DelisProg,
) -> Option<(String, Vec<sygus_parser::SortedVar>, Sort)> {
    problem.cmds.iter().find_map(|cmd| match cmd {
        crate::parser::DelisCmd::As(SyGuSCmd::SynthFun(name, args, sort, _)) => {
            Some((name.clone(), args.clone(), sort.clone()))
        }
        _ => None,
    })
}
