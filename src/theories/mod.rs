pub mod fol;
pub mod pml;
pub mod re;

use crate::lang::model::ThSpecCmd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logic {
    FOL,
    PML,
    BV,
    RE,
    CFE,
}

pub fn dispatch_extcmd(logic: Logic, extcmd: &ThSpecCmd) {
    match logic {
        Logic::FOL => fol::handle_extcmd(extcmd),
        Logic::PML => pml::handle_extcmd(extcmd),
        Logic::RE => re::handle_extcmd(extcmd),
        _ => {}
    }
}
