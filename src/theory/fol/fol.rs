use sygus_parser::Sort;
use crate::lang::model::{ModelDef, FuncInterp, InterpedValue};
use serde_json::{json, Value};
use crate::theory::utils::{eval, get_universe_domain_and_map};

#[derive(Debug, Default, Clone)]
pub struct FOLCtx {
    pub qvars: Vec<(String, sygus_parser::Sort)>, // (name, sort)
}

pub fn handle_extcmd(ctx: &mut FOLCtx, extcmd: &crate::lang::model::ThSpecCmd) {
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
            // 可以扩展更多指令
        }
    }
}

/// 主导出函数
pub fn model_to_json_with_symbol_map(model: &ModelDef, _ctx: &FOLCtx) -> Value {
    // 1. domain: 提取符号
    let (domain_syms, id2sym, _universe_sort) = get_universe_domain_and_map(model);
    let sym2id: std::collections::HashMap<String, usize> = domain_syms.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect();
    let domain: Vec<usize> = (0..domain_syms.len()).collect();

    // 2. constants
    let mut constants = serde_json::Map::new();

    // 3. predicates
    let mut predicates = serde_json::Map::new();

    for interp in &model.func_interps {
        match interp {
            FuncInterp::SimpleInterp(name, val) => {
                // 0-arity
                if let InterpedValue::UnivElem(elem) = val {
                    if let Some(&idx) = sym2id.get(&elem.name) {
                        constants.insert(name.clone(), json!(idx));
                    }
                }
            }
            FuncInterp::MatchInterp(name, cases) => {
                // 判断 arity
                let arity = cases.iter().find_map(|c| c.pattern.as_ref()).map(|p| p.elements.len()).unwrap_or(0);
                // 只处理 Bool 返回值
                let is_bool = cases.iter().any(|c| matches!(&c.value, InterpedValue::Literal(lit) if lit.to_string() == "true" || lit.to_string() == "false"));
                if is_bool {
                    // 枚举所有参数元组
                    let mut value_tuples = vec![];
                    let dom = &domain;
                    let tuples = match arity {
                        1 => dom.iter().map(|&a| vec![a]).collect(),
                        2 => dom.iter().flat_map(|&a| dom.iter().map(move |&b| vec![a, b])).collect(),
                        _ => vec![],
                    };
                    for tuple in tuples {
                        if let Some(InterpedValue::Literal(lit)) = eval(name, &tuple, &model.func_interps, &id2sym) {
                            if lit.to_string() == "true" {
                                value_tuples.push(tuple.clone());
                            }
                        }
                    }
                    predicates.insert(name.clone(), json!({
                        "arity": arity,
                        "value": value_tuples
                    }));
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
