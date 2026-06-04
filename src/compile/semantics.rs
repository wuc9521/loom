use crate::{compile::types::AspectValue, lang::model::InterpedValue};
use sygus_parser::SExpr as Value;
use rustc_hash::FxHashMap as HashMap;
use sygus_parser::Literal;

#[derive(Clone)]
pub struct Case {
    pub self_var: String,
    pub aspect_var: String,
    pub vars: Vec<String>,
    pub body: Value,
}

pub fn parse_cases(body: &Value, aspect_var: &str) -> Result<HashMap<String, Case>, String> {
    let outer = body
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must be a list: {body}"))?;
    let match_expr = match outer.as_slice() {
        [single] => single,
        _ => body,
    };
    let items = match_expr
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must contain a match expression: {body}"))?;
    if items.first().and_then(|v| v.as_symbol()) != Some("match") {
        return Err(format!(
            "define-semantics body must start with match: {body}"
        ));
    }
    let self_var = items
        .get(1)
        .and_then(|v| v.as_symbol())
        .ok_or_else(|| format!("match scrutinee must be a symbol: {match_expr}"))?
        .to_string();
    let mut cases = HashMap::default();
    for case in items.iter().skip(2) {
        let case_items = case
            .to_ref_vec()
            .ok_or_else(|| format!("match case must be a list: {case}"))?;
        let [pattern, body] = case_items.as_slice() else {
            return Err(format!("match case must have pattern and body: {case}"));
        };
        let pattern = pattern
            .to_ref_vec()
            .ok_or_else(|| format!("match pattern must be a list: {pattern}"))?;
        let cons = pattern
            .first()
            .and_then(|v| v.as_symbol())
            .ok_or_else(|| format!("match pattern constructor must be a symbol: {}", pattern[0]))?
            .to_string();
        let vars = pattern
            .iter()
            .skip(1)
            .map(|v| {
                v.as_symbol()
                    .map(str::to_string)
                    .ok_or_else(|| format!("match pattern variable must be a symbol: {v}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        cases.insert(
            cons,
            Case {
                self_var: self_var.clone(),
                aspect_var: aspect_var.to_string(),
                vars,
                body: (*body).clone(),
            },
        );
    }
    Ok(cases)
}

pub fn interped_bool(value: &InterpedValue) -> bool {
    matches!(value, InterpedValue::Literal(Literal::Bool(true)))
}

pub fn interped_value_matches(value: &InterpedValue, target: &AspectValue) -> bool {
    match (value, target) {
        (InterpedValue::UnivElem(elem), AspectValue::Univ { name, .. }) => {
            elem.name.as_str() == &**name
        }
        (InterpedValue::Literal(Literal::Bool(lhs)), AspectValue::Symbol(rhs)) => {
            lhs.to_string() == *rhs
        }
        (InterpedValue::Literal(lit), other) => lit.to_string() == other.display(),
        _ => false,
    }
}
