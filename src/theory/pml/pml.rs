use crate::lang::model::{InterpedValue, Model};
use crate::theory::utils::{eval, get_universe_domain_and_map};

#[derive(Debug, Default, Clone)]
pub struct KripkeCtx {
    pub init_name: Option<String>,
    pub tran_name: Option<String>,
    pub interp_table: crate::lang::runtime::ModelSymbolTable, // 新增：小符号表
}

pub fn handle_model_extcmd(ctx: &mut KripkeCtx, extcmd: &crate::lang::model::ThSpecCmd) {
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

pub fn get_transition_pred(model: &Model) -> Option<&str> {
    match &model.ext_ctx {
        Some(crate::theory::ModelCtx::PML(ctx)) => ctx.tran_name.as_deref(),
        _ => {
            eprintln!("Warning: Transition predicate not found");
            None
        }
    }
}

pub fn get_relation_adjacency(model: &Model, relation: &str) -> Vec<Vec<usize>> {
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
