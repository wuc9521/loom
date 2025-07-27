use crate::lang::model::{FuncInterp, Model};
use crate::parser::ast::LSTerm;
use crate::theory::utils::{eval, get_universe_domain_and_map};
use serde_json::{json, Value};
use sygus_parser::Sort;

#[derive(Debug, Default, Clone)]
pub struct FOModelCtx {
    pub qvars: Vec<(String, sygus_parser::Sort)>, // (name, sort)
    pub interp_table: crate::lang::runtime::ModelSymbolTable, // 新增：小符号表
}

pub fn handle_model_extcmd(ctx: &mut FOModelCtx, extcmd: &crate::lang::model::ThSpecCmd) {
    match extcmd.name.as_str() {
        "declare-qvar" => {
            let mut args = extcmd.args.split_whitespace();
            if let (Some(name), Some(sort)) = (args.next(), args.next()) {
                ctx.qvars
                    .push((name.to_string(), Sort::from_str(sort).unwrap()));
            }
        }
        // other FOL-specific commands
        _ => {
            // can extend more commands
        }
    }
}

/// export functions
pub fn model_to_json_with_symbol_map(model: &Model, _ctx: &FOModelCtx) -> Value {
    // 1. domain: extract symbols
    let (domain_syms, id2sym, _universe_sort) = get_universe_domain_and_map(model);
    let sym2id: std::collections::HashMap<String, usize> = domain_syms
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let domain: Vec<usize> = (0..domain_syms.len()).collect();

    // 2. constants: extract constants
    let mut constants = serde_json::Map::new();

    // 3. predicates: extract predicates
    let mut predicates = serde_json::Map::new();

    for interp in &model.func_interps {
        match interp {
            FuncInterp::SimpleInterp(name, args, _, term) => {
                // 0-arity functions are constants
                if args.is_empty() {
                    if let LSTerm::AppTerm(sym, _) = term {
                        if sym.to_string().starts_with('$') {
                            if let Some(&idx) = sym2id.get(&sym.to_string()) {
                                constants.insert(name.clone(), json!(idx));
                            }
                        }
                    }
                }
            }
            FuncInterp::MatchInterp(name, args, _, _, cases) => {
                // check arity
                let arity = args.len();

                // only handle Bool return value
                let is_bool = cases.iter().any(|c| {
                    matches!(&c.value, LSTerm::Literal(lit)
                        if lit.to_string() == "true" || lit.to_string() == "false")
                });

                if is_bool {
                    // enumerate all parameter tuples
                    let mut value_tuples = vec![];
                    let dom = &domain;
                    let tuples = match arity {
                        1 => dom.iter().map(|&a| vec![a]).collect(),
                        2 => dom
                            .iter()
                            .flat_map(|&a| dom.iter().map(move |&b| vec![a, b]))
                            .collect(),
                        _ => vec![],
                    };

                    for tuple in tuples {
                        if let Some(LSTerm::Literal(lit)) =
                            eval(name, &tuple, &model.func_interps, &id2sym)
                        {
                            if lit.to_string() == "true" {
                                value_tuples.push(tuple.clone());
                            }
                        }
                    }

                    predicates.insert(
                        name.clone(),
                        json!({
                            "arity": arity,
                            "value": value_tuples
                        }),
                    );
                }
            }
        }
    }

    json!({
        "domain": domain,
        "constants": constants,
        "predicates": predicates,
        "functions": serde_json::Map::new()
    })
}
