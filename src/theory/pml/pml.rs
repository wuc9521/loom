use crate::lang::model::{FuncInterp, InterpedValue, ModelDef, PatternElem};
use crate::theory::utils::{build_symbol_map, pattern_match};
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

pub fn model_to_json_with_symbol_map(model: &ModelDef, ctx: &PMLCtx) -> Value {
    // 1. domain
    let mut domain_syms = vec![];
    for univ in &model.univ_defs {
        if univ.sort.to_string() == "U" {
            for elem in &univ.elems {
                domain_syms.push(elem.name.clone());
            }
        }
    }
    domain_syms.sort();
    let (sym2id, id2sym) = build_symbol_map(&domain_syms);
    let domain: Vec<usize> = (0..domain_syms.len()).collect();

    // 调试输出
    eprintln!("[PML DEBUG] model.name = {}", model.name);
    eprintln!("[PML DEBUG] ctx.tran_name = {:?}", ctx.tran_name);
    for interp in &model.func_interps {
        match interp {
            FuncInterp::MatchInterp(name, cases) => {
                let arity = cases
                    .iter()
                    .find_map(|c| c.pattern.as_ref())
                    .map(|p| p.elements.len())
                    .unwrap_or(0);
                eprintln!(
                    "[PML DEBUG] FuncInterp::MatchInterp: name = {}, arity = {}, cases = {:?}",
                    name, arity, cases
                );
            }
            FuncInterp::SimpleInterp(name, val) => {
                eprintln!(
                    "[PML DEBUG] FuncInterp::SimpleInterp: name = {}, val = {}",
                    name, val
                );
            }
        }
    }

    // 2. transitions
    let mut transitions: HashMap<usize, Vec<usize>> = HashMap::new();
    if let Some(tran_name) = &ctx.tran_name {
        for interp in &model.func_interps {
            if let FuncInterp::MatchInterp(name, cases) = interp {
                let arity = cases
                    .iter()
                    .find_map(|c| c.pattern.as_ref())
                    .map(|p| p.elements.len())
                    .unwrap_or(0);
                let is_bool = cases.iter().any(|c| matches!(&c.value, InterpedValue::Literal(lit) if lit.to_string() == "true" || lit.to_string() == "false"));
                if is_bool && arity == 2 && name == tran_name {
                    let dom = &domain;
                    let tuples: Vec<Vec<usize>> = dom
                        .iter()
                        .flat_map(|&a| dom.iter().map(move |&b| vec![a, b]))
                        .collect();
                    for tuple in tuples {
                        let mut matched = false;
                        for case in cases {
                            if let Some(pat) = &case.pattern {
                                if pattern_match(&tuple, pat, &id2sym) {
                                    if let InterpedValue::Literal(lit) = &case.value {
                                        if lit.to_string() == "true" {
                                            transitions.entry(tuple[0]).or_default().push(tuple[1]);
                                        }
                                    }
                                    matched = true;
                                    break;
                                }
                            }
                        }
                        // fallback (_ _)
                        if !matched {
                            for case in cases {
                                if let Some(pat) = &case.pattern {
                                    if pat
                                        .elements
                                        .iter()
                                        .all(|e| matches!(e, PatternElem::Wildcard))
                                    {
                                        if let InterpedValue::Literal(lit) = &case.value {
                                            if lit.to_string() == "true" {
                                                transitions
                                                    .entry(tuple[0])
                                                    .or_default()
                                                    .push(tuple[1]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. init
    let mut init = vec![];
    if let Some(init_name) = &ctx.init_name {
        for interp in &model.func_interps {
            if let FuncInterp::SimpleInterp(name, val) = interp {
                if name == init_name {
                    if let InterpedValue::UnivElem(elem) = val {
                        if let Some(&idx) = sym2id.get(&elem.name) {
                            init.push(idx);
                        }
                    }
                }
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
        for (prop, cases) in &prop_cases {
            let tuple = vec![state];
            let mut matched = false;
            for case in *cases {
                if let Some(pat) = &case.pattern {
                    if pattern_match(&tuple, pat, &id2sym) {
                        if let InterpedValue::Literal(lit) = &case.value {
                            if lit.to_string() == "true" {
                                props.push(prop.clone());
                            }
                        }
                        matched = true;
                        break;
                    }
                }
            }
            // fallback (_)
            if !matched {
                for case in *cases {
                    if let Some(pat) = &case.pattern {
                        if pat
                            .elements
                            .iter()
                            .all(|e| matches!(e, PatternElem::Wildcard))
                        {
                            if let InterpedValue::Literal(lit) = &case.value {
                                if lit.to_string() == "true" {
                                    props.push(prop.clone());
                                }
                            }
                        }
                    }
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

/// 获取指定关系名的邻接表（邻接表: from -> Vec<to>），domain为0..N
pub fn get_relation_adjacency(model: &ModelDef, relation: &str) -> Vec<Vec<usize>> {
    // 1. domain
    let mut domain_syms = vec![];
    for univ in &model.univ_defs {
        if univ.sort.to_string() == "U" {
            for elem in &univ.elems {
                domain_syms.push(elem.name.clone());
            }
        }
    }
    domain_syms.sort();
    let (_sym2id, id2sym) = build_symbol_map(&domain_syms);
    let domain: Vec<usize> = (0..domain_syms.len()).collect();
    let mut adjacency: Vec<Vec<usize>> = vec![vec![]; domain.len()];
    for interp in &model.func_interps {
        if let FuncInterp::MatchInterp(name, cases) = interp {
            if name == relation {
                let arity = cases.iter().find_map(|c| c.pattern.as_ref()).map(|p| p.elements.len()).unwrap_or(0);
                let is_bool = cases.iter().any(|c| matches!(&c.value, InterpedValue::Literal(lit) if lit.to_string() == "true" || lit.to_string() == "false"));
                if is_bool && arity == 2 {
                    for from in &domain {
                        for to in &domain {
                            let tuple = vec![*from, *to];
                            let mut matched = false;
                            for case in cases {
                                if let Some(pat) = &case.pattern {
                                    if pattern_match(&tuple, pat, &id2sym) {
                                        if let InterpedValue::Literal(lit) = &case.value {
                                            if lit.to_string() == "true" {
                                                adjacency[*from].push(*to);
                                            }
                                        }
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                            // fallback (_ _)
                            if !matched {
                                for case in cases {
                                    if let Some(pat) = &case.pattern {
                                        if pat.elements.iter().all(|e| matches!(e, PatternElem::Wildcard)) {
                                            if let InterpedValue::Literal(lit) = &case.value {
                                                if lit.to_string() == "true" {
                                                    adjacency[*from].push(*to);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    adjacency
}
