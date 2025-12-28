use crate::parser::DelisCmd;
use crate::parser::DelisFile;

pub fn extract_logic(problem: &DelisFile) -> Option<&str> {
    for cmd in &problem.cmds {
        if let DelisCmd::As(sygus_cmd) = cmd {
            if let sygus_parser::SyGuSCmd::SetLogic(ref logic) = sygus_cmd {
                return Some(logic.as_str());
            }
        }
    }
    None
}

pub fn extract_logic_sygus(sygus: &sygus_parser::SyGuSFile) -> Option<&str> {
    for cmd in &sygus.cmds {
        if let sygus_parser::SyGuSCmd::SetLogic(ref logic) = cmd {
            return Some(logic.as_str());
        }
    }
    None
}
