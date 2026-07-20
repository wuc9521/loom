use crate::fxmap::HashMap;
use sygus_parser::{Identifier, Index, Literal, SyGuSTerm};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bits(pub Vec<u64>);

impl Bits {
    pub fn zero(width: usize) -> Self {
        Self(vec![0; width.div_ceil(64)])
    }

    pub fn from_lsb(values: &[bool]) -> Self {
        let mut bits = Self::zero(values.len());
        for (idx, value) in values.iter().enumerate() {
            if *value {
                bits.set(idx);
            }
        }
        bits
    }

    pub fn from_bin(text: &str) -> Self {
        let text = text.strip_prefix("#b").unwrap_or(text);
        Self::from_lsb(&text.chars().rev().map(|ch| ch == '1').collect::<Vec<_>>())
    }

    pub fn get(&self, idx: usize) -> bool {
        (self.0[idx / 64] & (1u64 << (idx % 64))) != 0
    }

    pub fn set(&mut self, idx: usize) {
        self.0[idx / 64] |= 1u64 << (idx % 64);
    }

    pub fn not(&self, width: usize) -> Self {
        let mut out = Self(self.0.iter().map(|word| !word).collect());
        out.mask(width);
        out
    }

    pub fn and(&self, other: &Self, width: usize) -> Self {
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

    pub fn or(&self, other: &Self, width: usize) -> Self {
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

    pub fn concat(high: &Self, high_width: usize, low: &Self, low_width: usize) -> Self {
        let mut values = Vec::with_capacity(high_width + low_width);
        for idx in 0..low_width {
            values.push(low.get(idx));
        }
        for idx in 0..high_width {
            values.push(high.get(idx));
        }
        Self::from_lsb(&values)
    }

    pub fn extract(&self, high: usize, low: usize) -> Self {
        Self::from_lsb(&(low..=high).map(|idx| self.get(idx)).collect::<Vec<_>>())
    }

    pub fn mask(&mut self, width: usize) {
        let rem = width % 64;
        if rem != 0 {
            if let Some(last) = self.0.last_mut() {
                *last &= (1u64 << rem) - 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct DefineFun {
    pub args: Vec<String>,
    pub body: SyGuSTerm,
    pub width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalValue {
    Bool(bool),
    Nat(usize),
    Bv(Bits, usize),
}

impl EvalValue {
    pub fn kind(&self) -> &'static str {
        match self {
            EvalValue::Bool(_) => "Bool",
            EvalValue::Nat(_) => "Nat",
            EvalValue::Bv(_, _) => "BitVec",
        }
    }
}

#[derive(Default)]
pub struct EvalEnv {
    base: HashMap<String, EvalValue>,
    scoped: HashMap<String, Vec<EvalValue>>,
    scopes: Vec<Vec<String>>,
}

impl EvalEnv {
    pub fn with_base(base: HashMap<String, EvalValue>) -> Self {
        Self {
            base,
            scoped: HashMap::default(),
            scopes: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&EvalValue> {
        self.scoped
            .get(name)
            .and_then(|values| values.last())
            .or_else(|| self.base.get(name))
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub fn insert_local(&mut self, name: String, value: EvalValue) {
        self.scoped.entry(name.clone()).or_default().push(value);
        self.scopes
            .last_mut()
            .expect("scope must exist before inserting locals")
            .push(name);
    }

    pub fn pop_scope(&mut self) {
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

pub struct EvalContext<'a> {
    pub funcs: &'a HashMap<String, DefineFun>,
}

impl<'a> EvalContext<'a> {
    #[inline]
    pub fn eval_bool(&self, term: &SyGuSTerm, env: &mut EvalEnv) -> Result<bool, String> {
        match self.eval(term, env)? {
            EvalValue::Bool(value) => Ok(value),
            other => Err(format!("expected Bool, got {:?}", other.kind())),
        }
    }

    #[inline]
    pub fn eval_bv(
        &self,
        term: &SyGuSTerm,
        env: &mut EvalEnv,
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

    #[inline]
    pub fn eval(&self, term: &SyGuSTerm, env: &mut EvalEnv) -> Result<EvalValue, String> {
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

    #[inline]
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

pub fn extract_indices(id: &Identifier) -> Option<(usize, usize)> {
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

pub fn is_closure_constraint(term: &SyGuSTerm, synth_name: &Option<String>) -> bool {
    let Some(synth_name) = synth_name else {
        return false;
    };
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
    let (a, b) = (&args[0], &args[1]);
    let a_is_synth = matches!(a, SyGuSTerm::Identifier(Identifier::Symbol(n)) if n == synth_name);
    let b_is_synth = matches!(b, SyGuSTerm::Identifier(Identifier::Symbol(n)) if n == synth_name);
    if !a_is_synth && !b_is_synth {
        return false;
    }
    let other = if a_is_synth { b } else { a };
    matches!(other, SyGuSTerm::Identifier(Identifier::Symbol(_)))
}

pub fn term_mentions_symbol(term: &SyGuSTerm, target: &str) -> bool {
    match term {
        SyGuSTerm::Identifier(id) => id.get_name() == target,
        SyGuSTerm::Application(id, args) => {
            id.get_name() == target || args.iter().any(|arg| term_mentions_symbol(arg, target))
        }
        SyGuSTerm::Annotated(inner, _) => term_mentions_symbol(inner, target),
        SyGuSTerm::Let(bindings, body) => {
            bindings
                .iter()
                .any(|binding| term_mentions_symbol(&binding.term, target))
                || term_mentions_symbol(body, target)
        }
        SyGuSTerm::Forall(_, body) | SyGuSTerm::Exists(_, body) => {
            term_mentions_symbol(body, target)
        }
        SyGuSTerm::Literal(_) => false,
    }
}
