use crate::lang::model::Model;
use crate::theory::utils::get_universe_domain_and_map;
use serde_json::{json, Value};
use sygus_parser::Sort;

#[derive(Debug, Default, Clone)]
pub struct FOModelCtx {
    pub qvars: Vec<(String, Sort)>, // (name, sort)
    pub interp_table: crate::lang::runtime::ModelSymbolTable, 
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
    let (domain_syms, _, _universe_sort) = get_universe_domain_and_map(model);
    let _sym2id: std::collections::HashMap<String, usize> = domain_syms
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let domain: Vec<usize> = (0..domain_syms.len()).collect();

    // 2. constants: extract constants
    let constants = serde_json::Map::new();

    // 3. predicates: extract predicates
    let predicates = serde_json::Map::new();
    json!({
        "domain": domain,
        "constants": constants,
        "predicates": predicates,
        "functions": serde_json::Map::new()
    })
}
