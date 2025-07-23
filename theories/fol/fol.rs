use crate::lang::model::ThSpecCmd;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct QVarSig {
    pub name: String,
    pub sort: String,
}

#[derive(Debug)]
pub enum FolCmd {
    DeclareQVar(QVarSig),
}

impl TryFrom<&ThSpecCmd> for FolCmd {
    type Error = &'static str;
    fn try_from(cmd: &ThSpecCmd) -> Result<Self, Self::Error> {
        match cmd.name.as_str() {
            "declare-qvar" => {
                let mut parts = cmd.args.split_whitespace();
                let name = parts.next().ok_or("missing name")?.to_string();
                let sort = parts.next().ok_or("missing sort")?.to_string();
                Ok(FolCmd::DeclareQVar(QVarSig { name, sort }))
            }
            _ => Err("Unknown FOL command"),
        }
    }
}

pub fn handle_extcmd(cmd: &dyn std::fmt::Debug) {
    if let Some(thcmd) = cmd.downcast_ref::<ThSpecCmd>() {
        match FolCmd::try_from(thcmd) {
            Ok(fol_cmd) => {
                println!("[FOL] structural command: {:?}", fol_cmd);
            }
            Err(_) => {
                println!("[FOL] unknown command: {:?}", thcmd);
            }
        }
    } else {
        println!("[FOL] not a ThSpecCmd: {:?}", cmd);
    }
} 