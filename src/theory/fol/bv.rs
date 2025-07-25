use crate::parser::LSFile;
use sygus_parser::SyGuSCmd;

pub fn to_sygus(lsfile: &LSFile) -> Vec<SyGuSCmd> {
    if lsfile.cmds.is_empty() {
        return Vec::new();
    }
    return Vec::new();
}
