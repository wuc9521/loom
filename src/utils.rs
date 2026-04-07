use crate::theory::Logic;
use crate::parser::DelisCmd;
use crate::parser::DelisProg;

pub fn extract_logic(problem: &DelisProg) -> Option<&str> {
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

pub fn infer_logic(problem: &DelisProg) -> Option<Logic> {
    if let Some(logic) = extract_logic(problem) {
        match logic {
            "ALL" => {}
            "FOL" => return Some(Logic::FOL),
            "PML" => return Some(Logic::PML),
            "RE" => return Some(Logic::RE),
            "CFE" | "CFG" | "CFL" => return Some(Logic::CFE),
            _ => return None,
        }
    }

    for cmd in &problem.cmds {
        match cmd {
            DelisCmd::DeclareTermTypes(_) => return Some(Logic::RE),
            DelisCmd::DefineModel(model) => {
                for ext_cmd in &model.ext_cmds {
                    match ext_cmd.name.as_str() {
                        "set-init" | "set-tran" => return Some(Logic::PML),
                        "declare-qvar" => return Some(Logic::FOL),
                        _ => {}
                    }
                }
            }
            DelisCmd::As(sygus_parser::SyGuSCmd::SynthFun(_, _, sort, grammar)) => {
                if matches!(sort.to_string().as_str(), "CFG" | "CFE" | "CFL") {
                    return Some(Logic::CFE);
                }
                if sort.to_string() == "RegLan" {
                    return Some(Logic::RE);
                }
                if let Some(grammar) = grammar {
                    let grammar_text = grammar.to_string();
                    if grammar_text.contains("rhs") || grammar_text.contains("lhs") || grammar_text.contains("top") {
                        return Some(Logic::CFE);
                    }
                    if grammar_text.contains("modal.") {
                        return Some(Logic::PML);
                    }
                }
            }
            DelisCmd::As(sygus_parser::SyGuSCmd::Constraint(term)) => {
                let term_text = term.to_string();
                if term_text.contains("models") {
                    return Some(Logic::RE);
                }
            }
            _ => {}
        }
    }

    None
}
