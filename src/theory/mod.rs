    pub mod fol;
pub mod pml;
pub mod re;
pub mod utils;
use derive_more::Display;

use crate::lang::model::ThSpecCmd;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Logic {
    #[display(fmt = "FOL")]
    FOL,
    #[display(fmt = "PML")]
    PML,
    #[display(fmt = "BV")]
    BV,
    #[display(fmt = "RE")]
    RE,
    #[display(fmt = "CFE")]
    CFE,
    #[display(fmt = "SLIA")]
    SLIA,
}

#[derive(Debug, Clone)]
pub enum Ctx {
    FOL(fol::FOLCtx),
    PML(pml::PMLCtx),
    RE(re::RECtx),
}

pub fn dispatch_extcmd(logic: Logic, ctx: &mut Ctx, extcmd: &ThSpecCmd) {
    match (logic, ctx) {
        (Logic::FOL, Ctx::FOL(ref mut fol_ctx)) => fol::handle_extcmd(fol_ctx, extcmd),
        (Logic::PML, Ctx::PML(ref mut pml_ctx)) => pml::handle_extcmd(pml_ctx, extcmd),
        (Logic::RE, Ctx::RE(ref mut re_ctx)) => re::handle_extcmd(re_ctx, extcmd),
        _ => {}
    }
}
