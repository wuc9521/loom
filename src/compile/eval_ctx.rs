use crate::fxmap::{FxHasher, HashMap};
use crate::{
    compile::{
        semantics::{interped_bool, Case},
        types::{
            AspectKey, AspectSpace, AspectValue, SemanticDomain, SemanticKind, SemanticProgram,
            SemanticSpec,
        },
    },
    lang::{
        model::{InterpedValue, Model},
        signature::FuncSig,
    },
    parser::{LoomCmd, LoomProg},
};
use itertools::Itertools;
use std::cell::{Cell, RefCell};
use std::hash::Hasher;
use std::sync::Arc;
use sygus_parser::SExpr as Value;

pub trait EnvView {
    fn aspects(&self) -> &HashMap<String, AspectKey>;
    fn values(&self) -> &HashMap<String, AspectValue>;
    /// Explicit current model context, if one has been established (e.g. by a
    /// surrounding `(with-model M …)`).  Implementations without such a context
    /// should return `None`; callers fall back to the model stored in the first
    /// aspect binding for backward compatibility with old-style aspects.
    fn current_model(&self) -> Option<&str> {
        None
    }
}

/// Borrowed-name view of a `Sort`: avoids a Display allocation for the common
/// `Sort::Simple` case on hot eval paths.
fn simple_sort_name(sort: &sygus_parser::Sort) -> std::borrow::Cow<'_, str> {
    match sort.as_simple_name() {
        Some(name) => std::borrow::Cow::Borrowed(name),
        None => std::borrow::Cow::Owned(sort.to_string()),
    }
}

struct ConstraintModelEnv {
    current_model: String,
}

impl ConstraintModelEnv {
    fn new(model: &str) -> Self {
        Self {
            current_model: model.to_string(),
        }
    }
}

impl EnvView for ConstraintModelEnv {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        static EMPTY: std::sync::LazyLock<HashMap<String, AspectKey>> =
            std::sync::LazyLock::new(HashMap::default);
        &EMPTY
    }

    fn values(&self) -> &HashMap<String, AspectValue> {
        static EMPTY: std::sync::LazyLock<HashMap<String, AspectValue>> =
            std::sync::LazyLock::new(HashMap::default);
        &EMPTY
    }

    fn current_model(&self) -> Option<&str> {
        Some(&self.current_model)
    }
}

pub struct SemanticContext<'a> {
    pub models: HashMap<String, &'a Model>,
    pub func_sigs: HashMap<String, FuncSig>,
    spaces: HashMap<String, AspectSpace>,
    /// All values declared across `(enum ...)` aspect-space field domains.
    /// Used to recognize bare enum members appearing in semantic bodies
    /// structurally — without ever consulting model element names.
    enum_values: std::collections::HashSet<String>,
    specs: HashMap<String, SemanticSpec>,
    domains_by_relation: HashMap<String, SemanticDomain>,
    domains_by_sort: HashMap<String, Vec<SemanticDomain>>,
    cases: HashMap<String, HashMap<String, Case>>,
    eval_cache: RefCell<HashMap<u64, Option<InterpedValue>>>,
    /// Hash-keyed cache for `eval_model_bool_call`. Mirrors `eval_cache_hashed`
    /// but stores the boolean result: keyed by a u64 hash of `(model.cache_id,
    /// func bytes, arg values)` so a probe never has to materialise a
    /// `BoolEvalKey { func: String, args: Vec<AspectValue> }`, which dominated
    /// this call on model-predicate-heavy bisim signatures.
    eval_bool_cache: RefCell<HashMap<u64, bool>>,
    /// Hash-keyed cache used by `eval_model_value_call`. Mirrors `eval_cache`
    /// but is keyed by a u64 hash of `(model.cache_id, func_bytes, arg
    /// values)` so cache hits don't have to materialise `EvalKey { func:
    /// String, args: Vec<String> }`. On test_col the wrapping bisim walk
    /// hits this cache 100% of the time — the cache key allocations
    /// previously dominated `eval_model_value_call`.
    ///
    /// Stores the already-interpreted `AspectValue` instead of the raw
    /// `InterpedValue` so cache hits skip `interped_aspect_value`'s
    /// per-call allocations (two `Arc<str>` for Univ, one `String` for
    /// Symbol). `None` represents a model call that returned no value.
    eval_cache_hashed: RefCell<HashMap<u64, Option<AspectValue>>>,
    /// Per-(model.cache_id, sort) universe as ready-built `AspectValue`s.
    /// `eval_value_for_sort`'s Int/Symbol conversions allocated 3 heap
    /// objects per call (element String + two `Arc::from`) and
    /// `argument_univs` cloned a whole `Vec<String>` per call. Universes are
    /// static for the run (same invariant as `lang/eval.rs`'s thread-local
    /// `UNIV_CACHE`), so building once makes every conversion an Arc clone
    /// or element clone — zero allocations.
    univ_cache: RefCell<HashMap<u64, HashMap<String, Arc<Vec<AspectValue>>>>>,
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
            if sort != "Int" {
                if let Some((univ, i)) = self.univ_index_of_symbol(model_ref, sort, symbol) {
                    return Ok(univ[i].clone());
                }
            }
            if let Some(sig) = self.func_sigs.get(symbol) {
                if sig.arg_sorts.is_empty() {
                    let value = self
                        .eval_model_symbols(model_ref, symbol, &[])
                        .ok_or_else(|| format!("model {model} has no value for {symbol}"))?;
                    return interped_aspect_value(&simple_sort_name(&sig.ret_sort), &value);
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
                    return interped_aspect_value(&simple_sort_name(&sig.ret_sort), &value);
                }
            }
        }
        self.eval_value_for_sort(arg, sort, env)
    }

    pub fn with_cases(
        problem: &'a LoomProg,
        program: &SemanticProgram,
        cases: &HashMap<String, HashMap<String, Case>>,
    ) -> Result<Self, String> {
        let models = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                LoomCmd::DefineModel(model) => Some((model.name.clone(), model)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let func_sigs = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                LoomCmd::DeclareFunc(sig) => Some((sig.name.clone(), sig.clone())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let spaces = program
            .aspect_spaces
            .iter()
            .map(|space| (space.name.clone(), space.clone()))
            .collect::<HashMap<_, _>>();
        let mut enum_values = std::collections::HashSet::new();
        for space in &program.aspect_spaces {
            for field in &space.fields {
                if let crate::compile::types::FieldDomain::Enum(values) = &field.domain {
                    enum_values.extend(values.iter().cloned());
                }
            }
        }
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
            enum_values,
            specs,
            domains_by_relation,
            domains_by_sort,
            cases: cases.clone(),
            eval_cache: RefCell::new(HashMap::default()),
            eval_bool_cache: RefCell::new(HashMap::default()),
            eval_cache_hashed: RefCell::new(HashMap::default()),
            univ_cache: RefCell::new(HashMap::default()),
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
        // Hash-keyed probe (same pattern as `eval_bool_cache` below): the
        // owned `EvalKey { func: String, args: Vec<String> }` put 2+ heap
        // allocations on every probe of a >99%-hit cache — the single hottest
        // primitive of CTL compile (10M+ probes per instance).
        let mut h = FxHasher::default();
        h.write_u64(model.cache_id);
        h.write(func.as_bytes());
        h.write_u8(0);
        for arg in args {
            h.write(arg.as_bytes());
            h.write_u8(0);
        }
        let key = h.finish();
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

    pub fn aspect_space(&self, name: &str) -> Result<&AspectSpace, String> {
        self.spaces
            .get(name)
            .ok_or_else(|| format!("unknown aspect space {name}"))
    }

    pub fn spec_for_relation(&self, relation: &str) -> Result<&SemanticSpec, String> {
        self.specs
            .get(relation)
            .ok_or_else(|| format!("no semantic spec for relation {relation}"))
    }

    /// Returns the current model name without cloning.
    ///
    /// Prefer an explicit model context on `env` (set by `(with-model M …)`),
    /// falling back to the model stored in the first aspect binding for
    /// backward compatibility with old-style aspects that carry a `(model
    /// Model)` field.  The borrowed `&str` is tied to `env`'s lifetime, so
    /// callers must not outlive `env`.
    pub fn current_model<'e>(&self, env: &'e impl EnvView) -> Result<&'e str, String> {
        env.current_model()
            .or_else(|| {
                env.aspects()
                    .values()
                    .next()
                    .map(|aspect| aspect.model.as_str())
            })
            .ok_or_else(|| "no current model in semantic environment".to_string())
    }

    pub fn model(&self, name: &str) -> Result<&'a Model, String> {
        self.models
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown model {name}"))
    }

    /// Shared `AspectValue` universe for (model, sort) — built once, then
    /// one Arc clone per call. The single source of universe elements for
    /// the compile layer (bisim's `expand_range` ranges over the same Arcs).
    pub(crate) fn univ_values(
        &self,
        model: &Model,
        sort: &str,
    ) -> Option<Arc<Vec<AspectValue>>> {
        if let Some(cached) = self
            .univ_cache
            .borrow()
            .get(&model.cache_id)
            .and_then(|by_sort| by_sort.get(sort))
        {
            return Some(cached.clone());
        }
        let names = model.univ_for_sort_arc(sort)?;
        let sort_arc: Arc<str> = Arc::from(sort);
        let values: Arc<Vec<AspectValue>> = Arc::new(
            names
                .iter()
                .map(|name| AspectValue::Univ {
                    sort: sort_arc.clone(),
                    name: Arc::from(name.as_str()),
                })
                .collect(),
        );
        self.univ_cache
            .borrow_mut()
            .entry(model.cache_id)
            .or_default()
            .insert(sort.to_string(), values.clone());
        Some(values)
    }

    pub fn argument_univs(
        &self,
        sig: &FuncSig,
        model: &Model,
    ) -> Result<Vec<Arc<Vec<AspectValue>>>, String> {
        sig.arg_sorts
            .iter()
            .map(|sort| {
                let sort = simple_sort_name(sort);
                self.univ_values(model, &sort)
                    .ok_or_else(|| format!("model {} has no universe for sort {sort}", model.name))
            })
            .collect()
    }

    pub fn parse_univ_expr(
        &self,
        expr: &Value,
        env: &impl EnvView,
    ) -> Result<(String, String), String> {
        if let Some(sort) = expr.as_symbol() {
            let model = self.current_model(env)?;
            return Ok((model.to_string(), sort.to_string()));
        }
        Err(format!("unsupported domain expression: {expr}"))
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
                "aspect constructor {} provides {} fields, expected {}",
                cons,
                items.len().saturating_sub(1),
                space.fields.len()
            ));
        }
        let has_model_field = space
            .fields
            .iter()
            .any(|field| field.domain.as_sort() == Some("Model"));
        let model_name = if has_model_field {
            let mut model = None;
            for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
                if field.domain.as_sort() == Some("Model") {
                    model = Some(self.eval_model_name(arg, env)?);
                }
            }
            model.ok_or_else(|| format!("aspect {cons} has no model argument"))?
        } else {
            self.current_model(env)?.to_string()
        };
        let mut fields = Vec::new();
        for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
            match &field.domain {
                crate::compile::types::FieldDomain::Sort(sort) if sort == "Model" => {}
                crate::compile::types::FieldDomain::Sort(sort) => {
                    let value = self
                        .eval_explicit_aspect_field(arg, sort, &model_name, env)
                        .unwrap_or_else(|_| AspectValue::Symbol(arg.to_string()));
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
            }
        }
        Ok(AspectKey {
            relation: relation.to_string(),
            model: model_name,
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
            // Disciplined path: a bare symbol denoting a domain value must be a
            // declared nullary function, interpreted by the model. Its
            // denotation flows through the structure. We never consult model
            // element names — those are arbitrary model-internal tokens and a
            // semantic body may mention only signature-level symbols.
            if let Some(sig) = self.func_sigs.get(symbol) {
                if sig.arg_sorts.is_empty() {
                    let model = self.current_model(env)?;
                    let model_ref = self
                        .models
                        .get(model)
                        .ok_or_else(|| format!("unknown model {model}"))?;
                    let value = self
                        .eval_model_symbols(model_ref, symbol, &[])
                        .ok_or_else(|| format!("model {model} has no value for {symbol}"))?;
                    return interped_aspect_value(&simple_sort_name(&sig.ret_sort), &value);
                }
            }
            // Enum members and model names are signature-level tokens that
            // appear as aspect arguments / enum comparisons; they carry through
            // as Symbol.
            if self.enum_values.contains(symbol) || self.models.contains_key(symbol) {
                return Ok(AspectValue::Symbol(symbol.to_string()));
            }
            return Err(format!(
                "undeclared symbol {symbol} in semantic body; expected a bound variable, \
                 a declared function, an enum value, or a model name (domain elements may \
                 not be named directly — declare and interpret a nullary constant instead)"
            ));
        }
        let items = expr
            .as_slice()
            .ok_or_else(|| format!("value expression must be a symbol or list: {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("value expression operator must be a symbol: {expr}"))?;
        // `(get-fun FUNC MODEL)` reads FUNC^MODEL — the interpretation of a
        // signature function in an EXPLICITLY named structure. The model is
        // explicit (not from `current_model`), which is what lets constraints
        // and term-synthesis checks resolve example data without an ambient
        // aspect. 0-ary form returns the element directly.
        if op == "get-fun" {
            if items.len() < 3 {
                return Err(format!(
                    "get-fun expects at least a function and a model: {expr}"
                ));
            }
            let func_item = &items[1];
            let model_item = &items[2];
            let func = func_item
                .as_symbol()
                .ok_or_else(|| format!("get-fun function must be a symbol: {func_item}"))?;
            let model = model_item
                .as_symbol()
                .ok_or_else(|| format!("get-fun model must be a symbol: {model_item}"))?;
            let sig = self
                .func_sigs
                .get(func)
                .ok_or_else(|| format!("get-fun: unknown function {func}"))?;
            let model_ref = self
                .models
                .get(model)
                .ok_or_else(|| format!("get-fun: unknown model {model}"))?;
            let extra_args = &items[3..];
            if extra_args.len() != sig.arg_sorts.len() {
                return Err(format!(
                    "get-fun: {func} takes {} arguments, got {}",
                    sig.arg_sorts.len(),
                    extra_args.len()
                ));
            }
            // Build a temporary env seeded with this model for arg evaluation.
            let model_env = ConstraintModelEnv::new(model);
            let arg_strs: Vec<String> = extra_args
                .iter()
                .zip(sig.arg_sorts.iter())
                .map(|(arg, sort)| {
                    let val = self.eval_value_for_sort(arg, &sort.to_string(), &model_env)?;
                    Ok(val.display())
                })
                .collect::<Result<_, String>>()?;
            let value = self
                .eval_model_symbols(model_ref, func, &arg_strs)
                .ok_or_else(|| {
                    format!(
                        "model {model} has no value for {func}({})",
                        arg_strs.join(", ")
                    )
                })?;
            return interped_aspect_value(&simple_sort_name(&sig.ret_sort), &value);
        }
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
        if let Some(sig) = self.func_sigs.get(op) {
            return self.eval_model_value_call(sig, &items[1..], env);
        }
        match op {
            "ite" if items.len() == 4 => {
                return if self.eval_bool_value(&items[1], env)? {
                    self.eval_value(&items[2], env)
                } else {
                    self.eval_value(&items[3], env)
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
        // `as_slice` not `to_ref_vec`: this is the bisim hot path (called
        // ~12M+ times per quotient, once per quantifier body per range value,
        // plus recursion), and `to_ref_vec` allocates a `Vec<&Value>` each
        // time. Only the private `eval_model_bool_call` helper shares the
        // `&[Value]` shape; `eval_value`/`eval_int_expr` keep their own
        // signatures and are handed `&items[k]`.
        let items = expr.as_slice().ok_or_else(String::new)?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(String::new)?;
        // First-byte dispatch: the literal match ran up to 7 string compares
        // per call on bisim's hottest probe (~25M calls per quotient). Guard
        // failure falls through to the default arm exactly as before.
        match op.as_bytes().first().copied() {
            Some(b'n') if op == "not" && items.len() == 2 => {
                Ok(!self.eval_bool_value(&items[1], env)?)
            }
            Some(b'a') if op == "and" => items[1..].iter().try_fold(true, |acc, item| {
                Ok(acc && self.eval_bool_value(item, env)?)
            }),
            Some(b'o') if op == "or" => items[1..].iter().try_fold(false, |acc, item| {
                Ok(acc || self.eval_bool_value(item, env)?)
            }),
            Some(b'i') if op == "ite" && items.len() == 4 => {
                if self.eval_bool_value(&items[1], env)? {
                    self.eval_bool_value(&items[2], env)
                } else {
                    self.eval_bool_value(&items[3], env)
                }
            }
            Some(b'=') if op == "=" && items.len() == 3 => {
                if let (Ok(lhs), Ok(rhs)) = (
                    self.eval_value(&items[1], env),
                    self.eval_value(&items[2], env),
                ) {
                    return self.values_equal(&lhs, &rhs, env);
                }
                Ok(self.eval_int_expr(&items[1], env)? == self.eval_int_expr(&items[2], env)?)
            }
            Some(b'<') if op == "<" && items.len() == 3 => {
                Ok(self.eval_int_expr(&items[1], env)? < self.eval_int_expr(&items[2], env)?)
            }
            Some(b'<') if op == "<=" && items.len() == 3 => {
                Ok(self.eval_int_expr(&items[1], env)? <= self.eval_int_expr(&items[2], env)?)
            }
            _ => {
                // A concrete test on the model: a declared model function
                // returning Bool applied to ground arguments (e.g. `leq`,
                // `succ`). The paper's δ evaluates such tests immediately, so
                // they must be recognized as constants here too — otherwise a
                // model predicate over aspect fields is treated as dynamic and
                // its truth pattern leaks absolute positions into the bisim
                // signature. Non-model ops and non-ground args stay dynamic
                // (Err), exactly as before.
                let sig = self.func_sigs.get(op).ok_or_else(String::new)?;
                if sig.ret_sort.as_simple_name() != Some("Bool")
                    || sig.arg_sorts.len() != items.len() - 1
                {
                    return Err(String::new());
                }
                self.eval_model_bool_call(sig, &items[1..], env)
            }
        }
    }

    /// Shared key for both model-call caches: FxHash of (model cache_id,
    /// func name, converted args). `hash_value_for_sort` writes exactly the
    /// bytes `hash_aspect_value(&eval_value_for_sort(..))` would, so keys
    /// are unchanged from the pre-streaming code.
    fn model_call_key(
        &self,
        model_ref: &Model,
        sig: &FuncSig,
        args: &[Value],
        env: &impl EnvView,
    ) -> Result<u64, String> {
        let mut h = FxHasher::default();
        h.write_u64(model_ref.cache_id);
        h.write(sig.name.as_bytes());
        h.write_u8(0); // separator between func name and args
        for (arg, sort) in args.iter().zip(sig.arg_sorts.iter()) {
            self.hash_value_for_sort(&mut h, arg, &simple_sort_name(sort), env)?;
        }
        Ok(h.finish())
    }

    /// Owned arg names for `Model::eval_symbols` (its exact-case index is a
    /// `HashMap<Vec<String>, _>`). Miss/error paths only — the hit path
    /// streams through `model_call_key`.
    fn model_call_args(
        &self,
        sig: &FuncSig,
        args: &[Value],
        env: &impl EnvView,
    ) -> Result<Vec<String>, String> {
        args.iter()
            .zip(sig.arg_sorts.iter())
            .map(|(arg, sort)| {
                self.eval_value_for_sort(arg, &simple_sort_name(sort), env)
                    .map(|v| v.display())
            })
            .collect()
    }

    /// `sig` is the caller's `func_sigs` lookup — this is only reached from
    /// `eval_bool_value`, which already resolved and Bool/arity-checked it.
    fn eval_model_bool_call(
        &self,
        sig: &FuncSig,
        args: &[Value],
        env: &impl EnvView,
    ) -> Result<bool, String> {
        let model_ref = self.model(self.current_model(env)?)?;
        let key_hash = self.model_call_key(model_ref, sig, args, env)?;
        if let Some(&cached) = self.eval_bool_cache.borrow().get(&key_hash) {
            self.eval_hits.set(self.eval_hits.get() + 1);
            return Ok(cached);
        }
        self.eval_misses.set(self.eval_misses.get() + 1);
        // ponytail: args re-evaluated on the miss path (~0.1% of calls) —
        // cheaper than materializing an owned Vec on every hit.
        let arg_names = self.model_call_args(sig, args, env)?;
        let value = model_ref
            .eval_symbols(&sig.name, &arg_names)
            .is_some_and(interped_bool);
        self.eval_bool_cache.borrow_mut().insert(key_hash, value);
        Ok(value)
    }

    pub fn eval_int_expr(&self, expr: &Value, env: &impl EnvView) -> Result<isize, String> {
        if let Some(value) = expr.as_i64() {
            return Ok(value as isize);
        }
        let Some(items) = expr.as_slice() else {
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
                let Some([sort_expr, model_expr]) = items[1].as_slice() else {
                    return Err(format!("size expects (Sort model), got {}", items[1]));
                };
                let sort = sort_expr
                    .as_symbol()
                    .ok_or_else(|| format!("size sort must be a symbol: {sort_expr}"))?;
                let model = self.eval_model_name(model_expr, env)?;
                let model_ref = self.model(&model)?;
                let values = model_ref
                    .univ_for_sort_arc(sort)
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

    /// `sig` is the caller's `func_sigs` lookup — the only call site
    /// (`eval_value`) resolved it by `op`, and `func_sigs` is keyed by
    /// `sig.name`, so hashing `sig.name` bytes matches the old `func` bytes.
    pub fn eval_model_value_call(
        &self,
        sig: &FuncSig,
        args: &[Value],
        env: &impl EnvView,
    ) -> Result<AspectValue, String> {
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {} expects {} arguments, got {}",
                sig.name,
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.current_model(env)?;
        let model_ref = self.model(model)?;
        let key_hash = self.model_call_key(model_ref, sig, args, env)?;
        let cached = self.eval_cache_hashed.borrow().get(&key_hash).cloned();
        let aspect_value = if let Some(cached) = cached {
            self.eval_hits.set(self.eval_hits.get() + 1);
            cached
        } else {
            self.eval_misses.set(self.eval_misses.get() + 1);
            // ponytail: args re-evaluated on the miss path (~0.1% of calls).
            // The InterpedValue is converted here so the cache stores the
            // interpreted form — hits skip `interped_aspect_value`.
            let arg_strings = self.model_call_args(sig, args, env)?;
            let raw = model_ref.eval_symbols(&sig.name, &arg_strings).cloned();
            let converted = match raw {
                Some(v) => Some(interped_aspect_value(
                    &simple_sort_name(&sig.ret_sort),
                    &v,
                )?),
                None => None,
            };
            self.eval_cache_hashed
                .borrow_mut()
                .insert(key_hash, converted.clone());
            converted
        };

        aspect_value.ok_or_else(|| {
            // ponytail: best-effort re-eval for the friendly arg list; only
            // runs on real errors, and error text is not part of the output.
            let parts = self.model_call_args(sig, args, env).unwrap_or_default();
            format!(
                "model {model} has no value for {}({})",
                sig.name,
                parts.join(" ")
            )
        })
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
        self.with_value_for_sort(expr, sort, env, |v| v.clone())
    }

    /// Hashes exactly what `eval_value_for_sort(expr, sort)` would return —
    /// identical bytes, so cache keys are unchanged — without materializing
    /// the owned conversion (an entry clone + drop per arg per call on the
    /// 12.7M-call bisim path).
    fn hash_value_for_sort<H: Hasher>(
        &self,
        h: &mut H,
        expr: &Value,
        sort: &str,
        env: &impl EnvView,
    ) -> Result<(), String> {
        self.with_value_for_sort(expr, sort, env, |v| Self::hash_aspect_value(h, v))
    }

    /// The one conversion site: evaluates `expr`, then hands `f` the value
    /// the sort denotes — the borrowed `univ_cache` entry when an Int index
    /// or Symbol converts under `sort`, else the evaluated value itself.
    fn with_value_for_sort<R>(
        &self,
        expr: &Value,
        sort: &str,
        env: &impl EnvView,
        f: impl FnOnce(&AspectValue) -> R,
    ) -> Result<R, String> {
        let value = self.eval_value(expr, env)?;
        if sort != "Int" {
            if let Some((univ, i)) = self.univ_index_of(&value, sort, env)? {
                return Ok(f(&univ[i]));
            }
        }
        Ok(f(&value))
    }

    /// The `univ_cache` entry (Arc + index) that `value` converts to under
    /// `sort`, if any. The index lets callers borrow the entry in place; the
    /// Arc escape keeps the borrow alive outside the RefCell guard. Model
    /// resolution is lazy: values that can't convert never touch the model.
    fn univ_index_of(
        &self,
        value: &AspectValue,
        sort: &str,
        env: &impl EnvView,
    ) -> Result<Option<(Arc<Vec<AspectValue>>, usize)>, String> {
        match value {
            AspectValue::Int(idx) => {
                let model_ref = self.model(self.current_model(env)?)?;
                let Some(univ) = self.univ_values(model_ref, sort) else {
                    return Ok(None);
                };
                Ok((*idx < univ.len()).then_some(*idx).map(|i| (univ, i)))
            }
            AspectValue::Symbol(symbol) => {
                let model_ref = self.model(self.current_model(env)?)?;
                Ok(self.univ_index_of_symbol(model_ref, sort, symbol))
            }
            _ => Ok(None),
        }
    }

    /// The universe entry whose element name is `symbol` — the single
    /// membership+index lookup behind every Symbol → Univ conversion.
    fn univ_index_of_symbol(
        &self,
        model_ref: &Model,
        sort: &str,
        symbol: &str,
    ) -> Option<(Arc<Vec<AspectValue>>, usize)> {
        let univ = self.univ_values(model_ref, sort)?;
        let i = univ
            .iter()
            .position(|v| matches!(v, AspectValue::Univ { name, .. } if &**name == symbol))?;
        Some((univ, i))
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
        Ok(
            self.value_matches_target(lhs, rhs, env)?
                && self.value_matches_target(rhs, lhs, env)?,
        )
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
                let Some(univ) = model_ref.univ_for_sort_arc(sort) else {
                    return Ok(false);
                };
                Ok(univ
                    .get(*idx)
                    .is_some_and(|candidate| candidate.as_str() == &**name))
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
