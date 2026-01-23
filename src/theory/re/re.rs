#[derive(Debug, Default, Clone)]
pub struct StringCtx {
    pub interp_table: crate::lang::runtime::ModelSymbolTable, // 新增：小符号表
}

pub fn handle_model_extcmd(_ctx: &mut StringCtx, _extcmd: &crate::lang::model::ThSpecCmd) {
    // currently RE does not need extension commands
}
