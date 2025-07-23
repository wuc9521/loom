use crate::lang::model::ThSpecCmd;

pub fn handle_extcmd(extcmd: &ThSpecCmd) {
    match extcmd.name.as_str() {
        "declare-qvar" => {
            // TODO: parse and handle declare-qvar
            println!("[FOL] handle declare-qvar: {}", extcmd.args);
        }
        // other FOL-specific commands
        _ => {
            println!("[FOL] unknown command: {} {}", extcmd.name, extcmd.args);
        }
    }
} 