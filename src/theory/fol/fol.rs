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

// cfg -> benchmark -> frontend.
