use crate::lang::model::ThSpecCmd;

#[derive(Debug, Default, Clone)]
pub struct RECtx {}

pub fn handle_extcmd(_ctx: &mut RECtx, extcmd: &ThSpecCmd) {
    match extcmd.name.as_str() {
        _ => {
            println!("[RE] unknown command: {} {}", extcmd.name, extcmd.args);
        }
    }
}
