use crate::{parser::DelisProg, sorts::SortEnv};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::time::Instant;
use sygus_parser::{
    GrammarDef, Identifier, Index, Literal, Sort, SortedVar, SyGuSCmd, SyGuSFile, SyGuSGTerm,
    SyGuSTerm,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Bits(Vec<u64>);

impl Bits {
    fn zero(width: usize) -> Self {
        Self(vec![0; width.div_ceil(64)])
    }

    fn from_lsb(values: &[bool]) -> Self {
        let mut bits = Self::zero(values.len());
        for (idx, value) in values.iter().enumerate() {
            if *value {
                bits.set(idx);
            }
        }
        bits
    }

    fn from_bin(text: &str) -> Self {
        let text = text.strip_prefix("#b").unwrap_or(text);
        Self::from_lsb(&text.chars().rev().map(|ch| ch == '1').collect::<Vec<_>>())
    }

    fn get(&self, idx: usize) -> bool {
        (self.0[idx / 64] & (1u64 << (idx % 64))) != 0
    }

    fn set(&mut self, idx: usize) {
        self.0[idx / 64] |= 1u64 << (idx % 64);
    }

    fn not(&self, width: usize) -> Self {
        let mut out = Self(self.0.iter().map(|word| !word).collect());
        out.mask(width);
        out
    }

    fn and(&self, other: &Self, width: usize) -> Self {
        let mut out = Self(
            self.0
                .iter()
                .zip(&other.0)
                .map(|(lhs, rhs)| lhs & rhs)
                .collect(),
        );
        out.mask(width);
        out
    }

    fn or(&self, other: &Self, width: usize) -> Self {
        let mut out = Self(
            self.0
                .iter()
                .zip(&other.0)
                .map(|(lhs, rhs)| lhs | rhs)
                .collect(),
        );
        out.mask(width);
        out
    }

    fn concat(high: &Self, high_width: usize, low: &Self, low_width: usize) -> Self {
        let mut values = Vec::with_capacity(high_width + low_width);
        for idx in 0..low_width {
            values.push(low.get(idx));
        }
        for idx in 0..high_width {
            values.push(high.get(idx));
        }
        Self::from_lsb(&values)
    }

    fn extract(&self, high: usize, low: usize) -> Self {
        Self::from_lsb(&(low..=high).map(|idx| self.get(idx)).collect::<Vec<_>>())
    }

    fn mask(&mut self, width: usize) {
        let rem = width % 64;
        if rem != 0 {
            if let Some(last) = self.0.last_mut() {
                *last &= (1u64 << rem) - 1;
            }
        }
    }
}

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

#[derive(Clone)]
struct DefineFun {
    args: Vec<String>,
    body: SyGuSTerm,
    width: usize,
}

struct EvalContext<'a> {
    funcs: &'a HashMap<String, DefineFun>,
}

#[derive(Default)]
struct EvalEnv {
    base: HashMap<String, EvalValue>,
    scoped: HashMap<String, Vec<EvalValue>>,
    scopes: Vec<Vec<String>>,
}

impl EvalEnv {
    fn with_base(base: HashMap<String, EvalValue>) -> Self {
        Self {
            base,
            scoped: HashMap::default(),
            scopes: Vec::new(),
        }
    }

    fn get(&self, name: &str) -> Option<&EvalValue> {
        self.scoped
            .get(name)
            .and_then(|values| values.last())
            .or_else(|| self.base.get(name))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn insert_local(&mut self, name: String, value: EvalValue) {
        self.scoped.entry(name.clone()).or_default().push(value);
        self.scopes
            .last_mut()
            .expect("scope must exist before inserting locals")
            .push(name);
    }

    fn pop_scope(&mut self) {
        if let Some(names) = self.scopes.pop() {
            for name in names.into_iter().rev() {
                if let Some(values) = self.scoped.get_mut(&name) {
                    values.pop();
                    if values.is_empty() {
                        self.scoped.remove(&name);
                    }
                }
            }
        }
    }
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

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    let verifier = ConstraintVerifier::new(cmds)?;
    verifier.failing_constraint_indices(solution)
}

fn is_constant_grammar(grammar: &GrammarDef) -> bool {
    grammar.grouped_rule_lists.len() == 1
        && grammar.grouped_rule_lists[0].terms.len() == 1
        && matches!(&grammar.grouped_rule_lists[0].terms[0], SyGuSGTerm::Constant(_))
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
                                body: SyGuSTerm::Literal(Literal::BinConst(
                                    "0".repeat(width),
                                )),
                                width,
                            },
                        );
                    } else if args.is_empty() {
                        let g = grammar.ok_or_else(|| "closure requires SyGuS grammar".to_string())?;
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
                log::debug!(
                    "closure: iteration {} in {:?}",
                    iteration,
                    start.elapsed()
                );
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
            env.insert(
                self.synth_name.clone(),
                EvalValue::Bv(bits.clone(), width),
            );
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
        args: &[Value],
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum EvalValue {
    Bool(bool),
    Nat(usize),
    Bv(Bits, usize),
}

impl EvalValue {
    fn kind(&self) -> &'static str {
        match self {
            EvalValue::Bool(_) => "Bool",
            EvalValue::Nat(_) => "Nat",
            EvalValue::Bv(_, _) => "BitVec",
        }
    }
}

impl<'a> EvalContext<'a> {
    fn eval_bool(&self, term: &SyGuSTerm, env: &mut EvalEnv) -> Result<bool, String> {
        match self.eval(term, env)? {
            EvalValue::Bool(value) => Ok(value),
            other => Err(format!("expected Bool, got {:?}", other.kind())),
        }
    }

    fn eval_bv(&self, term: &SyGuSTerm, env: &mut EvalEnv, width: usize) -> Result<Bits, String> {
        match self.eval(term, env)? {
            EvalValue::Bv(bits, actual) => {
                if actual == width {
                    Ok(bits)
                } else {
                    Ok(bits.extract(width - 1, 0))
                }
            }
            other => Err(format!("expected BitVec, got {:?}", other.kind())),
        }
    }

    fn eval(&self, term: &SyGuSTerm, env: &mut EvalEnv) -> Result<EvalValue, String> {
        match term {
            SyGuSTerm::Identifier(Identifier::Symbol(name)) => {
                if let Some(value) = env.get(name) {
                    return Ok(value.clone());
                }
                if let Some(func) = self.funcs.get(name) {
                    if func.args.is_empty() {
                        return self.eval(&func.body, env);
                    }
                }
                Err(format!("unknown identifier {name}"))
            }
            SyGuSTerm::Identifier(other) => Err(format!("unsupported identifier {other}")),
            SyGuSTerm::Literal(Literal::Bool(value)) => Ok(EvalValue::Bool(*value)),
            SyGuSTerm::Literal(Literal::Numeral(value)) => Ok(EvalValue::Nat(*value)),
            SyGuSTerm::Literal(Literal::BinConst(text)) => {
                let bits = Bits::from_bin(text);
                let width = text.strip_prefix("#b").unwrap_or(text).len();
                Ok(EvalValue::Bv(bits, width))
            }
            SyGuSTerm::Application(id, args) => self.eval_app(id, args, env),
            SyGuSTerm::Let(bindings, body) => {
                env.push_scope();
                for binding in bindings {
                    let value = self.eval(&binding.term, env)?;
                    env.insert_local(binding.name.clone(), value);
                }
                let result = self.eval(body, env);
                env.pop_scope();
                result
            }
            SyGuSTerm::Annotated(inner, _) => self.eval(inner, env),
            _ => Err(format!("unsupported SyGuS term in closure eval: {term}")),
        }
    }

    fn eval_app(
        &self,
        id: &Identifier,
        args: &[SyGuSTerm],
        env: &mut EvalEnv,
    ) -> Result<EvalValue, String> {
        let op = id.get_name();
        match op {
            "not" => Ok(EvalValue::Bool(!self.eval_bool(&args[0], env)?)),
            "and" => {
                for arg in args {
                    if !self.eval_bool(arg, env)? {
                        return Ok(EvalValue::Bool(false));
                    }
                }
                Ok(EvalValue::Bool(true))
            }
            "or" => {
                for arg in args {
                    if self.eval_bool(arg, env)? {
                        return Ok(EvalValue::Bool(true));
                    }
                }
                Ok(EvalValue::Bool(false))
            }
            "=" => {
                let lhs = self.eval(&args[0], env)?;
                let rhs = self.eval(&args[1], env)?;
                Ok(EvalValue::Bool(lhs == rhs))
            }
            "ite" => {
                if self.eval_bool(&args[0], env)? {
                    self.eval(&args[1], env)
                } else {
                    self.eval(&args[2], env)
                }
            }
            "bvnot" => {
                let (bits, width) = self.eval_bv_value(&args[0], env)?;
                Ok(EvalValue::Bv(bits.not(width), width))
            }
            "bvand" => {
                let (lhs, width) = self.eval_bv_value(&args[0], env)?;
                let (rhs, _) = self.eval_bv_value(&args[1], env)?;
                Ok(EvalValue::Bv(lhs.and(&rhs, width), width))
            }
            "bvor" => {
                let (lhs, width) = self.eval_bv_value(&args[0], env)?;
                let (rhs, _) = self.eval_bv_value(&args[1], env)?;
                Ok(EvalValue::Bv(lhs.or(&rhs, width), width))
            }
            "concat" => {
                let (high, high_width) = self.eval_bv_value(&args[0], env)?;
                let (low, low_width) = self.eval_bv_value(&args[1], env)?;
                Ok(EvalValue::Bv(
                    Bits::concat(&high, high_width, &low, low_width),
                    high_width + low_width,
                ))
            }
            _ => {
                if let Some((high, low)) = extract_indices(id) {
                    let (bits, _) = self.eval_bv_value(&args[0], env)?;
                    return Ok(EvalValue::Bv(bits.extract(high, low), high - low + 1));
                }
                if let Some(func) = self.funcs.get(op) {
                    if func.args.len() != args.len() {
                        return Err(format!(
                            "function {} expects {} args, got {}",
                            op,
                            func.args.len(),
                            args.len()
                        ));
                    }
                    let arg_values = args
                        .iter()
                        .map(|arg| self.eval(arg, env))
                        .collect::<Result<Vec<_>, _>>()?;
                    env.push_scope();
                    for (name, value) in func.args.iter().cloned().zip(arg_values) {
                        env.insert_local(name, value);
                    }
                    let result = self.eval(&func.body, env);
                    env.pop_scope();
                    return result;
                }
                Err(format!("unsupported SyGuS operator {id}"))
            }
        }
    }

    fn eval_bv_value(&self, term: &SyGuSTerm, env: &mut EvalEnv) -> Result<(Bits, usize), String> {
        match self.eval(term, env)? {
            EvalValue::Bv(bits, width) => Ok((bits, width)),
            other => Err(format!("expected BitVec, got {:?}", other.kind())),
        }
    }
}

fn is_closure_constraint(term: &SyGuSTerm, synth_name: &Option<String>) -> bool {
    let Some(synth_name) = synth_name else {
        return false;
    };
    // (= synth_name aux_name) where aux_name is a synth-fun with constant grammar
    let SyGuSTerm::Application(id, args) = term else {
        return false;
    };
    if args.len() != 2 {
        return false;
    }
    let id_name = match id {
        Identifier::Symbol(name) => name,
        _ => return false,
    };
    if id_name != "=" {
        return false;
    }
    // Check if one side is the synth-fun name and the other is a different identifier
    let (a, b) = (&args[0], &args[1]);
    let a_is_synth = matches!(a, SyGuSTerm::Identifier(Identifier::Symbol(n)) if n == synth_name);
    let b_is_synth = matches!(b, SyGuSTerm::Identifier(Identifier::Symbol(n)) if n == synth_name);
    if !a_is_synth && !b_is_synth {
        return false;
    }
    // The other side must be a different identifier (the aux synth-fun)
    let other = if a_is_synth { b } else { a };
    matches!(other, SyGuSTerm::Identifier(Identifier::Symbol(_)))
}

struct ConstraintVerifier {
    synth_name: String,
    synth_width: usize,
    funcs: HashMap<String, DefineFun>,
    constraints: Vec<SyGuSTerm>,
}

impl ConstraintVerifier {
    fn new(cmds: &[SyGuSCmd]) -> Result<Self, String> {
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
                                body: SyGuSTerm::Literal(Literal::BinConst(
                                    "0".repeat(width),
                                )),
                                width,
                            },
                        );
                    } else if args.is_empty() {
                        // Only non-constant-grammar nullary synth-funs are the
                        // actual synthesis targets.
                        synth = Some((name.clone(), width));
                    }
                }
                SyGuSCmd::Constraint(term) => {
                    // Skip the closure constraint (= synth aux) — it's an
                    // internal implementation detail, not a user constraint.
                    if is_closure_constraint(term, &synth.as_ref().map(|(n, _)| n.clone())) {
                        continue;
                    }
                    constraints.push(term.clone());
                }
                _ => {}
            }
        }
        let (synth_name, synth_width) =
            synth.ok_or_else(|| "missing synth-fun command".to_string())?;
        Ok(Self {
            synth_name,
            synth_width,
            funcs,
            constraints,
        })
    }

    fn failing_constraint_indices(&self, solution: &str) -> Result<Vec<usize>, String> {
        let define_funs = parse_solution_define_funs(solution)?;
        let body = define_funs
            .iter()
            .find_map(|(name, _, body)| (name == &self.synth_name).then_some(body.clone()))
            .or_else(|| define_funs.first().map(|(_, _, body)| body.clone()))
            .ok_or_else(|| "solver response did not contain a define-fun".to_string())?;
        let mut base = HashMap::default();
        let evaluator = self.evaluator();
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
        let requires_self_fixpoint = term_mentions_symbol(&body, &self.synth_name)
            || pending
                .iter()
                .any(|(_, _, def_body)| term_mentions_symbol(def_body, &self.synth_name));
        let requires_self_fixpoint = requires_self_fixpoint
            || self
                .funcs
                .iter()
                .any(|(name, func)| name != &self.synth_name && term_mentions_symbol(&func.body, &self.synth_name));
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
                let next =
                    evaluator.eval_bv(&body, &mut EvalEnv::with_base(iter_base), self.synth_width)?;
                if next == current {
                    converged = Some(next);
                    break;
                }
                current = next;
            }
            converged.ok_or_else(|| "verifier: self-closure candidate did not converge".to_string())?
        } else {
            evaluator.eval_bv(&body, &mut EvalEnv::with_base(base.clone()), self.synth_width)?
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
        let mut env = EvalEnv::with_base(base);
        let mut failing = Vec::new();
        for (idx, constraint) in self.constraints.iter().enumerate() {
            if !self.evaluator().eval_bool(constraint, &mut env)? {
                failing.push(idx);
            }
        }
        Ok(failing)
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

fn extract_indices(id: &Identifier) -> Option<(usize, usize)> {
    let text = id.to_string();
    if let Some(rest) = text
        .strip_prefix("(_ extract ")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let mut parts = rest.split_whitespace();
        let high = parts.next()?.parse().ok()?;
        let low = parts.next()?.parse().ok()?;
        return Some((high, low));
    }
    let Identifier::Indexed(symbol, indices) = id else {
        return None;
    };
    let number_offset = if symbol == "extract" {
        0
    } else if symbol == "_"
        && matches!(indices.first(), Some(Index::Symbol(operator)) if operator == "extract")
    {
        1
    } else {
        return None;
    };
    if indices.len() != number_offset + 2 {
        return None;
    }
    let high = match &indices[number_offset] {
        Index::Numeral(value) => *value,
        Index::Symbol(_) => return None,
    };
    let low = match &indices[number_offset + 1] {
        Index::Numeral(value) => *value,
        Index::Symbol(_) => return None,
    };
    Some((high, low))
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

fn term_mentions_symbol(term: &SyGuSTerm, target: &str) -> bool {
    match term {
        SyGuSTerm::Identifier(id) => id.get_name() == target,
        SyGuSTerm::Application(id, args) => {
            id.get_name() == target || args.iter().any(|arg| term_mentions_symbol(arg, target))
        }
        SyGuSTerm::Annotated(inner, _) => term_mentions_symbol(inner, target),
        SyGuSTerm::Let(bindings, body) => {
            bindings.iter().any(|binding| term_mentions_symbol(&binding.term, target))
                || term_mentions_symbol(body, target)
        }
        SyGuSTerm::Forall(_, body) | SyGuSTerm::Exists(_, body) => term_mentions_symbol(body, target),
        SyGuSTerm::Literal(_) => false,
    }
}
