use crate::{
    compile::{
        semantic::Case,
        types::{
            AspectKey, AspectValue, FiniteSpace, SemanticDomain, SemanticKind, SemanticProgram,
            SemanticSpec,
        },
    },
    lang::{
        eval::{eval_symbols, get_domain_for_sort},
        model::{InterpedValue, Model},
        signature::FuncSig,
    },
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;

pub trait EnvView {
    fn aspects(&self) -> &HashMap<String, AspectKey>;
    fn values(&self) -> &HashMap<String, AspectValue>;
}

pub struct SemanticContext<'a> {
    pub models: HashMap<String, &'a Model>,
    pub func_sigs: HashMap<String, FuncSig>,
    spaces: HashMap<String, FiniteSpace>,
    specs: HashMap<String, SemanticSpec>,
    domains_by_relation: HashMap<String, SemanticDomain>,
    domains_by_sort: HashMap<String, Vec<SemanticDomain>>,
    cases: HashMap<String, HashMap<String, Case>>,
}

impl<'a> SemanticContext<'a> {
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
        let mut domains_by_sort: HashMap<String, Vec<SemanticDomain>> = HashMap::new();
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
        })
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

    pub fn current_model(&self, env: &impl EnvView) -> Result<String, String> {
        env.aspects()
            .get("asp")
            .map(|aspect| aspect.model.clone())
            .ok_or_else(|| "no current aspect in semantic environment".to_string())
    }

    pub fn model(&self, name: &str) -> Result<&'a Model, String> {
        self.models
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown model {name}"))
    }

    pub fn argument_domains(
        &self,
        sig: &FuncSig,
        model: &Model,
    ) -> Result<Vec<Vec<AspectValue>>, String> {
        sig.arg_sorts
            .iter()
            .map(|sort| {
                let sort = sort.to_string();
                get_domain_for_sort(model, &sort)
                    .ok_or_else(|| format!("model {} has no universe for sort {sort}", model.name))
                    .map(|values| {
                        values
                            .into_iter()
                            .map(|name| AspectValue::Univ {
                                sort: sort.clone(),
                                name,
                            })
                            .collect()
                    })
            })
            .collect()
    }

    pub fn parse_domain_expr(
        &self,
        expr: &Value,
        env: &impl EnvView,
    ) -> Result<(String, String), String> {
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("domain expression must be a list: {expr}"))?;
        match items.as_slice() {
            [domain, sort_app] if domain.as_symbol() == Some("domain") => {
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
                    let value = self.eval_value_for_sort(arg, sort, env)?;
                    match &value {
                        AspectValue::Univ { sort: actual, .. } if actual == sort => {}
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
                        AspectValue::Univ { sort, .. } if sorts.contains(sort) => {}
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
                if let Some(model_ref) = self.models.get(&model) {
                    for univ in &model_ref.univ_defs {
                        if univ.elems.iter().any(|elem| elem.name == symbol) {
                            return Ok(AspectValue::Univ {
                                sort: univ.sort.to_string(),
                                name: symbol.to_string(),
                            });
                        }
                    }
                    if let Some(sig) = self.func_sigs.get(symbol) {
                        if sig.arg_sorts.is_empty() {
                            let value = eval_symbols(symbol, &[], &model_ref.func_interps)
                                .ok_or_else(|| {
                                    format!("model {model} has no value for {symbol}")
                                })?;
                            return interped_aspect_value(&sig.ret_sort.to_string(), value);
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
        Err(format!("unsupported value expression: {expr}"))
    }

    pub fn eval_bool_value(&self, expr: &Value, env: &impl EnvView) -> Result<bool, String> {
        if let Some(value) = expr.as_bool() {
            return Ok(value);
        }
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(format!("expected Bool expression, got {expr}")),
            };
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected Bool expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("Bool expression operator must be a symbol: {expr}"))?;
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
            _ => Err(format!("unsupported Bool expression: {expr}")),
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
                let values = get_domain_for_sort(model_ref, sort)
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
        let model_ref = self.model(&model)?;
        let arg_names = args
            .iter()
            .zip(sig.arg_sorts.iter())
            .map(|(arg, sort)| {
                self.eval_value_for_sort(arg, &sort.to_string(), env)
                    .map(|value| value.display())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let value = eval_symbols(func, &arg_names, &model_ref.func_interps).ok_or_else(|| {
            format!(
                "model {model} has no value for {func}({})",
                arg_names.join(" ")
            )
        })?;
        interped_aspect_value(&sig.ret_sort.to_string(), value)
    }

    pub fn eval_value_for_sort(
        &self,
        expr: &Value,
        sort: &str,
        env: &impl EnvView,
    ) -> Result<AspectValue, String> {
        let value = self.eval_value(expr, env)?;
        if let AspectValue::Int(idx) = &value {
            let model = self.current_model(env)?;
            let model_ref = self.model(&model)?;
            if let Some(domain) = get_domain_for_sort(model_ref, sort) {
                if let Some(name) = domain.get(*idx) {
                    return Ok(AspectValue::Univ {
                        sort: sort.to_string(),
                        name: name.clone(),
                    });
                }
            }
        }
        if matches!(&value, AspectValue::Symbol(_)) {
            let model = self.current_model(env)?;
            let model_ref = self.model(&model)?;
            if get_domain_for_sort(model_ref, sort)
                .is_some_and(|domain| domain.contains(&value.display()))
            {
                return Ok(AspectValue::Univ {
                    sort: sort.to_string(),
                    name: value.display(),
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
                return Some(sort.clone());
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
                        return Some(sort.clone());
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
            (AspectValue::Int(idx), AspectValue::Univ { sort, name }) => {
                let model = self.current_model(env)?;
                let model_ref = self.model(&model)?;
                let Some(domain) = get_domain_for_sort(model_ref, sort) else {
                    return Ok(false);
                };
                Ok(domain.get(*idx).is_some_and(|candidate| candidate == name))
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
        Ok(eval_symbols(symbol, &[], &model_ref.func_interps)
            .map(|value| crate::compile::semantic::interped_value_matches(value, target)))
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
        let domains = self.argument_domains(sig, model_ref)?;
        let mut out = Vec::new();
        for tuple in domains
            .iter()
            .map(|values| values.iter())
            .multi_cartesian_product()
        {
            let arg_names = tuple
                .iter()
                .map(|value| value.display())
                .collect::<Vec<_>>();
            let Some(value) = eval_symbols(func, &arg_names, &model_ref.func_interps) else {
                continue;
            };
            if keep(value) {
                out.push(tuple.into_iter().cloned().collect());
            }
        }
        Ok(out)
    }
}

fn interped_aspect_value(sort: &str, value: &InterpedValue) -> Result<AspectValue, String> {
    match value {
        InterpedValue::UnivElem(elem) => Ok(AspectValue::Univ {
            sort: sort.to_string(),
            name: elem.name.clone(),
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
