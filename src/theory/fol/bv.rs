use crate::parser::DelisFile;
use sygus_parser::SyGuSCmd;

pub fn to_sygus(lsfile: &DelisFile) -> Vec<SyGuSCmd> {
    if lsfile.cmds.is_empty() {
        return Vec::new();
    }
    return Vec::new();
}
