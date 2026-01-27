use crate::parser::DelisProg;
use sygus_parser::SyGuSCmd;

pub fn to_sygus(lsfile: &DelisProg) -> Vec<SyGuSCmd> {
    if lsfile.cmds.is_empty() {
        return Vec::new();
    }
    return Vec::new();
}
