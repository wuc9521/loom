use crate::parser::LSFile;
use crate::parser::LSCmd;

pub fn extract_logic(problem: &LSFile) -> Option<&str> {
    for cmd in &problem.cmds {
        if let LSCmd::As(sygus_cmd) = cmd {
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