use crate::lang::model::ThSpecCmd;

pub fn handle_extcmd(extcmd: &ThSpecCmd) {
    match extcmd.name.as_str() {
        "define-init" => {
            // TODO: parse and handle define-init
            println!("[PML] handle define-init: {}", extcmd.args);
        }
        "define-label" => {
            // TODO: parse and handle define-label
            println!("[PML] handle define-label: {}", extcmd.args);
        }
        // other PML-specific commands
        _ => {
            println!("[PML] unknown command: {} {}", extcmd.name, extcmd.args);
        }
    }
} 