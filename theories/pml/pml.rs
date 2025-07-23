use crate::lang::model::ThSpecCmd;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum PmlCmd {
    DefineInit { name: String, states: Vec<String> },
    DefineLabel { name: String, expr: String },
    // more PML-specific commands
}

impl TryFrom<&ThSpecCmd> for PmlCmd {
    type Error = &'static str;
    fn try_from(cmd: &ThSpecCmd) -> Result<Self, Self::Error> {
        match cmd.name.as_str() {
            "define-init" => {
                let mut parts = cmd.args.splitn(2, ' ');
                let name = parts.next().ok_or("missing name")?.to_string();
                let states = parts
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c| c == '(' || c == ')')
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                Ok(PmlCmd::DefineInit { name, states })
            }
            "define-label" => {
                let mut parts = cmd.args.splitn(2, ' ');
                let name = parts.next().ok_or("missing name")?.to_string();
                let expr = parts.next().unwrap_or("").to_string();
                Ok(PmlCmd::DefineLabel { name, expr })
            }
            _ => Err("Unknown PML command"),
        }
    }
}

pub fn handle_extcmd(cmd: &dyn std::fmt::Debug) {
    if let Some(thcmd) = cmd.downcast_ref::<ThSpecCmd>() {
        match PmlCmd::try_from(thcmd) {
            Ok(pml_cmd) => {
                println!("[PML] structural command: {:?}", pml_cmd);
            }
            Err(_) => {
                println!("[PML] unknown command: {:?}", thcmd);
            }
        }
    } else {
        println!("[PML] not a ThSpecCmd: {:?}", cmd);
    }
} 