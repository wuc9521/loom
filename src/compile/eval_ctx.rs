use crate::{
    compile::{
        semantics::Case,
        types::{
            AspectKey, AspectValue, FiniteSpace, SemanticDomain, SemanticKind, SemanticProgram,
            SemanticSpec,
        },
    },
    lang::{
        model::{InterpedValue, Model},
        signature::FuncSig,
    },
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use rustc_hash::{FxHashMap as HashMap, FxHasher};
use std::cell::{Cell, RefCell};
use std::hash::Hasher;
use std::sync::Arc;
use sygus_parser::SExpr as Value;

pub trait EnvView {
    fn aspects(&self) -> &HashMap<String, AspectKey>;
    fn values(&self) -> &HashMap<String, AspectValue>;
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct EvalKey {
    cache_id: u64,
    func: String,
    args: Vec<String>,
}

pub struct SemanticContext<'a> {
    pub models: HashMap<String, &'a Model>,
    pub func_sigs: HashMap<String, FuncSig>,
    spaces: HashMap<String, FiniteSpace>,
    specs: HashMap<String, SemanticSpec>,
    domains_by_relation: HashMap<String, SemanticDomain>,
    domains_by_sort: HashMap<String, Vec<SemanticDomain>>,
    cases: HashMap<String, HashMap<String, Case>>,
    eval_cache: RefCell<HashMap<EvalKey, Option<InterpedValue>>>,
    /// Hash-keyed cache used by `eval_model_value_call`. Mirrors `eval_cache`
    /// but is keyed by a u64 hash of `(model.cache_id, func_bytes, arg
    /// values)` so cache hits don't have to materialise `EvalKey { func:
    /// String, args: Vec<String> }`. On test_col the wrapping bisim walk
    /// hits this cache 100% of the time — the cache key allocations
    /// previously dominated `eval_model_value_call`.
    eval_cache_hashed: RefCell<HashMap<u64, Option<InterpedValue>>>,
    eval_hits: Cell<usize>,
    eval_misses: Cell<usize>,
}

impl<'a> SemanticContext<'a> {
    fn eval_explicit_aspect_field(
        &self,
        arg: &Value,
        sort: &str,
        model: &str,
        env: &impl EnvView,
    ) -> Result<AspectValue, String> {
        let model_ref = self.model(model)?;
        if let Some(symbol) = arg.as_symbol() {
            if sort != "Int"
                && model_ref
                    .univ_for_sort(sort)
                    .is_some_and(|univ| univ.iter().any(|elem| elem == symbol))
            {
                return Ok(AspectValue::Univ {
                    sort: Arc::from(sort),
                    name: Arc::from(symbol),
                });
            }
            if let Some(sig) = self.func_sigs.get(symbol) {
                if sig.arg_sorts.is_empty() {
                    let value = self
                        .eval_model_symbols(model_ref, symbol, &[])
                        .ok_or_else(|| {
                            format!("model {model} has no value for {symbol}")
                        })?;
                    return interped_aspect_value(&sig.ret_sort.to_string(), &value);
                }
            }
        }
        if let Some(items) = arg.to_ref_vec() {
            if let Some(op) = items.first().and_then(|v| v.as_symbol()) {
                if self.func_sigs.contains_key(op) {
                    let sig = &self.func_sigs[op];
                    let arg_values = items[1..]
                        .iter()
                        .zip(sig.arg_sorts.iter())
                        .map(|(a, s)| {
                            self.eval_explicit_aspect_field(a, &s.to_string(), model, env)
                                .map(|v| v.display())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let value = self
                        .eval_model_symbols(model_ref, op, &arg_values)
                        .ok_or_else(|| {
                            format!(
                                "model {model} has no value for {}({})",
                                op,
                                arg_values.join(" ")
                            )
                        })?;
                    return interped_aspect_value(&sig.ret_sort.to_string(), &value);
                }
            }
        }
        self.eval_value_for_sort(arg, sort, env)
    }

    pub fn with_cases(
        problem: &'a DelisProg,
        program: &SemanticProgram,
        cases: &HashMap<String, HashMap<String, Case>>,
    ) -> Result<Self, String> {
        let models = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DelisCmd::DefineModel(model) => Some((model.name.clone(), model)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let func_sigs = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DelisCmd::DeclareFunc(sig) => Some((sig.name.clone(), sig.clone())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let spaces = program
            .finite_spaces
            .iter()
            .map(|space| (space.name.clone(), space.clone()))
            .collect::<HashMap<_, _>>();
        let specs = program
            .semantics
            .iter()
            .map(|spec| (spec.relation.clone(), spec.clone()))
            .collect::<HashMap<_, _>>();
        let domains_by_relation = program
            .domains
            .iter()
            .map(|domain| (domain.relation.clone(), domain.clone()))
            .collect::<HashMap<_, _>>();
        let mut domains_by_sort: HashMap<String, Vec<SemanticDomain>> = HashMap::default();
        for domain in &program.domains {
            domains_by_sort
                .entry(domain.term_sort.clone())
                .or_default()
                .push(domain.clone());
        }
        Ok(Self {
            models,
            func_sigs,
            spaces,
            specs,
            domains_by_relation,
            domains_by_sort,
            cases: cases.clone(),
            eval_cache: RefCell::new(HashMap::default()),
            eval_cache_hashed: RefCell::new(HashMap::default()),
            eval_hits: Cell::new(0),
            eval_misses: Cell::new(0),
        })
    }

    /// Cached wrapper over `Model::eval_symbols`. All semantic and compile
    /// paths that ground a model function should go through this; the cache
    /// is keyed by `(model.cache_id, func, args)`, so calls on the same model
    /// with the same arguments are evaluated at most once per
    /// `SemanticContext`. Returns an owned `InterpedValue` so the cache can
    /// outlive any one borrow.
    pub fn eval_model_symbols(
        &self,
        model: &Model,
        func: &str,
        args: &[String],
    ) -> Option<InterpedValue> {
        let key = EvalKey {
            cache_id: model.cache_id,
            func: func.to_string(),
            args: args.to_vec(),
        };
        if let Some(cached) = self.eval_cache.borrow().get(&key) {
            self.eval_hits.set(self.eval_hits.get() + 1);
            return cached.clone();
        }
        self.eval_misses.set(self.eval_misses.get() + 1);
        let value = model.eval_symbols(func, args).cloned();
        self.eval_cache.borrow_mut().insert(key, value.clone());
        value
    }

    pub fn eval_cache_hits(&self) -> usize {
        self.eval_hits.get()
    }

    pub fn eval_cache_misses(&self) -> usize {
        self.eval_misses.get()
    }

    pub fn cases(&self) -> &HashMap<String, HashMap<String, Case>> {
        &self.cases
    }

    pub fn has_relation(&self, relation: &str) -> bool {
        self.domains_by_relation.contains_key(relation)
    }

    pub fn has_func(&self, func: &str) -> bool {
        self.func_sigs.contains_key(func)
    }

    pub fn domains_for_sort(&self, sort: &str) -> Result<&[SemanticDomain], String> {
        self.domains_by_sort
            .get(sort)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("no semantic domain for term sort {sort}"))
    }

    pub fn domain_for_relation(&self, relation: &str) -> Result<&SemanticDomain, String> {
        self.domains_by_relation
            .get(relation)
            .ok_or_else(|| format!("no semantic domain for relation {relation}"))
    }

    pub fn aspect_space(&self, name: &str) -> Result<&FiniteSpace, String> {
        self.spaces
            .get(name)
            .ok_or_else(|| format!("unknown aspect space {name}"))
    }

    pub fn spec_for_relation(&self, relation: &str) -> Result<&SemanticSpec, String> {
        self.specs
            .get(relation)
            .ok_or_else(|| format!("no semantic spec for relation {relation}"))
    }

    /// Returns the current aspect's model name without cloning.
    ///
    /// The borrowed `&str` is tied to `env`'s lifetime, so callers must not
    /// outlive `env`. Saves one allocation per call — hot enough on the bisim
    /// path to show up at 0.7% self time on test_col before this change.
    pub fn current_model<'e>(&self, env: &'e impl EnvView) -> Result<&'e str, String> {
        env.aspects()
            .get("asp")
            .map(|aspect| aspect.model.as_str())
            .ok_or_else(|| "no current aspect in semantic environment".to_string())
    }

    pub fn model(&self, name: &str) -> Result<&'a Model, String> {
        self.models
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown model {name}"))
    }

    pub fn argument_univs(
        &self,
        sig: &FuncSig,
        model: &Model,
    ) -> Result<Vec<Vec<AspectValue>>, String> {
        sig.arg_sorts
            .iter()
            .map(|sort| {
                let sort = sort.to_string();
                let sort_arc: Arc<str> = Arc::from(sort.as_str());
                model.univ_for_sort(&sort)
                    .ok_or_else(|| format!("model {} has no universe for sort {sort}", model.name))
                    .map(|values| {
                        values
                            .into_iter()
                            .map(|name| AspectValue::Univ {
                                sort: sort_arc.clone(),
                                name: Arc::from(name),
                            })
                            .collect()
                    })
            })
            .collect()
    }

    pub fn parse_univ_expr(
        &self,
        expr: &Value,
        env: &impl EnvView,
    ) -> Result<(String, String), String> {
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("domain expression must be a list: {expr}"))?;
        match items.as_slice() {
            [domain, sort_app] if domain.as_symbol() == Some("univ") => {
                let sort_items = sort_app.to_ref_vec().ok_or_else(|| {
                    format!("domain argument must be a sort application: {sort_app}")
                })?;
                let [sort, model_expr] = sort_items.as_slice() else {
                    return Err(format!(
                        "domain argument must have shape (Sort model): {sort_app}"
                    ));
                };
                let sort = sort
                    .as_symbol()
                    .ok_or_else(|| format!("domain sort must be a symbol: {sort}"))?
                    .to_string();
                let model = self.eval_model_name(model_expr, env)?;
                Ok((model, sort))
            }
            _ => Err(format!("unsupported domain expression: {expr}")),
        }
    }

    pub fn eval_aspect(
        &self,
        expr: &Value,
        relation: &str,
        env: &impl EnvView,
    ) -> Result<AspectKey, String> {
        if let Some(symbol) = expr.as_symbol() {
            let aspect = env
                .aspects()
                .get(symbol)
                .ok_or_else(|| format!("unknown aspect variable {symbol}"))?;
            let mut aspect = aspect.clone();
            aspect.relation = relation.to_string();
            return Ok(aspect);
        }
        let domain = self.domain_for_relation(relation)?;
        let space = self.aspect_space(&domain.aspect_space)?;
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("aspect expression must be a list: {expr}"))?;
        let cons = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("aspect constructor must be a symbol: {expr}"))?;
        if cons != space.name {
            return Err(format!("aspect constructor must be {}", space.name));
        }
        if items.len() - 1 != space.fields.len() {
            return Err(format!(
                "aspect constructor {} provides {} scalar fields, expected {}",
                cons,
                items.len().saturating_sub(2),
                space.fields.len().saturating_sub(1)
            ));
        }
        let mut model = None;
        let mut fields = Vec::new();
        for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
            match &field.domain {
                crate::compile::types::FieldDomain::Sort(sort) if sort == "Model" => {
                    model = Some(self.eval_model_name(arg, env)?);
                }
                crate::compile::types::FieldDomain::Sort(sort) => {
                    let value = if let Some(model_name) = model.as_deref() {
                        self.eval_explicit_aspect_field(arg, sort, model_name, env)?
                    } else {
                        self.eval_value_for_sort(arg, sort, env)?
                    };
                    match &value {
                        AspectValue::Univ { sort: actual, .. } if actual.as_ref() == sort.as_str() => {}
                        AspectValue::Int(_) if sort == "Int" => {}
                        _ => {
                            return Err(format!(
                                "aspect field {} expected sort {}, got {}",
                                field.name,
                                sort,
                                value.display()
                            ));
                        }
                    }
                    fields.push((field.name.clone(), value));
                }
                crate::compile::types::FieldDomain::Enum(values) => {
                    let value = self.eval_value(arg, env)?;
                    if !values.contains(&value.display()) {
                        return Err(format!(
                            "aspect field {} expected one of ({})",
                            field.name,
                            values.join(" ")
                        ));
                    }
                    fields.push((field.name.clone(), value));
                }
                crate::compile::types::FieldDomain::Union { bot, sorts } => {
                    let value = self.eval_value(arg, env)?;
                    match &value {
                        AspectValue::Symbol(symbol) if symbol == bot => {}
                        AspectValue::Univ { sort, .. }
                            if sorts.iter().any(|s| s.as_str() == &**sort) => {}
                        _ => {
                            return Err(format!(
                                "aspect field {} expected {} or one of ({}) but got {}",
                                field.name,
                                bot,
                                sorts.join(" "),
                                value.display()
                            ));
                        }
                    }
                    fields.push((field.name.clone(), value));
                }
            }
        }
        Ok(AspectKey {
            relation: relation.to_string(),
            model: model.ok_or_else(|| format!("aspect {cons} has no model field"))?,
            fields,
        })
    }

    pub fn eval_value(&self, expr: &Value, env: &impl EnvView) -> Result<AspectValue, String> {
        if let Some(value) = expr.as_i64() {
            return self.nonnegative_int_value(value as isize);
        }
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values().get(symbol) {
                return Ok(value.clone());
            }
            if let Ok(model) = self.current_model(env) {
                if let Some(model_ref) = self.models.get(model) {
                    for univ in &model_ref.univ_defs {
                        if univ.elems.iter().any(|elem| elem.name == symbol) {
                            return Ok(AspectValue::Univ {
                                sort: Arc::from(univ.sort.to_string()),
                                name: Arc::from(symbol),
                            });
                        }
                    }
                    if let Some(sig) = self.func_sigs.get(symbol) {
                        if sig.arg_sorts.is_empty() {
                            let value = self
                                .eval_model_symbols(model_ref, symbol, &[])
                                .ok_or_else(|| {
                                    format!("model {model} has no value for {symbol}")
                                })?;
                            return interped_aspect_value(&sig.ret_sort.to_string(), &value);
                        }
                    }
                }
            }
            return Ok(AspectValue::Symbol(symbol.to_string()));
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("value expression must be a symbol or list: {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("value expression operator must be a symbol: {expr}"))?;
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspects().get(aspect_arg) {
                    if op == "model" {
                        return Ok(AspectValue::Model(aspect.model.clone()));
                    }
                    if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                        return Ok(value.clone());
                    }
                }
            }
        }
        if self.func_sigs.contains_key(op) {
            return self.eval_model_value_call(op, &items[1..], env);
        }
        match op {
            "ite" if items.len() == 4 => {
                return if self.eval_bool_value(items[1], env)? {
                    self.eval_value(items[2], env)
                } else {
                    self.eval_value(items[3], env)
                };
            }
            "+" | "-" | "size" => {
                return self.nonnegative_int_value(self.eval_int_expr(expr, env)?);
            }
            _ => {}
        }
        // Hot path: bisim::Refiner::walk probes eval_value and discards Err.
        // Formatting `{expr}` via lexpr Display::fmt was ~15% of bisim time.
        Err(String::new())
    }

    pub fn eval_bool_value(&self, expr: &Value, env: &impl EnvView) -> Result<bool, String> {
        if let Some(value) = expr.as_bool() {
            return Ok(value);
        }
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(String::new()),
            };
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(String::new)?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(String::new)?;
        match op {
            "not" if items.len() == 2 => Ok(!self.eval_bool_value(items[1], env)?),
            "and" => items[1..]
                .iter()
                .try_fold(true, |acc, item| Ok(acc && self.eval_bool_value(item, env)?)),
            "or" => items[1..]
                .iter()
                .try_fold(false, |acc, item| Ok(acc || self.eval_bool_value(item, env)?)),
            "ite" if items.len() == 4 => {
                if self.eval_bool_value(items[1], env)? {
                    self.eval_bool_value(items[2], env)
                } else {
                    self.eval_bool_value(items[3], env)
                }
            }
            "=" if items.len() == 3 => {
                if let (Ok(lhs), Ok(rhs)) =
                    (self.eval_value(items[1], env), self.eval_value(items[2], env))
                {
                    return self.values_equal(&lhs, &rhs, env);
                }
                Ok(self.eval_int_expr(items[1], env)? == self.eval_int_expr(items[2], env)?)
            }
            "<" if items.len() == 3 => {
                Ok(self.eval_int_expr(items[1], env)? < self.eval_int_expr(items[2], env)?)
            }
            "<=" if items.len() == 3 => {
                Ok(self.eval_int_expr(items[1], env)? <= self.eval_int_expr(items[2], env)?)
            }
            "ite" if items.len() == 4 => {
                if self.eval_bool_value(items[1], env)? {
                    self.eval_bool_value(items[2], env)
                } else {
                    self.eval_bool_value(items[3], env)
                }
            }
            _ => Err(String::new()),
        }
    }

    pub fn eval_int_expr(&self, expr: &Value, env: &impl EnvView) -> Result<isize, String> {
        if let Some(value) = expr.as_i64() {
            return Ok(value as isize);
        }
        let Some(items) = expr.to_ref_vec() else {
            return match self.eval_value(expr, env)? {
                AspectValue::Int(value) => Ok(value as isize),
                other => Err(format!("expected Int value, got {}", other.display())),
            };
        };
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("Int expression operator must be a symbol: {expr}"))?;
        match op {
            "+" => items[1..]
                .iter()
                .try_fold(0isize, |acc, item| Ok(acc + self.eval_int_expr(item, env)?)),
            "-" if items.len() >= 2 => {
                let mut values = items[1..].iter();
                let first = values
                    .next()
                    .ok_or_else(|| format!("- expects at least one argument: {expr}"))?;
                let mut total = self.eval_int_expr(first, env)?;
                if items.len() == 2 {
                    total = -total;
                } else {
                    for item in values {
                        total -= self.eval_int_expr(item, env)?;
                    }
                }
                Ok(total)
            }
            "size" if items.len() == 2 => {
                let domain = items[1]
                    .to_ref_vec()
                    .ok_or_else(|| format!("size expects (Sort model), got {}", items[1]))?;
                let [sort_expr, model_expr] = domain.as_slice() else {
                    return Err(format!("size expects (Sort model), got {}", items[1]));
                };
                let sort = sort_expr
                    .as_symbol()
                    .ok_or_else(|| format!("size sort must be a symbol: {sort_expr}"))?;
                let model = self.eval_model_name(model_expr, env)?;
                let model_ref = self.model(&model)?;
                let values = model_ref.univ_for_sort(sort)
                    .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
                Ok(values.len() as isize)
            }
            _ => match self.eval_value(expr, env)? {
                AspectValue::Int(value) => Ok(value as isize),
                other => Err(format!("expected Int value, got {}", other.display())),
            },
        }
    }

    fn nonnegative_int_value(&self, value: isize) -> Result<AspectValue, String> {
        if value < 0 {
            return Err(format!(
                "negative integer aspect value {value} is unsupported"
            ));
        }
        Ok(AspectValue::Int(value as usize))
    }

    pub fn eval_model_value_call(
        &self,
        func: &str,
        args: &[&Value],
        env: &impl EnvView,
    ) -> Result<AspectValue, String> {
        let sig = self
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {func} expects {} arguments, got {}",
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.current_model(env)?;
        let model_ref = self.model(model)?;

        // Resolve args to AspectValues. We need these regardless of cache
        // outcome (they're how we hash the key), but they're typically just
        // a HashMap lookup + clone of a value already in `env.values`, not a
        // recursive eval. The `as_simple_name()` fast path avoids a per-arg
        // `sort.to_string()` allocation for `Sort::Simple` — the common case
        // for declared funcs whose args are over simple sorts like `Pos`.
        let mut arg_values: Vec<AspectValue> = Vec::with_capacity(args.len());
        for (arg, sort) in args.iter().zip(sig.arg_sorts.iter()) {
            let owned_sort_name;
            let sort_str: &str = if let Some(s) = sort.as_simple_name() {
                s
            } else {
                owned_sort_name = sort.to_string();
                &owned_sort_name
            };
            arg_values.push(self.eval_value_for_sort(arg, sort_str, env)?);
        }

        // Hash-keyed cache probe. Sidesteps the `EvalKey { func: String,
        // args: Vec<String> }` allocation that every probe through
        // `eval_model_symbols` requires.
        let mut h = FxHasher::default();
        h.write_u64(model_ref.cache_id);
        h.write(func.as_bytes());
        h.write_u8(0); // separator between func name and args
        for v in &arg_values {
            Self::hash_aspect_value(&mut h, v);
        }
        let key_hash = h.finish();

        let cached = self.eval_cache_hashed.borrow().get(&key_hash).cloned();
        let interped = if let Some(cached) = cached {
            self.eval_hits.set(self.eval_hits.get() + 1);
            cached
        } else {
            self.eval_misses.set(self.eval_misses.get() + 1);
            // Cache miss: materialise the String args for `Model::eval_symbols`
            // (its inner exact-case index is `HashMap<Vec<String>, _>` so we
            // can't avoid this on the miss path).
            let arg_strings: Vec<String> =
                arg_values.iter().map(|v| v.display()).collect();
            let value = model_ref.eval_symbols(func, &arg_strings).cloned();
            self.eval_cache_hashed
                .borrow_mut()
                .insert(key_hash, value.clone());
            value
        };

        let value = interped.ok_or_else(|| {
            // Format the friendly arg list lazily so the alloc only happens
            // on real errors.
            let parts: Vec<String> = arg_values.iter().map(|v| v.display()).collect();
            format!("model {model} has no value for {func}({})", parts.join(" "))
        })?;
        let owned_ret;
        let ret_str: &str = if let Some(s) = sig.ret_sort.as_simple_name() {
            s
        } else {
            owned_ret = sig.ret_sort.to_string();
            &owned_ret
        };
        interped_aspect_value(ret_str, &value)
    }

    /// Hash an AspectValue into a Hasher without going through Display
    /// (which would allocate a String per call).
    fn hash_aspect_value<H: Hasher>(h: &mut H, v: &AspectValue) {
        match v {
            AspectValue::Int(i) => {
                h.write_u8(b'i');
                h.write_u64(*i as u64);
            }
            AspectValue::Univ { sort, name } => {
                h.write_u8(b'u');
                h.write(sort.as_bytes());
                h.write_u8(0);
                h.write(name.as_bytes());
            }
            AspectValue::Symbol(s) => {
                h.write_u8(b's');
                h.write(s.as_bytes());
            }
            AspectValue::Model(m) => {
                h.write_u8(b'm');
                h.write(m.as_bytes());
            }
        }
    }

    pub fn eval_value_for_sort(
        &self,
        expr: &Value,
        sort: &str,
        env: &impl EnvView,
    ) -> Result<AspectValue, String> {
        let value = self.eval_value(expr, env)?;
        if sort == "Int" {
            return Ok(value);
        }
        if let AspectValue::Int(idx) = &value {
            let model = self.current_model(env)?;
            let model_ref = self.model(model)?;
            if let Some(name) = model_ref.univ_element_at(sort, *idx) {
                return Ok(AspectValue::Univ {
                    sort: Arc::from(sort),
                    name: Arc::from(name),
                });
            }
        }
        if let AspectValue::Symbol(symbol) = &value {
            let model = self.current_model(env)?;
            let model_ref = self.model(model)?;
            if model_ref.univ_contains(sort, symbol) {
                return Ok(AspectValue::Univ {
                    sort: Arc::from(sort),
                    name: Arc::from(symbol.as_str()),
                });
            }
        }
        Ok(value)
    }

    pub fn eval_model_name(&self, expr: &Value, env: &impl EnvView) -> Result<String, String> {
        match self.eval_value(expr, env)? {
            AspectValue::Model(model) | AspectValue::Symbol(model) => Ok(model),
            other => Err(format!("expected model value, got {}", other.display())),
        }
    }

    pub fn infer_finite_sort(&self, expr: &Value, env: &impl EnvView) -> Option<String> {
        if let Some(symbol) = expr.as_symbol() {
            if let Some(AspectValue::Univ { sort, .. }) = env.values().get(symbol) {
                return Some(sort.to_string());
            }
            return self
                .func_sigs
                .get(symbol)
                .map(|sig| sig.ret_sort.to_string());
        }
        let items = expr.to_ref_vec()?;
        let op = items.first()?.as_symbol()?;
        if op == "ite" && items.len() == 4 {
            let then_sort = self.infer_finite_sort(items[2], env);
            let else_sort = self.infer_finite_sort(items[3], env);
            return match (then_sort, else_sort) {
                (Some(lhs), Some(rhs)) if lhs == rhs => Some(lhs),
                (Some(sort), None) | (None, Some(sort)) => Some(sort),
                _ => None,
            };
        }
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspects().get(aspect_arg) {
                    if let Some((_, AspectValue::Univ { sort, .. })) =
                        aspect.fields.iter().find(|(name, _)| name == op)
                    {
                        return Some(sort.to_string());
                    }
                }
            }
        }
        if let Some(domain) = self.domains_by_relation.get(op) {
            if let SemanticKind::FiniteValue { sort } = &domain.kind {
                return Some(sort.clone());
            }
        }
        self.func_sigs.get(op).map(|sig| sig.ret_sort.to_string())
    }

    pub fn values_equal(
        &self,
        lhs: &AspectValue,
        rhs: &AspectValue,
        env: &impl EnvView,
    ) -> Result<bool, String> {
        Ok(self.value_matches_target(lhs, rhs, env)? && self.value_matches_target(rhs, lhs, env)?)
    }

    pub fn value_matches_target(
        &self,
        value: &AspectValue,
        target: &AspectValue,
        env: &impl EnvView,
    ) -> Result<bool, String> {
        match (value, target) {
            (AspectValue::Int(idx), AspectValue::Univ { sort, name })
            | (AspectValue::Univ { sort, name }, AspectValue::Int(idx)) => {
                let model = self.current_model(env)?;
                let model_ref = self.model(&model)?;
                let Some(univ) = model_ref.univ_for_sort(sort) else {
                    return Ok(false);
                };
                Ok(univ.get(*idx).is_some_and(|candidate| candidate.as_str() == &**name))
            }
            _ => Ok(value == target),
        }
    }

    pub fn constant_value_matches(
        &self,
        symbol: &str,
        target: &AspectValue,
        env: &impl EnvView,
    ) -> Result<Option<bool>, String> {
        let model = self.current_model(env)?;
        let model_ref = self.model(&model)?;
        Ok(self
            .eval_model_symbols(model_ref, symbol, &[])
            .map(|value| crate::compile::semantics::interped_value_matches(&value, target)))
    }

    pub fn matching_model_call_tuples<F>(
        &self,
        func: &str,
        args: &[&Value],
        env: &impl EnvView,
        mut keep: F,
    ) -> Result<Vec<Vec<AspectValue>>, String>
    where
        F: FnMut(&InterpedValue) -> bool,
    {
        let sig = self
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {func} expects {} arguments, got {}",
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.current_model(env)?;
        let model_ref = self.model(&model)?;
        let univs = self.argument_univs(sig, model_ref)?;
        let mut out = Vec::new();
        for tuple in univs
            .iter()
            .map(|values| values.iter())
            .multi_cartesian_product()
        {
            let arg_names = tuple
                .iter()
                .map(|value| value.display())
                .collect::<Vec<_>>();
            let Some(value) = self.eval_model_symbols(model_ref, func, &arg_names) else {
                continue;
            };
            if keep(&value) {
                out.push(tuple.into_iter().cloned().collect());
            }
        }
        Ok(out)
    }
}

fn interped_aspect_value(sort: &str, value: &InterpedValue) -> Result<AspectValue, String> {
    match value {
        InterpedValue::UnivElem(elem) => Ok(AspectValue::Univ {
            sort: Arc::from(sort),
            name: Arc::from(elem.name.as_str()),
        }),
        InterpedValue::Literal(lit) if sort == "Int" => lit
            .to_string()
            .parse::<usize>()
            .map(AspectValue::Int)
            .map_err(|_| format!("expected Int value, got {lit}")),
        InterpedValue::Literal(lit) => Ok(AspectValue::Symbol(lit.to_string())),
        InterpedValue::FuncApp(..) => {
            Err(format!("nested interpreted value {value} is unsupported"))
        }
    }
}
