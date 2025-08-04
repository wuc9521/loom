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
pub enum ModelCtx {
    FOL(fol::FOModelCtx),
    PML(pml::KripkeCtx),
    RE(re::StringCtx),
}

pub fn dispatch_extcmd(logic: Logic, ctx: &mut ModelCtx, extcmd: &ThSpecCmd) {
    match (logic, ctx) {
        (Logic::FOL, ModelCtx::FOL(ref mut fol_ctx)) => fol::handle_model_extcmd(fol_ctx, extcmd),
        (Logic::PML, ModelCtx::PML(ref mut pml_ctx)) => pml::handle_model_extcmd(pml_ctx, extcmd),
        (Logic::RE, ModelCtx::RE(ref mut re_ctx)) => re::handle_model_extcmd(re_ctx, extcmd),
        _ => {}
    }
}
