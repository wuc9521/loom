use crate::{
    backends::smt2::{
        ast::HornTerm,
        pred::{dnf_and, dnf_false, dnf_or, dnf_true, negate_dnf, Pred, PredAtom, PredDnf},
    },
    compile::{
        syntax::{TermConstructor, TermSignature},
        types::{AspectKey, AspectValue},
    },
    lang::eval::{eval_symbols, get_domain_for_sort},
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;
use sygus_parser::{Literal, Sort, SyGuSTerm};

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModelValue {
    Bool(bool),
    Int(isize),
    Univ { sort: String, name: String },
}

impl ModelValue {
    fn sort_name(&self) -> &'static str {
        match self {
            ModelValue::Bool(_) => "Bool",
            ModelValue::Int(_) => "Int",
            ModelValue::Univ { .. } => "universe",
        }
    }
}

fn aspect_scalar_env(aspect: &AspectKey) -> Vec<(String, AspectValue)> {
    let mut env = vec![(
        "model".to_string(),
        AspectValue::Model(aspect.model.clone()),
    )];
    for (name, value) in &aspect.fields {
        env.push((name.clone(), value.clone()));
    }
    env
}

pub fn eval_constructor_case(
    file: &DelisProg,
    expr: &Value,
    current_aspect: &AspectKey,
    aspect_constructor: &str,
    predicate_names: &HashMap<AspectKey, (String, String)>,
    semantic_relations: &[String],
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<PredDnf, String> {
    let scalar_env = aspect_scalar_env(current_aspect);
    eval_semantic_dnf(
        file,
        expr,
        current_aspect,
        aspect_constructor,
        predicate_names,
        &scalar_env,
        semantic_relations,
        child_vars,
        sig,
        current_constructor,
        current_term,
        self_var,
    )
}

fn eval_semantic_dnf(
    problem: &DelisProg,
    expr: &Value,
    current_aspect: &AspectKey,
    aspect_constructor: &str,
    predicate_names: &HashMap<AspectKey, (String, String)>,
    scalar_env: &[(String, AspectValue)],
    semantic_relations: &[String],
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<PredDnf, String> {
    if let Some(symbol) = expr.as_symbol() {
        return match symbol {
            "true" => Ok(dnf_true()),
            "false" => Ok(dnf_false()),
            _ => Err(format!("unsupported semantic symbol {symbol}")),
        };
    }
    let items = expr
        .to_ref_vec()
        .ok_or_else(|| format!("semantic expression must be a list: {expr}"))?;
    let op = items
        .first()
        .and_then(|v| v.as_symbol())
        .ok_or_else(|| format!("semantic expression must start with a symbol: {expr}"))?;
    match op {
        "and" => items[1..].iter().try_fold(dnf_true(), |acc, arg| {
            Ok(dnf_and(
                acc,
                eval_semantic_dnf(
                    problem,
                    arg,
                    current_aspect,
                    aspect_constructor,
                    predicate_names,
                    scalar_env,
                    semantic_relations,
                    child_vars,
                    sig,
                    current_constructor,
                    current_term,
                    self_var,
                )?,
            ))
        }),
        "or" => items[1..].iter().try_fold(dnf_false(), |acc, arg| {
            Ok(dnf_or(
                acc,
                eval_semantic_dnf(
                    problem,
                    arg,
                    current_aspect,
                    aspect_constructor,
                    predicate_names,
                    scalar_env,
                    semantic_relations,
                    child_vars,
                    sig,
                    current_constructor,
                    current_term,
                    self_var,
                )?,
            ))
        }),
        "not" if items.len() == 2 => Ok(negate_dnf(&eval_semantic_dnf(
            problem,
            items[1],
            current_aspect,
            aspect_constructor,
            predicate_names,
            scalar_env,
            semantic_relations,
            child_vars,
            sig,
            current_constructor,
            current_term,
            self_var,
        )?)),
        "exists" if items.len() == 3 => {
            let mut out = dnf_false();
            for extra_env in enumerate_exists_bindings(problem, items[1], scalar_env)? {
                let mut next_env = scalar_env.to_vec();
                next_env.extend(extra_env);
                out = dnf_or(
                    out,
                    eval_semantic_dnf(
                        problem,
                        items[2],
                        current_aspect,
                        aspect_constructor,
                        predicate_names,
                        &next_env,
                        semantic_relations,
                        child_vars,
                        sig,
                        current_constructor,
                        current_term,
                        self_var,
                    )?,
                );
            }
            Ok(out)
        }
        "forall" if items.len() == 3 => {
            let mut out = dnf_true();
            for extra_env in enumerate_exists_bindings(problem, items[1], scalar_env)? {
                let mut next_env = scalar_env.to_vec();
                next_env.extend(extra_env);
                out = dnf_and(
                    out,
                    eval_semantic_dnf(
                        problem,
                        items[2],
                        current_aspect,
                        aspect_constructor,
                        predicate_names,
                        &next_env,
                        semantic_relations,
                        child_vars,
                        sig,
                        current_constructor,
                        current_term,
                        self_var,
                    )?,
                );
            }
            Ok(out)
        }
        "=" | "<" | "<=" => Ok(
            if eval_semantic_bool(problem, &current_aspect.model, expr, scalar_env)? {
                dnf_true()
            } else {
                dnf_false()
            },
        ),
        _ if semantic_relations.iter().any(|rel| rel == op) => {
            if items.len() != 3 {
                return Err(format!(
                    "semantic call {expr} expects exactly term and aspect arguments"
                ));
            }
            let aspect = semantic_call_aspect(
                problem,
                items[2],
                op,
                current_aspect,
                aspect_constructor,
                scalar_env,
            )?;
            let (positive, negative) = predicate_names
                .get(&aspect)
                .ok_or_else(|| {
                    format!(
                        "semantic call references aspect outside encoded space: {}({}; {})",
                        aspect.relation,
                        aspect.model,
                        aspect.display_fields()
                    )
                })?
                .clone();
            let pred = Pred::new(positive, negative);
            let terms = semantic_call_terms(
                items[1],
                child_vars,
                sig,
                current_constructor,
                current_term,
                self_var,
            )?;
            Ok(terms
                .into_iter()
                .map(|term| vec![PredAtom::new(pred.clone(), term)])
                .collect())
        }
        _ => {
            let models = models_by_name(problem);
            match eval_model_term(problem, &current_aspect.model, expr, scalar_env, &models)? {
                ModelValue::Bool(true) => Ok(dnf_true()),
                ModelValue::Bool(false) => Ok(dnf_false()),
                other => Err(format!(
                    "semantic expression {expr} must evaluate to Bool, got {}",
                    other.sort_name()
                )),
            }
        }
    }
}

fn semantic_call_terms(
    target: &Value,
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<Vec<HornTerm>, String> {
    if let Some(symbol) = target.as_symbol() {
        if child_vars.contains(symbol) {
            return Ok(vec![HornTerm::var(symbol)]);
        }
        if symbol == self_var {
            return Ok(vec![current_term.clone()]);
        }
        return Err(format!(
            "semantic call target {symbol} is not a pattern child; use an explicit navigation form"
        ));
    }
    let items = target.to_ref_vec().ok_or_else(|| {
        format!("semantic call target must be a child or navigation form: {target}")
    })?;
    match items.as_slice() {
        [op, inner] if op.as_symbol() == Some("parent") => {
            if inner.as_symbol() != Some(self_var) {
                return Err(format!(
                    "parent target must name the current term variable {self_var}: {target}"
                ));
            }
            let contexts =
                sig.parent_contexts(&current_constructor.result_sort, current_term.clone());
            if contexts.is_empty() {
                return Err(format!(
                    "no parent constructor context can contain sort {}",
                    current_constructor.result_sort
                ));
            }
            Ok(contexts)
        }
        _ => Err(format!(
            "unsupported semantic call navigation target {target}"
        )),
    }
}

fn model_by_name<'a>(
    problem: &'a DelisProg,
    model_name: &str,
) -> Result<&'a crate::lang::model::Model, String> {
    problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) if model.name == model_name => Some(model),
            _ => None,
        })
        .ok_or_else(|| format!("unknown model {model_name}"))
}

fn models_by_name(problem: &DelisProg) -> HashMap<String, &crate::lang::model::Model> {
    problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) => Some((model.name.clone(), model)),
            _ => None,
        })
        .collect()
}

fn aspect_projection(
    expr: &Value,
    env: &[(String, AspectValue)],
) -> Option<Result<AspectValue, String>> {
    let items = expr.to_ref_vec()?;
    let [field, aspect] = items.as_slice() else {
        return None;
    };
    if aspect.as_symbol() != Some("asp") {
        return None;
    }
    let field = match field.as_symbol() {
        Some(field) => field,
        None => {
            return Some(Err(format!(
                "aspect projection field must be a symbol: {expr}"
            )));
        }
    };
    Some(Ok(lookup(env, field)
        .cloned()
        .unwrap_or_else(|| AspectValue::Symbol(field.to_string()))))
}

fn model_name_expr(expr: &Value, env: &[(String, AspectValue)]) -> Result<String, String> {
    match raw_value_expr(expr, env)? {
        AspectValue::Model(model) => Ok(model),
        AspectValue::Symbol(symbol)
            if env.iter().any(|(_, value)| {
                matches!(value, AspectValue::Model(model_name) if model_name == &symbol)
            }) =>
        {
            Ok(symbol)
        }
        other => Err(format!("expected Model expression, got {other:?}")),
    }
}

fn sorted_model_domain_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<(String, String, Vec<String>), String> {
    let items = expr.to_ref_vec().ok_or_else(|| {
        format!("expected sorted model domain expression (Sort model), got {expr}")
    })?;
    let [sort_expr, model_expr] = items.as_slice() else {
        return Err(format!(
            "sorted model domain expression must have shape (Sort model), got {expr}"
        ));
    };
    let sort = sort_expr
        .as_symbol()
        .ok_or_else(|| format!("domain sort must be a symbol: {sort_expr}"))?
        .to_string();
    let model_name = model_name_expr(model_expr, env)?;
    let model = models
        .get(&model_name)
        .ok_or_else(|| format!("unknown model {model_name}"))?;
    let domain = get_domain_for_sort(model, &sort)
        .ok_or_else(|| format!("model {model_name} has no universe for sort {sort}"))?;
    Ok((sort, model_name, domain))
}

fn raw_value_expr(expr: &Value, env: &[(String, AspectValue)]) -> Result<AspectValue, String> {
    if let Some(value) = expr.as_i64() {
        return if value >= 0 {
            Ok(AspectValue::Int(value as usize))
        } else {
            Err(format!(
                "negative integer value {value} is not a finite aspect value"
            ))
        };
    }
    if let Some(symbol) = expr.as_symbol() {
        return Ok(lookup(env, symbol)
            .cloned()
            .unwrap_or_else(|| AspectValue::Symbol(symbol.to_string())));
    }
    if let Some(value) = aspect_projection(expr, env) {
        return value;
    }
    Err(format!("expected finite value expression, got {expr}"))
}

fn value_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<AspectValue, String> {
    if let Ok(value) = raw_value_expr(expr, env) {
        return Ok(value);
    }
    eval_int(expr, env, models).and_then(|value| {
        if value >= 0 {
            Ok(AspectValue::Int(value as usize))
        } else {
            Err(format!(
                "negative integer value {value} is not a finite aspect value"
            ))
        }
    })
}

fn func_sig<'a>(problem: &'a DelisProg, name: &str) -> Option<&'a crate::lang::signature::FuncSig> {
    problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::DeclareFunc(sig) if sig.name == name => Some(sig),
        _ => None,
    })
}

fn eval_model_term(
    problem: &DelisProg,
    model_name: &str,
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<ModelValue, String> {
    if let Some(value) = expr.as_i64() {
        return Ok(ModelValue::Int(value as isize));
    }
    if let Some(symbol) = expr.as_symbol() {
        return eval_model_symbol(problem, model_name, symbol, env);
    }
    if let Some(value) = aspect_projection(expr, env) {
        return aspect_value_to_model_value(problem, model_name, &value?);
    }
    if let Ok(value) = eval_int(expr, env, models) {
        return Ok(ModelValue::Int(value));
    }
    let items = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected model term expression, got {expr}"))?;
    let func = items
        .first()
        .and_then(|item| item.as_symbol())
        .ok_or_else(|| format!("model term expression must start with a symbol: {expr}"))?;
    eval_model_application(problem, model_name, func, &items[1..], env, models)
}

fn eval_model_symbol(
    problem: &DelisProg,
    model_name: &str,
    symbol: &str,
    env: &[(String, AspectValue)],
) -> Result<ModelValue, String> {
    match symbol {
        "true" => return Ok(ModelValue::Bool(true)),
        "false" => return Ok(ModelValue::Bool(false)),
        _ => {}
    }
    if let Some(value) = lookup(env, symbol) {
        return aspect_value_to_model_value(problem, model_name, value);
    }
    if let Some(sig) = func_sig(problem, symbol) {
        if sig.arg_sorts.is_empty() {
            return eval_model_application(problem, model_name, symbol, &[], env, &models_by_name(problem));
        }
    }
    let model = model_by_name(problem, model_name)?;
    let matching_sorts = element_sorts(model, symbol);
    match matching_sorts.as_slice() {
        [sort] => Ok(ModelValue::Univ {
            sort: sort.clone(),
            name: symbol.to_string(),
        }),
        [] => Err(format!(
            "symbol {symbol} is neither a local value, finite universe element, nor zero-arity model function in model {model_name}"
        )),
        sorts => Err(format!(
            "symbol {symbol} is ambiguous in model {model_name}; it belongs to sorts {}",
            sorts.join(", ")
        )),
    }
}

fn eval_model_application(
    problem: &DelisProg,
    model_name: &str,
    func: &str,
    args: &[&Value],
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<ModelValue, String> {
    let model = model_by_name(problem, model_name)?;
    let sig = func_sig(problem, func)
        .ok_or_else(|| format!("model function {func} has no declare-fun signature"))?;
    if sig.arg_sorts.len() != args.len() {
        return Err(format!(
            "{func} expects {} arguments, got {}",
            sig.arg_sorts.len(),
            args.len()
        ));
    }
    let mut arg_symbols = Vec::new();
    for (arg, sort) in args.iter().zip(sig.arg_sorts.iter()) {
        let value = eval_model_term(problem, model_name, arg, env, models)?;
        let sort_name = sort.to_string();
        let Some(arg_symbol) = model_value_as_sort(model, &value, &sort_name)? else {
            if sig.ret_sort.to_string() == "Bool" {
                return Ok(ModelValue::Bool(false));
            }
            return Err(format!(
                "argument {arg} is outside sort {sort_name} for non-Boolean function {func}"
            ));
        };
        arg_symbols.push(arg_symbol);
    }
    let value = eval_symbols(func, &arg_symbols, &model.func_interps)
        .ok_or_else(|| format!("model {model_name} has no interpretation for {func}{arg_symbols:?}"))?;
    interped_value_to_model_value(problem, model_name, value, &sig.ret_sort)
}

fn aspect_value_to_model_value(
    problem: &DelisProg,
    model_name: &str,
    value: &AspectValue,
) -> Result<ModelValue, String> {
    match value {
        AspectValue::Int(value) => Ok(ModelValue::Int(*value as isize)),
        AspectValue::Univ { sort, name } => Ok(ModelValue::Univ {
            sort: sort.clone(),
            name: name.clone(),
        }),
        AspectValue::Symbol(symbol) => eval_model_symbol(problem, model_name, symbol, &[]),
        AspectValue::Model(model) => Err(format!(
            "expected model term value in model {model_name}, got model {model}"
        )),
    }
}

fn model_value_as_sort(
    model: &crate::lang::model::Model,
    value: &ModelValue,
    expected_sort: &str,
) -> Result<Option<String>, String> {
    match (value, expected_sort) {
        (ModelValue::Univ { sort, name }, expected) if sort == expected => Ok(Some(name.clone())),
        (ModelValue::Univ { name, .. }, expected)
            if get_domain_for_sort(model, expected)
                .map(|domain| domain.iter().any(|elem| elem == name))
                .unwrap_or(false) =>
        {
            Ok(Some(name.clone()))
        }
        (ModelValue::Univ { sort, name }, expected) => Err(format!(
            "expected element of sort {expected}, got {name} from sort {sort}"
        )),
        (ModelValue::Bool(value), "Bool") => Ok(Some(value.to_string())),
        (ModelValue::Int(value), "Int") => Ok(Some(value.to_string())),
        (ModelValue::Int(value), expected) => {
            let domain = get_domain_for_sort(model, expected)
                .ok_or_else(|| format!("model has no universe for sort {expected}"))?;
            if *value < 0 {
                return Ok(None);
            }
            Ok(domain.get(*value as usize).cloned())
        }
        (other, expected) => Err(format!(
            "expected argument of sort {expected}, got {}",
            other.sort_name()
        )),
    }
}

fn interped_value_to_model_value(
    problem: &DelisProg,
    model_name: &str,
    value: &crate::lang::model::InterpedValue,
    expected_sort: &Sort,
) -> Result<ModelValue, String> {
    let expected = expected_sort.to_string();
    match value {
        crate::lang::model::InterpedValue::Literal(lit) => literal_to_model_value(lit, &expected),
        crate::lang::model::InterpedValue::UnivElem(elem) => {
            let model = model_by_name(problem, model_name)?;
            if !get_domain_for_sort(model, &expected)
                .map(|domain| domain.iter().any(|name| name == &elem.name))
                .unwrap_or(false)
            {
                return Err(format!(
                    "model {model_name} interpretation returns {}, which is not in sort {expected}",
                    elem.name
                ));
            }
            Ok(ModelValue::Univ {
                sort: expected,
                name: elem.name.clone(),
            })
        }
        crate::lang::model::InterpedValue::FuncApp(name, args) => {
            let src = format!(
                "({} {})",
                name,
                args.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
            );
            let expr = lexpr::from_str(&src)
                .map_err(|e| format!("failed to parse interpreted function value {src}: {e}"))?;
            let models = models_by_name(problem);
            let value = eval_model_term(problem, model_name, &expr, &[], &models)?;
            ensure_model_value_sort(problem, model_name, value, &expected)
        }
    }
}

fn literal_to_model_value(lit: &Literal, expected_sort: &str) -> Result<ModelValue, String> {
    let src = lit.to_string();
    match expected_sort {
        "Bool" if src == "true" => Ok(ModelValue::Bool(true)),
        "Bool" if src == "false" => Ok(ModelValue::Bool(false)),
        "Int" => src
            .parse::<isize>()
            .map(ModelValue::Int)
            .map_err(|_| format!("expected Int literal, got {src}")),
        _ => Err(format!("literal {src} is not a value of sort {expected_sort}")),
    }
}

fn ensure_model_value_sort(
    problem: &DelisProg,
    model_name: &str,
    value: ModelValue,
    expected_sort: &str,
) -> Result<ModelValue, String> {
    let model = model_by_name(problem, model_name)?;
    match expected_sort {
        "Bool" if matches!(value, ModelValue::Bool(_)) => Ok(value),
        "Int" if matches!(value, ModelValue::Int(_)) => Ok(value),
        expected => {
            let symbol = model_value_as_sort(model, &value, expected)?.ok_or_else(|| {
                format!("value is outside sort {expected} in model {model_name}")
            })?;
            Ok(ModelValue::Univ {
                sort: expected.to_string(),
                name: symbol,
            })
        }
    }
}

fn element_sorts(model: &crate::lang::model::Model, symbol: &str) -> Vec<String> {
    model
        .univ_defs
        .iter()
        .filter_map(|univ| {
            univ.elems
                .iter()
                .any(|elem| elem.name == symbol)
                .then(|| univ.sort.to_string())
        })
        .collect()
}

fn enumerate_exists_bindings(
    problem: &DelisProg,
    bindings: &Value,
    scalar_env: &[(String, AspectValue)],
) -> Result<Vec<Vec<(String, AspectValue)>>, String> {
    let binding_items = bindings
        .to_ref_vec()
        .ok_or_else(|| format!("exists bindings must be a list: {bindings}"))?;
    let mut domains = Vec::new();
    let models = models_by_name(problem);
    for binding in binding_items {
        let item = binding
            .to_ref_vec()
            .ok_or_else(|| format!("exists binding must be a list: {binding}"))?;
        let [name, domain] = item.as_slice() else {
            return Err(format!(
                "exists binding must have shape (name domain): {binding}"
            ));
        };
        let name = name
            .as_symbol()
            .ok_or_else(|| format!("exists binding name must be a symbol: {name}"))?
            .to_string();
        let domain_items = domain
            .to_ref_vec()
            .ok_or_else(|| format!("exists domain must be a list: {domain}"))?;
        match domain_items.first().and_then(|v| v.as_symbol()) {
            Some("int-range") if domain_items.len() == 3 => {
                let lo = eval_int(domain_items[1], scalar_env, &models)?;
                let hi = eval_int(domain_items[2], scalar_env, &models)?;
                if lo < 0 || hi < lo {
                    domains.push((name, Vec::new()));
                } else {
                    domains.push((
                        name,
                        (lo as usize..=hi as usize)
                            .map(AspectValue::Int)
                            .collect::<Vec<_>>(),
                    ));
                }
            }
            Some("domain") if domain_items.len() == 2 => {
                let (sort, _model_name, domain_syms) =
                    sorted_model_domain_expr(domain_items[1], scalar_env, &models)?;
                let values = domain_syms
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort.clone(),
                        name,
                    })
                    .collect();
                domains.push((name, values));
            }
            _ => {
                return Err(format!(
                    "supported quantified domains are (int-range lo hi) and (domain (Sort model-expr)): {domain}"
                ));
            }
        }
    }

    let mut out = Vec::new();
    for values in domains
        .iter()
        .map(|(_, values)| values)
        .multi_cartesian_product()
    {
        out.push(
            domains
                .iter()
                .zip(values)
                .map(|((name, _), value)| (name.clone(), value.clone()))
                .collect(),
        );
    }
    Ok(out)
}

fn semantic_call_aspect(
    problem: &DelisProg,
    aspect_expr: &Value,
    relation: &str,
    current_aspect: &AspectKey,
    aspect_constructor: &str,
    scalar_env: &[(String, AspectValue)],
) -> Result<AspectKey, String> {
    if aspect_expr.as_symbol() == Some("asp") {
        let mut aspect = current_aspect.clone();
        aspect.relation = relation.to_string();
        return Ok(aspect);
    }
    let items = aspect_expr.to_ref_vec().ok_or_else(|| {
        format!("semantic call aspect must be asp or an aspect application: {aspect_expr}")
    })?;
    if items.is_empty() {
        return Err(format!("empty aspect expression {aspect_expr}"));
    }
    let constructor = items
        .first()
        .and_then(|item| item.as_symbol())
        .ok_or_else(|| format!("aspect expression head must be a symbol: {aspect_expr}"))?;
    if constructor != aspect_constructor {
        return Err(format!(
            "semantic call aspect constructor must be {aspect_constructor}, got {constructor}"
        ));
    }
    let mut model = None;
    let mut fields = Vec::new();
    let models = models_by_name(problem);
    let mut field_idx = 0usize;
    for arg in items.iter().skip(1) {
        let value =
            raw_value_expr(arg, scalar_env).or_else(|_| value_expr(arg, scalar_env, &models))?;
        if let AspectValue::Model(model_name) = value {
            model = Some(model_name);
            continue;
        }
        let Some((field_name, _)) = current_aspect.fields.get(field_idx) else {
            return Err(format!(
                "too many scalar fields in aspect expression {aspect_expr}"
            ));
        };
        fields.push((field_name.clone(), value));
        field_idx += 1;
    }
    if field_idx != current_aspect.fields.len() {
        return Err(format!(
            "aspect expression {aspect_expr} provides {field_idx} scalar fields, expected {}",
            current_aspect.fields.len()
        ));
    }
    Ok(AspectKey {
        relation: relation.to_string(),
        model: model.unwrap_or_else(|| current_aspect.model.clone()),
        fields,
    })
}

pub fn eval_sygus_int(
    term: &SyGuSTerm,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<isize, String> {
    let expr = lexpr::from_str(&term.to_string())
        .map_err(|e| format!("failed to parse integer aspect expression {term}: {e}"))?;
    eval_int(&expr, env, models)
}
fn lookup<'a>(env: &'a [(String, AspectValue)], name: &str) -> Option<&'a AspectValue> {
    env.iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

pub fn eval_bool(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<bool, String> {
    if let Some(symbol) = expr.as_symbol() {
        return match symbol {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(format!("unknown boolean symbol {symbol}")),
        };
    }
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected boolean expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "boolean expression must start with an operator: {expr}"
        ));
    };
    match op {
        "and" => list[1..]
            .iter()
            .try_fold(true, |acc, expr| Ok(acc && eval_bool(expr, env, models)?)),
        "or" => list[1..]
            .iter()
            .try_fold(false, |acc, expr| Ok(acc || eval_bool(expr, env, models)?)),
        "not" if list.len() == 2 => Ok(!eval_bool(list[1], env, models)?),
        "=" | "<" | "<=" => {
            if list.len() != 3 {
                return Err(format!("{op} expects two arguments, got {expr}"));
            }
            let lhs = eval_int(list[1], env, models)?;
            let rhs = eval_int(list[2], env, models)?;
            match op {
                "=" => Ok(lhs == rhs),
                "<" => Ok(lhs < rhs),
                "<=" => Ok(lhs <= rhs),
                _ => unreachable!(),
            }
        }
        _ => Err(format!("unsupported boolean operator {op} in {expr}")),
    }
}

fn eval_semantic_bool(
    problem: &DelisProg,
    model_name: &str,
    expr: &Value,
    env: &[(String, AspectValue)],
) -> Result<bool, String> {
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected semantic boolean expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "semantic boolean expression must start with an operator: {expr}"
        ));
    };
    if list.len() != 3 {
        return Err(format!("{op} expects two arguments, got {expr}"));
    }
    let models = models_by_name(problem);
    match op {
        "=" => {
            let lhs = eval_model_term(problem, model_name, list[1], env, &models)?;
            let rhs = eval_model_term(problem, model_name, list[2], env, &models)?;
            Ok(lhs == rhs)
        }
        "<" | "<=" => {
            let lhs = eval_int(list[1], env, &models)?;
            let rhs = eval_int(list[2], env, &models)?;
            match op {
                "<" => Ok(lhs < rhs),
                "<=" => Ok(lhs <= rhs),
                _ => unreachable!(),
            }
        }
        _ => Err(format!(
            "unsupported semantic boolean operator {op} in {expr}"
        )),
    }
}

fn eval_int(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &crate::lang::model::Model>,
) -> Result<isize, String> {
    if let Some(value) = expr.as_i64() {
        return Ok(value as isize);
    }
    if let Some(symbol) = expr.as_symbol() {
        return match lookup(env, symbol) {
            Some(AspectValue::Int(value)) => Ok(*value as isize),
            Some(value) => Err(format!("expected Int field {symbol}, got {value:?}")),
            None => Err(format!("unknown Int symbol {symbol}")),
        };
    }
    if let Some(value) = aspect_projection(expr, env) {
        return match value? {
            AspectValue::Int(value) => Ok(value as isize),
            other => Err(format!("expected Int aspect projection, got {other:?}")),
        };
    }
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected integer expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "integer expression must start with an operator: {expr}"
        ));
    };
    match op {
        "+" => list[1..]
            .iter()
            .try_fold(0, |acc, expr| Ok(acc + eval_int(expr, env, models)?)),
        "-" if list.len() == 2 => Ok(-eval_int(list[1], env, models)?),
        "-" => {
            let Some((first, rest)) = list[1..].split_first() else {
                return Err(format!("- expects at least one argument, got {expr}"));
            };
            let init = eval_int(first, env, models)?;
            rest.iter()
                .try_fold(init, |acc, expr| Ok(acc - eval_int(expr, env, models)?))
        }
        "len" if list.len() == 2 => {
            let (_sort, _model_name, domain) = sorted_model_domain_expr(list[1], env, models)?;
            Ok(domain.len() as isize)
        }
        _ => Err(format!("unsupported integer operator {op} in {expr}")),
    }
}
