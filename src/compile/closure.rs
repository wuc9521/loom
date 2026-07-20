use crate::{parser::DelisProg, sorts::SortEnv};
use crate::fxmap::{HashMap, HashSet};
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

#[derive(Clone, Copy)]
struct ValueRef<'a> {
    bits: &'a Bits,
    width: usize,
    term: &'a SyGuSTerm,
}

struct Nonterminal {
    width: usize,
    values: Vec<Value>,
    seen: HashSet<Bits>,
    productions: Vec<Production>,
    bool_ops: BoolOps,
    bool_closure_generators: usize,
}

#[derive(Clone)]
struct Production {
    ctor: String,
    args: Vec<String>,
    arg_indices: Vec<usize>,
    requires_self_fixpoint: bool,
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
    funcs: HashMap<String, DefineFun>,
    nonterminals: Vec<Nonterminal>,
    constraints: Vec<SyGuSTerm>,
    processed: HashMap<(usize, usize), Vec<usize>>,
    next_start_check: usize,
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
        let (mut nonterminals, nt_by_symbol, start) = build_nonterminals(grammar, &sort_env)?;
        let start_nt_idx = nt_by_symbol
            .get(&start)
            .copied()
            .ok_or_else(|| format!("start nonterminal {start} not in grammar"))?;
        for nt in &mut nonterminals {
            for prod in &mut nt.productions {
                prod.requires_self_fixpoint = funcs
                    .get(&prod.ctor)
                    .is_some_and(|f| term_mentions_symbol(&f.body, &synth_name));
            }
        }
        Ok(Some(Self {
            problem,
            synth_name,
            synth_sort,
            funcs,
            nonterminals,
            constraints,
            processed: HashMap::default(),
            next_start_check: 0,
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
        let (name, args, sort) = self
            .problem
            .synth_signature()
            .unwrap_or_else(|| (self.synth_name.clone(), Vec::new(), self.synth_sort.clone()));
        let cmd = SyGuSCmd::DefineFun(name, args, sort, body);
        format!("(\n{}\n)", cmd)
    }

    fn satisfying_start_term(&mut self) -> Result<Option<SyGuSTerm>, String> {
        while self.next_start_check < self.nonterminals[self.start_nt_idx].values.len() {
            let idx = self.next_start_check;
            self.next_start_check += 1;
            let value = &self.nonterminals[self.start_nt_idx].values[idx];
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
            EvalValue::Bv(bits.clone(), self.nonterminals[self.start_nt_idx].width),
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
        let key = (nt_idx, prod_idx);
        if prod.args.is_empty() {
            if self.processed.contains_key(&key) {
                return Ok(false);
            }
            self.processed.insert(key, Vec::new());
            let bits = self.eval_constructor_bits(&prod, &[])?;
            if self.nonterminals[nt_idx].seen.contains(&bits) {
                return Ok(false);
            }
            let term = self.constructor_term(&prod, &[]);
            return Ok(self.insert_value(nt_idx, bits, term));
        }
        let current_lengths = prod
            .arg_indices
            .iter()
            .map(|nt| self.nonterminals[*nt].values.len())
            .collect::<Vec<_>>();
        let previous_lengths = self
            .processed
            .get(&key)
            .cloned()
            .unwrap_or_else(|| vec![0; prod.arg_indices.len()]);
        if current_lengths.iter().any(|len| *len == 0) {
            self.processed.insert(key, current_lengths);
            return Ok(false);
        }
        let mut changed = false;
        for_each_tuple_index(&current_lengths, &mut Vec::new(), &mut |tuple| {
            if tuple_is_inside_lengths(tuple, &previous_lengths) {
                return Ok(());
            }
            changed |= self.apply_tuple(nt_idx, &prod, &prod.arg_indices, tuple)?;
            Ok(())
        })?;
        self.processed.insert(key, current_lengths);
        Ok(changed)
    }

    fn apply_tuple(
        &mut self,
        nt_idx: usize,
        prod: &Production,
        arg_indices: &[usize],
        tuple: &[usize],
    ) -> Result<bool, String> {
        let Some((bits, term)) = ({
            let args = tuple
                .iter()
                .zip(arg_indices)
                .map(|(value_idx, nt)| {
                    let value = &self.nonterminals[*nt].values[*value_idx];
                    ValueRef {
                        bits: &value.bits,
                        width: value.width,
                        term: &value.term,
                    }
                })
                .collect::<Vec<_>>();
            let bits = self.eval_constructor_bits(prod, &args)?;
            if self.nonterminals[nt_idx].seen.contains(&bits) {
                None
            } else {
                Some((bits, self.constructor_term(prod, &args)))
            }
        }) else {
            return Ok(false);
        };
        Ok(self.insert_value(nt_idx, bits, term))
    }

    fn eval_constructor_bits(
        &self,
        prod: &Production,
        args: &[ValueRef<'_>],
    ) -> Result<Bits, String> {
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
        let bits = if prod.requires_self_fixpoint {
            self.eval_constructor_lfp(func, args)?
        } else {
            self.eval_constructor_once(func, args, None)?
        };
        Ok(bits)
    }

    fn constructor_term(&self, prod: &Production, args: &[ValueRef<'_>]) -> SyGuSTerm {
        if args.is_empty() {
            SyGuSTerm::Identifier(Identifier::Symbol(prod.ctor.clone()))
        } else {
            SyGuSTerm::Application(
                Identifier::Symbol(prod.ctor.clone()),
                args.iter().map(|arg| arg.term.clone()).collect(),
            )
        }
    }

    /// Evaluate a constructor body once with the given args, optionally
    /// binding the synth-fun's name to a given iterate.
    fn eval_constructor_once(
        &self,
        func: &DefineFun,
        args: &[ValueRef<'_>],
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
    fn eval_constructor_lfp(
        &self,
        func: &DefineFun,
        args: &[ValueRef<'_>],
    ) -> Result<Bits, String> {
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
        let generator_count = self.nonterminals[nt_idx].values.len();
        if width > 24
            || generator_count == 0
            || generator_count == self.nonterminals[nt_idx].bool_closure_generators
        {
            return Ok(false);
        }
        let mut signatures: HashMap<Vec<bool>, Vec<usize>> = HashMap::default();
        for bit in 0..width {
            let sig = (0..generator_count)
                .map(|idx| self.nonterminals[nt_idx].values[idx].bits.get(bit))
                .collect::<Vec<_>>();
            signatures.entry(sig).or_default().push(bit);
        }
        let blocks = signatures.into_values().collect::<Vec<_>>();
        if blocks.len() > 16 {
            self.nonterminals[nt_idx].bool_closure_generators = generator_count;
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
            let term = {
                if mask == 0 {
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
                        for gen_idx in 0..generator_count {
                            let gen = &self.nonterminals[nt_idx].values[gen_idx];
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
                            SyGuSTerm::Application(
                                Identifier::Symbol(or_ctor.clone()),
                                vec![lhs, rhs],
                            )
                        })
                        .unwrap_or(false_term.clone())
                }
            };
            changed |= self.insert_value(nt_idx, bits, term);
        }
        self.nonterminals[nt_idx].bool_closure_generators = generator_count;
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
            bool_closure_generators: 0,
        });
    }
    for nt in &mut nonterminals {
        for prod in &mut nt.productions {
            prod.arg_indices = prod
                .args
                .iter()
                .map(|arg| {
                    nt_by_symbol
                        .get(arg)
                        .copied()
                        .ok_or_else(|| format!("unknown grammar nonterminal {arg}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
        }
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
                arg_indices: Vec::new(),
                requires_self_fixpoint: false,
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
                arg_indices: Vec::new(),
                requires_self_fixpoint: false,
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

fn tuple_is_inside_lengths(tuple: &[usize], lengths: &[usize]) -> bool {
    tuple
        .iter()
        .zip(lengths)
        .all(|(idx, processed_len)| *idx < *processed_len)
}

fn for_each_tuple_index(
    lengths: &[usize],
    current: &mut Vec<usize>,
    f: &mut impl FnMut(&[usize]) -> Result<(), String>,
) -> Result<(), String> {
    if current.len() == lengths.len() {
        return f(current);
    }
    let pos = current.len();
    for idx in 0..lengths[pos] {
        current.push(idx);
        for_each_tuple_index(lengths, current, f)?;
        current.pop();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(name: &str) -> SyGuSTerm {
        SyGuSTerm::Identifier(Identifier::Symbol(name.to_string()))
    }

    fn value(bits: &str, width: usize, name: &str) -> Value {
        Value {
            bits: Bits::from_bin(bits),
            width,
            term: symbol(name),
        }
    }

    fn nonterminal(width: usize, values: Vec<Value>, productions: Vec<Production>) -> Nonterminal {
        let seen = values.iter().map(|value| value.bits.clone()).collect();
        Nonterminal {
            width,
            values,
            seen,
            productions,
            bool_ops: BoolOps::default(),
            bool_closure_generators: 0,
        }
    }

    #[test]
    fn tuple_frontier_identifies_previous_rectangle() {
        assert!(tuple_is_inside_lengths(&[0, 0], &[2, 1]));
        assert!(tuple_is_inside_lengths(&[1, 0], &[2, 1]));
        assert!(!tuple_is_inside_lengths(&[0, 1], &[2, 1]));
        assert!(!tuple_is_inside_lengths(&[2, 0], &[2, 1]));

        let mut tuples = Vec::new();
        for_each_tuple_index(&[2, 2], &mut Vec::new(), &mut |tuple| {
            tuples.push(tuple.to_vec());
            Ok(())
        })
        .unwrap();
        assert_eq!(tuples, vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1]]);
    }

    #[test]
    fn apply_production_tracks_compact_frontier_as_child_pools_grow() {
        let problem = DelisProg { cmds: Vec::new() };
        let pair_body = SyGuSTerm::Application(
            Identifier::Symbol("concat".to_string()),
            vec![symbol("x"), symbol("y")],
        );
        let mut funcs = HashMap::default();
        funcs.insert(
            "pair".to_string(),
            DefineFun {
                args: vec!["x".to_string(), "y".to_string()],
                body: pair_body,
                width: 2,
            },
        );

        let mut solver = ClosureSolver {
            problem: &problem,
            synth_name: "g".to_string(),
            synth_sort: Sort::Simple(Identifier::Symbol("Out".to_string())),
            funcs,
            nonterminals: vec![
                nonterminal(1, vec![value("0", 1, "a0"), value("1", 1, "a1")], vec![]),
                nonterminal(1, vec![value("0", 1, "b0")], vec![]),
                nonterminal(
                    2,
                    vec![],
                    vec![Production {
                        ctor: "pair".to_string(),
                        args: vec!["A".to_string(), "B".to_string()],
                        arg_indices: vec![0, 1],
                        requires_self_fixpoint: false,
                    }],
                ),
            ],
            constraints: Vec::new(),
            processed: HashMap::default(),
            next_start_check: 0,
            start_nt_idx: 2,
        };

        assert!(solver.apply_production(2, 0).unwrap());
        assert_eq!(solver.nonterminals[2].values.len(), 2);
        assert_eq!(solver.processed[&(2, 0)], vec![2, 1]);

        let new_b = value("1", 1, "b1");
        solver.nonterminals[1].seen.insert(new_b.bits.clone());
        solver.nonterminals[1].values.push(new_b);

        assert!(solver.apply_production(2, 0).unwrap());
        assert_eq!(solver.nonterminals[2].values.len(), 4);
        assert_eq!(solver.processed[&(2, 0)], vec![2, 2]);

        assert!(!solver.apply_production(2, 0).unwrap());
        assert_eq!(solver.nonterminals[2].values.len(), 4);
        assert_eq!(solver.processed[&(2, 0)], vec![2, 2]);
    }

    #[test]
    fn satisfying_start_term_checks_only_new_values() {
        let problem = DelisProg { cmds: Vec::new() };
        let constraint = SyGuSTerm::Application(
            Identifier::Symbol("=".to_string()),
            vec![
                symbol("g"),
                SyGuSTerm::Literal(Literal::BinConst("1".to_string())),
            ],
        );
        let mut solver = ClosureSolver {
            problem: &problem,
            synth_name: "g".to_string(),
            synth_sort: Sort::Simple(Identifier::Symbol("Start".to_string())),
            funcs: HashMap::default(),
            nonterminals: vec![nonterminal(1, vec![value("0", 1, "zero")], vec![])],
            constraints: vec![constraint],
            processed: HashMap::default(),
            next_start_check: 0,
            start_nt_idx: 0,
        };

        assert!(solver.satisfying_start_term().unwrap().is_none());
        assert_eq!(solver.next_start_check, 1);

        solver.insert_value(0, Bits::from_bin("1"), symbol("one"));

        assert!(solver.satisfying_start_term().unwrap().is_some());
        assert_eq!(solver.next_start_check, 2);
    }

    #[test]
    fn boolean_partition_close_skips_unchanged_generators() {
        let problem = DelisProg { cmds: Vec::new() };
        let mut nt = nonterminal(1, vec![value("0", 1, "zero"), value("1", 1, "one")], vec![]);
        nt.bool_ops = BoolOps {
            true_ctor: Some("Bool.true".to_string()),
            false_ctor: Some("Bool.false".to_string()),
            not_ctor: Some("Bool.not".to_string()),
            and_ctor: Some("Bool.and".to_string()),
            or_ctor: Some("Bool.or".to_string()),
        };
        let mut solver = ClosureSolver {
            problem: &problem,
            synth_name: "g".to_string(),
            synth_sort: Sort::Simple(Identifier::Symbol("Start".to_string())),
            funcs: HashMap::default(),
            nonterminals: vec![nt],
            constraints: Vec::new(),
            processed: HashMap::default(),
            next_start_check: 0,
            start_nt_idx: 0,
        };

        assert!(!solver.boolean_partition_close(0).unwrap());
        assert_eq!(solver.nonterminals[0].bool_closure_generators, 2);
        assert!(!solver.boolean_partition_close(0).unwrap());
        assert_eq!(solver.nonterminals[0].values.len(), 2);
    }
}
