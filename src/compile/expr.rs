use super::syntax::{TermConstructor, TermSignature};
use super::types::{AspectKey, AspectValue};
use crate::chc::ast::HornTerm;
use crate::chc::pred::{dnf_and, dnf_false, dnf_or, dnf_true, negate_dnf, Pred, PredAtom, PredDnf};
use crate::lang::eval::{eval, get_universe_domain_and_map};
use crate::parser::{DelisCmd, DelisProg};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;
use sygus_parser::ast::SyGuSTerm;

fn model_len(file: &DelisProg, model_name: &str) -> Result<usize, String> {
    let model = file
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) if model.name == model_name => Some(model),
            _ => None,
        })
        .ok_or_else(|| format!("unknown model {model_name}"))?;
    let (domain, _id2sym, _sort) = get_universe_domain_and_map(model);
    Ok(domain.len())
}

fn aspect_scalar_env(aspect: &AspectKey, model_len: usize) -> Vec<(String, AspectValue)> {
    let mut env = vec![(
        "model".to_string(),
        AspectValue::Model(aspect.model.clone()),
    )];
    for (name, value) in &aspect.fields {
        let value = value
            .parse::<usize>()
            .map(AspectValue::Int)
            .unwrap_or_else(|_| AspectValue::Symbol(value.clone()));
        env.push((name.clone(), value));
    }
    env.push(("__len".to_string(), AspectValue::Int(model_len)));
    env
}

pub fn eval_constructor_case(
    file: &DelisProg,
    expr: &Value,
    current_aspect: &AspectKey,
    predicate_names: &HashMap<AspectKey, (String, String)>,
    semantic_relations: &[String],
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<PredDnf, String> {
    let model_len = model_len(file, &current_aspect.model)?;
    let scalar_env = aspect_scalar_env(current_aspect, model_len);
    eval_semantic_dnf(
        file,
        expr,
        current_aspect,
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
        "not" | "dual" if items.len() == 2 => Ok(negate_dnf(&eval_semantic_dnf(
            problem,
            items[1],
            current_aspect,
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
            if items.len() < 3 {
                return Err(format!(
                    "semantic call {expr} expects term and aspect arguments"
                ));
            }
            let aspect = semantic_call_aspect(items[2], op, current_aspect, scalar_env)?;
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
            let args = items[1..]
                .iter()
                .map(|arg| value_expr(arg, scalar_env, current_model_len(scalar_env)?))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(
                if eval_model_bool_values(problem, &current_aspect.model, op, &args)? {
                    dnf_true()
                } else {
                    dnf_false()
                },
            )
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

fn current_model_len(env: &[(String, AspectValue)]) -> Result<usize, String> {
    match lookup(env, "__len") {
        Some(AspectValue::Int(value)) => Ok(*value),
        _ => Err("internal error: missing __len in semantic environment".to_string()),
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

fn aspect_value_to_index(value: &AspectValue, id2sym: &[String]) -> Result<usize, String> {
    match value {
        AspectValue::Int(idx) => Ok(*idx),
        AspectValue::Symbol(sym) => id2sym
            .iter()
            .position(|candidate| candidate == sym)
            .ok_or_else(|| format!("unknown universe element {sym}")),
        AspectValue::Model(model) => Err(format!("expected universe element, got model {model}")),
    }
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

fn model_name_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<String, String> {
    match value_expr(expr, env, model_len)? {
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

fn value_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<AspectValue, String> {
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
    eval_int(expr, env, model_len).and_then(|value| {
        if value >= 0 {
            Ok(AspectValue::Int(value as usize))
        } else {
            Err(format!(
                "negative integer value {value} is not a finite aspect value"
            ))
        }
    })
}

fn eval_model_bool_values(
    problem: &DelisProg,
    model_name: &str,
    prop: &str,
    args: &[AspectValue],
) -> Result<bool, String> {
    let model = model_by_name(problem, model_name)?;
    let (_domain, id2sym, _sort) = get_universe_domain_and_map(model);
    let resolved_args = args
        .iter()
        .map(|arg| resolve_model_value(problem, model_name, arg))
        .collect::<Result<Vec<_>, _>>()?;
    let indices = resolved_args
        .iter()
        .map(|arg| aspect_value_to_index(arg, &id2sym))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(matches!(
        eval(prop, &indices, &model.func_interps, &id2sym),
        Some(crate::lang::model::InterpedValue::Literal(lit))
            if lit.to_string() == "true"
    ))
}

fn resolve_model_value(
    problem: &DelisProg,
    model_name: &str,
    value: &AspectValue,
) -> Result<AspectValue, String> {
    let AspectValue::Symbol(symbol) = value else {
        return Ok(value.clone());
    };
    let model = model_by_name(problem, model_name)?;
    let (_domain, id2sym, _sort) = get_universe_domain_and_map(model);
    if id2sym.iter().any(|elem| elem == symbol) {
        return Ok(value.clone());
    }
    let Some(value) = eval(symbol, &[], &model.func_interps, &id2sym) else {
        return Ok(AspectValue::Symbol(symbol.clone()));
    };
    match value {
        crate::lang::model::InterpedValue::UnivElem(elem) => {
            Ok(AspectValue::Symbol(elem.name.clone()))
        }
        other => Err(format!(
            "expected zero-arity model symbol {symbol} to evaluate to a universe element, got {other}"
        )),
    }
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
        let model_len = current_model_len(scalar_env)?;
        match domain_items.first().and_then(|v| v.as_symbol()) {
            Some("int-range") if domain_items.len() == 3 => {
                let lo = eval_int(domain_items[1], scalar_env, model_len)?;
                let hi = eval_int(domain_items[2], scalar_env, model_len)?;
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
            Some("domain") if domain_items.len() >= 2 => {
                let model_name = model_name_expr(domain_items[1], scalar_env, model_len)?;
                let model = model_by_name(problem, &model_name)?;
                let (domain_syms, _id2sym, _sort) = get_universe_domain_and_map(model);
                domains.push((
                    name,
                    domain_syms.into_iter().map(AspectValue::Symbol).collect(),
                ));
            }
            _ => {
                return Err(format!(
                    "supported quantified domains are (int-range lo hi) and (domain model-expr): {domain}"
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
    aspect_expr: &Value,
    relation: &str,
    current_aspect: &AspectKey,
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
    let mut model = None;
    let mut fields = Vec::new();
    let model_len = current_model_len(scalar_env)?;
    let mut field_idx = 0usize;
    for arg in items.iter().skip(1) {
        let value = value_expr(arg, scalar_env, model_len)?;
        if let AspectValue::Model(model_name) = value {
            model = Some(model_name);
            continue;
        }
        let Some((field_name, _)) = current_aspect.fields.get(field_idx) else {
            return Err(format!(
                "too many scalar fields in aspect expression {aspect_expr}"
            ));
        };
        fields.push((field_name.clone(), value.display()));
        field_idx += 1;
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
    model_len: usize,
) -> Result<isize, String> {
    let expr = lexpr::from_str(&term.to_string())
        .map_err(|e| format!("failed to parse integer aspect expression {term}: {e}"))?;
    eval_int(&expr, env, model_len)
}
fn lookup<'a>(env: &'a [(String, AspectValue)], name: &str) -> Option<&'a AspectValue> {
    env.iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

pub fn eval_bool(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
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
        "and" => {
            list[1..].iter().try_fold(
                true,
                |acc, expr| Ok(acc && eval_bool(expr, env, model_len)?),
            )
        }
        "or" => list[1..].iter().try_fold(false, |acc, expr| {
            Ok(acc || eval_bool(expr, env, model_len)?)
        }),
        "not" if list.len() == 2 => Ok(!eval_bool(list[1], env, model_len)?),
        "=" | "<" | "<=" => {
            if list.len() != 3 {
                return Err(format!("{op} expects two arguments, got {expr}"));
            }
            let lhs = eval_int(list[1], env, model_len)?;
            let rhs = eval_int(list[2], env, model_len)?;
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
    let model_len = current_model_len(env)?;
    match op {
        "=" => {
            let lhs =
                resolve_model_value(problem, model_name, &value_expr(list[1], env, model_len)?)?;
            let rhs =
                resolve_model_value(problem, model_name, &value_expr(list[2], env, model_len)?)?;
            Ok(lhs == rhs)
        }
        "<" | "<=" => {
            let lhs = eval_int(list[1], env, model_len)?;
            let rhs = eval_int(list[2], env, model_len)?;
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
    model_len: usize,
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
            .try_fold(0, |acc, expr| Ok(acc + eval_int(expr, env, model_len)?)),
        "-" if list.len() == 2 => Ok(-eval_int(list[1], env, model_len)?),
        "-" => {
            let Some((first, rest)) = list[1..].split_first() else {
                return Err(format!("- expects at least one argument, got {expr}"));
            };
            let init = eval_int(first, env, model_len)?;
            rest.iter()
                .try_fold(init, |acc, expr| Ok(acc - eval_int(expr, env, model_len)?))
        }
        "len" if list.len() == 2 => {
            model_name_expr(list[1], env, model_len)?;
            Ok(model_len as isize)
        }
        _ => Err(format!("unsupported integer operator {op} in {expr}")),
    }
}
