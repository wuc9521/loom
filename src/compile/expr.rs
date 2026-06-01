use crate::{compile::types::AspectValue, lang::model::Model};
use lexpr::Value;
use rustc_hash::FxHashMap as HashMap;
use sygus_parser::{Identifier, Literal, SyGuSTerm};

pub fn sygus_term_to_value(term: &SyGuSTerm) -> Result<Value, String> {
    match term {
        SyGuSTerm::Identifier(Identifier::Symbol(name)) => Ok(Value::symbol(name.as_str())),
        SyGuSTerm::Identifier(other) => Ok(Value::symbol(other.to_string())),
        SyGuSTerm::Literal(Literal::Bool(value)) => Ok(Value::symbol(if *value {
            "true"
        } else {
            "false"
        })),
        SyGuSTerm::Literal(Literal::Numeral(value)) => Ok(Value::from(*value as i64)),
        SyGuSTerm::Literal(Literal::Decimal(value)) => {
            Err(format!("decimal literals are unsupported in Delis constraints: {value}"))
        }
        SyGuSTerm::Literal(Literal::BinConst(value))
        | SyGuSTerm::Literal(Literal::HexConst(value)) => Ok(Value::symbol(value.as_str())),
        SyGuSTerm::Application(id, args) => {
            let mut values = Vec::with_capacity(args.len() + 1);
            values.push(Value::symbol(id.to_string()));
            for arg in args {
                values.push(sygus_term_to_value(arg)?);
            }
            Ok(Value::list(values))
        }
        SyGuSTerm::Let(bindings, body) => {
            let mut lets = Vec::with_capacity(bindings.len() + 1);
            lets.push(Value::symbol("let"));
            let mut binding_values = Vec::with_capacity(bindings.len());
            for binding in bindings {
                binding_values.push(Value::list(vec![
                    Value::symbol(binding.name.as_str()),
                    sygus_term_to_value(&binding.term)?,
                ]));
            }
            lets.push(Value::list(binding_values));
            lets.push(sygus_term_to_value(body)?);
            Ok(Value::list(lets))
        }
        SyGuSTerm::Annotated(inner, _) => sygus_term_to_value(inner),
        other => Err(format!("unsupported SyGuS term in Delis constraint lowering: {other}")),
    }
}

pub fn eval_sygus_int(
    term: &SyGuSTerm,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &Model>,
) -> Result<isize, String> {
    let expr = lexpr::from_str(&term.to_string())
        .map_err(|e| format!("failed to parse integer aspect expression {term}: {e}"))?;
    eval_int(&expr, env, models)
}

pub fn eval_bool(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &Model>,
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
        "ite" if list.len() == 4 => {
            if eval_bool(list[1], env, models)? {
                eval_bool(list[2], env, models)
            } else {
                eval_bool(list[3], env, models)
            }
        }
        "=" | "<" | "<=" => {
            if list.len() != 3 {
                return Err(format!("{op} expects two arguments, got {expr}"));
            }
            if op == "=" {
                if let Some(value) = eval_enum_equality(list[1], list[2], env)? {
                    return Ok(value);
                }
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

fn lookup<'a>(env: &'a [(String, AspectValue)], name: &str) -> Option<&'a AspectValue> {
    env.iter()
        .find_map(|(field, value)| (field == name).then_some(value))
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

fn sorted_model_univ_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &Model>,
) -> Result<(String, String, Vec<String>), String> {
    let items = expr.to_ref_vec().ok_or_else(|| {
        format!("expected sorted model univ expression (Sort model), got {expr}")
    })?;
    let [sort_expr, model_expr] = items.as_slice() else {
        return Err(format!(
            "sorted model univ expression must have shape (Sort model), got {expr}"
        ));
    };
    let sort = sort_expr
        .as_symbol()
        .ok_or_else(|| format!("univ sort must be a symbol: {sort_expr}"))?
        .to_string();
    let model_name = model_name_expr(model_expr, env)?;
    let model = models
        .get(&model_name)
        .ok_or_else(|| format!("unknown model {model_name}"))?;
    let univ = model.univ_for_sort(&sort)
        .ok_or_else(|| format!("model {model_name} has no universe for sort {sort}"))?;
    Ok((sort, model_name, univ))
}

fn enum_value_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
) -> Option<Result<AspectValue, String>> {
    if let Some(value) = aspect_projection(expr, env) {
        return match value {
            Ok(value @ AspectValue::Symbol(_)) => Some(Ok(value)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        };
    }
    if let Some(symbol) = expr.as_symbol() {
        if let Some(value @ AspectValue::Symbol(_)) = lookup(env, symbol) {
            return Some(Ok(value.clone()));
        }
    }
    None
}

fn eval_enum_equality(
    lhs: &Value,
    rhs: &Value,
    env: &[(String, AspectValue)],
) -> Result<Option<bool>, String> {
    if let Some(lhs_value) = enum_value_expr(lhs, env) {
        return Ok(Some(lhs_value? == raw_value_expr(rhs, env)?));
    }
    if let Some(rhs_value) = enum_value_expr(rhs, env) {
        return Ok(Some(raw_value_expr(lhs, env)? == rhs_value?));
    }
    Ok(None)
}

fn eval_int(
    expr: &Value,
    env: &[(String, AspectValue)],
    models: &HashMap<String, &Model>,
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
        "size" if list.len() == 2 => {
            let (_sort, _model_name, univ) = sorted_model_univ_expr(list[1], env, models)?;
            Ok(univ.len() as isize)
        }
        _ => Err(format!("unsupported integer operator {op} in {expr}")),
    }
}
