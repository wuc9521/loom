use crate::lang::model::Model;
use crate::theory::utils::get_universe_domain_and_map;
use serde_json::{json, Value};

#[derive(Debug, Default, Clone)]
pub struct StringCtx {
    pub interp_table: crate::lang::runtime::ModelSymbolTable, // 新增：小符号表
}

pub fn handle_model_extcmd(_ctx: &mut StringCtx, _extcmd: &crate::lang::model::ThSpecCmd) {
    // currently RE does not need extension commands
}

pub fn model_to_json_with_symbol_map(model: &Model, _ctx: &StringCtx) -> Value {
    // based on get_universe_domain_and_map, export RE model structure
    let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
    let domain: Vec<usize> = (0..domain_syms.len()).collect();
    json!({
        "domain": domain,
        "symbols": domain_syms,
    })
}
