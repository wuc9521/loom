use crate::{compile, parser::DelisProg};
use std::collections::{HashMap, HashSet};
use sygus_parser::{GrammarDef, Identifier, Index, Literal, Sort, SyGuSCmd, SyGuSGTerm, SyGuSTerm};

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

struct DefineFun {
    args: Vec<String>,
    body: SyGuSTerm,
    width: usize,
}

pub fn solve(problem: &DelisProg, optimize: bool) -> Result<Option<String>, String> {
    let cmds = compile::to_sygus(problem, optimize)?;
    let mut solver = ClosureSolver::new(problem, cmds)?;
    Ok(Some(solver.solve()?))
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
}

impl<'a> ClosureSolver<'a> {
    fn new(problem: &'a DelisProg, cmds: Vec<SyGuSCmd>) -> Result<Self, String> {
        let mut funcs = HashMap::new();
        let mut synth = None;
        let mut constraints = Vec::new();
        for cmd in cmds {
            match cmd {
                SyGuSCmd::DefineFun(name, args, sort, body) => {
                    let width = bv_width(&sort)?;
                    funcs.insert(
                        name,
                        DefineFun {
                            args: args.into_iter().map(|arg| arg.name).collect(),
                            body,
                            width,
                        },
                    );
                }
                SyGuSCmd::SynthFun(name, _args, sort, grammar) => {
                    let grammar =
                        grammar.ok_or_else(|| "closure requires SyGuS grammar".to_string())?;
                    synth = Some((name, sort, grammar));
                }
                SyGuSCmd::Constraint(term) => constraints.push(term),
                _ => {}
            }
        }
        let (synth_name, synth_sort, grammar) =
            synth.ok_or_else(|| "closure requires synth-fun".to_string())?;
        let (nonterminals, nt_by_symbol, start) = build_nonterminals(grammar)?;
        Ok(Self {
            problem,
            synth_name,
            synth_sort,
            start,
            funcs,
            nonterminals,
            nt_by_symbol,
            constraints,
            processed: HashMap::new(),
        })
    }

    fn solve(&mut self) -> Result<String, String> {
        loop {
            let mut changed = false;
            for nt_idx in 0..self.nonterminals.len() {
                changed |= self.boolean_partition_close(nt_idx)?;
                let prod_count = self.nonterminals[nt_idx].productions.len();
                for prod_idx in 0..prod_count {
                    changed |= self.apply_production(nt_idx, prod_idx)?;
                    if let Some(term) = self.satisfying_start_term()? {
                        return Ok(self.define_fun(term));
                    }
                }
            }
            if let Some(term) = self.satisfying_start_term()? {
                return Ok(self.define_fun(term));
            }
            if !changed {
                return Ok("infeasible".to_string());
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
        let mut env = HashMap::new();
        env.insert(
            self.synth_name.clone(),
            EvalValue::Bv(
                bits.clone(),
                self.nonterminals[self.nt_by_symbol[&self.start]].width,
            ),
        );
        for constraint in &self.constraints {
            if !self.eval_bool(constraint, &env)? {
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
        let mut env = HashMap::new();
        for (name, value) in func.args.iter().zip(args) {
            env.insert(name.clone(), EvalValue::Bv(value.bits.clone(), value.width));
        }
        let bits = self.eval_bv(&func.body, &env, func.width)?;
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
        let mut signatures: HashMap<Vec<bool>, Vec<usize>> = HashMap::new();
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

    fn eval_bool(
        &self,
        term: &SyGuSTerm,
        env: &HashMap<String, EvalValue>,
    ) -> Result<bool, String> {
        match self.eval(term, env)? {
            EvalValue::Bool(value) => Ok(value),
            other => Err(format!("expected Bool, got {:?}", other.kind())),
        }
    }

    fn eval_bv(
        &self,
        term: &SyGuSTerm,
        env: &HashMap<String, EvalValue>,
        width: usize,
    ) -> Result<Bits, String> {
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

    fn eval(
        &self,
        term: &SyGuSTerm,
        env: &HashMap<String, EvalValue>,
    ) -> Result<EvalValue, String> {
        match term {
            SyGuSTerm::Identifier(Identifier::Symbol(name)) => env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown identifier {name}")),
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
                let mut next = env.clone();
                for binding in bindings {
                    next.insert(binding.name.clone(), self.eval(&binding.term, &next)?);
                }
                self.eval(body, &next)
            }
            SyGuSTerm::Annotated(inner, _) => self.eval(inner, env),
            _ => Err(format!("unsupported SyGuS term in closure eval: {term}")),
        }
    }

    fn eval_app(
        &self,
        id: &Identifier,
        args: &[SyGuSTerm],
        env: &HashMap<String, EvalValue>,
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
                Err(format!("unsupported SyGuS operator {id}"))
            }
        }
    }

    fn eval_bv_value(
        &self,
        term: &SyGuSTerm,
        env: &HashMap<String, EvalValue>,
    ) -> Result<(Bits, usize), String> {
        match self.eval(term, env)? {
            EvalValue::Bv(bits, width) => Ok((bits, width)),
            other => Err(format!("expected BitVec, got {:?}", other.kind())),
        }
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

fn build_nonterminals(
    grammar: GrammarDef,
) -> Result<(Vec<Nonterminal>, HashMap<String, usize>, String), String> {
    let mut nonterminals = Vec::new();
    let mut nt_by_symbol = HashMap::new();
    let start = grammar
        .grouped_rule_lists
        .first()
        .map(|group| group.symbol.clone())
        .ok_or_else(|| "empty SyGuS grammar".to_string())?;
    for group in grammar.grouped_rule_lists {
        let width = bv_width(&group.sort)?;
        let mut productions = Vec::new();
        let mut bool_ops = BoolOps::default();
        for term in group.terms {
            let prod = production_from_gterm(term)?;
            classify_bool_op(&mut bool_ops, &prod);
            productions.push(prod);
        }
        let idx = nonterminals.len();
        nt_by_symbol.insert(group.symbol.clone(), idx);
        nonterminals.push(Nonterminal {
            width,
            values: Vec::new(),
            seen: HashSet::new(),
            productions,
            bool_ops,
        });
    }
    Ok((nonterminals, nt_by_symbol, start))
}

fn production_from_gterm(term: SyGuSGTerm) -> Result<Production, String> {
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

fn classify_bool_op(ops: &mut BoolOps, prod: &Production) {
    if !prod.args.is_empty() {
        if prod.args.len() == 1 && prod.ctor.ends_with(".not") {
            ops.not_ctor = Some(prod.ctor.clone());
        } else if prod.args.len() == 2 && prod.ctor.ends_with(".and") {
            ops.and_ctor = Some(prod.ctor.clone());
        } else if prod.args.len() == 2 && prod.ctor.ends_with(".or") {
            ops.or_ctor = Some(prod.ctor.clone());
        }
        return;
    }
    if prod.ctor.ends_with(".true") {
        ops.true_ctor = Some(prod.ctor.clone());
    } else if prod.ctor.ends_with(".false") {
        ops.false_ctor = Some(prod.ctor.clone());
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

fn bv_width(sort: &Sort) -> Result<usize, String> {
    let text = sort.to_string();
    text.strip_prefix("(_ BitVec ")
        .and_then(|rest| rest.strip_suffix(')'))
        .and_then(|width| width.parse::<usize>().ok())
        .ok_or_else(|| format!("expected BitVec sort, got {sort}"))
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
