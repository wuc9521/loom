use crate::lang::model::{FuncInterp, InterpedValue, ModelDef};
use crate::theory::utils::{eval, get_universe_domain_and_map};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct PMLCtx {
    pub init_name: Option<String>,
    pub tran_name: Option<String>,
}

pub fn handle_extcmd(ctx: &mut PMLCtx, extcmd: &crate::lang::model::ThSpecCmd) {
    match extcmd.name.as_str() {
        "set-init" => {
            ctx.init_name = Some(
                extcmd
                    .args
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string(),
            )
        }
        "set-tran" => {
            ctx.tran_name = Some(
                extcmd
                    .args
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string(),
            )
        }
        _ => {}
    }
}

pub fn get_transition_pred(model: &ModelDef) -> Option<&str> {
    match &model.ext_ctx {
        Some(crate::theory::Ctx::PML(ctx)) => ctx.tran_name.as_deref(),
        _ => {
            eprintln!("Warning: Transition predicate not found");
            None
        }
    }
}

pub fn model_to_json_with_symbol_map(model: &ModelDef, ctx: &PMLCtx) -> Value {
    // 1. domain
    let (domain_syms, id2sym, _universe_sort) = get_universe_domain_and_map(model);
    let sym2id: std::collections::HashMap<String, usize> = domain_syms
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let domain: Vec<usize> = (0..domain_syms.len()).collect();

    // 2. transitions
    let mut transitions: HashMap<usize, Vec<usize>> = HashMap::new();
    if let Some(tran_name) = get_transition_pred(model) {
        for from in &domain {
            for to in &domain {
                if let Some(InterpedValue::Literal(lit)) =
                    eval(tran_name, &[*from, *to], &model.func_interps, &id2sym)
                {
                    if lit.to_string() == "true" {
                        transitions.entry(*from).or_default().push(*to);
                    }
                }
            }
        }
    }

    // 3. init
    let mut init = vec![];
    if let Some(init_name) = &ctx.init_name {
        if let Some(InterpedValue::UnivElem(elem)) =
            eval(init_name, &[], &model.func_interps, &id2sym)
        {
            if let Some(&idx) = sym2id.get(&elem.name) {
                init.push(idx);
            }
        }
    }

    // 4. propositions & labels
    let mut propositions = vec![];
    let mut prop_cases = HashMap::new();
    let mut labels: HashMap<usize, Vec<String>> = HashMap::new();
    for interp in &model.func_interps {
        if let FuncInterp::MatchInterp(name, cases) = interp {
            let arity = cases
                .iter()
                .find_map(|c| c.pattern.as_ref())
                .map(|p| p.elements.len())
                .unwrap_or(0);
            let is_bool = cases.iter().any(|c| matches!(&c.value, InterpedValue::Literal(lit) if lit.to_string() == "true" || lit.to_string() == "false"));
            if is_bool && arity == 1 {
                propositions.push(name.clone());
                prop_cases.insert(name.clone(), cases);
            }
        }
    }
    for &state in &domain {
        let mut props = vec![];
        for prop in &propositions {
            if let Some(InterpedValue::Literal(lit)) =
                eval(prop, &[state], &model.func_interps, &id2sym)
            {
                if lit.to_string() == "true" {
                    props.push(prop.clone());
                }
            }
        }
        labels.insert(state, props);
    }

    json!({
        "domain": domain,
        "init": init,
        "propositions": propositions,
        "transitions": transitions,
        "labels": labels
    })
}

pub fn get_relation_adjacency(model: &ModelDef, relation: &str) -> Vec<Vec<usize>> {
    let (domain_syms, id2sym, _universe_sort) = get_universe_domain_and_map(model);
    let domain: Vec<usize> = (0..domain_syms.len()).collect();
    let mut adjacency: Vec<Vec<usize>> = vec![vec![]; domain.len()];
    for from in &domain {
        for to in &domain {
            if let Some(InterpedValue::Literal(lit)) =
                eval(relation, &[*from, *to], &model.func_interps, &id2sym)
            {
                if lit.to_string() == "true" {
                    adjacency[*from].push(*to);
                }
            }
        }
    }
    adjacency
}
