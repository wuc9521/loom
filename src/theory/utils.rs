use std::collections::HashMap;
use crate::lang::model::{Pattern, PatternElem};

pub fn build_symbol_map(domain_syms: &[String]) -> (HashMap<String, usize>, Vec<String>) {
    let mut map = HashMap::new();
    let mut rev = vec![];
    for (i, sym) in domain_syms.iter().enumerate() {
        map.insert(sym.clone(), i);
        rev.push(sym.clone());
    }
    (map, rev)
}


pub fn pattern_match(tuple: &Vec<usize>, pat: &Pattern, id2sym: &Vec<String>) -> bool {
    if pat.elements.len() != tuple.len() { return false; }
    for (i, elem) in pat.elements.iter().enumerate() {
        match elem {
            PatternElem::UnivElem(e) => {
                if let Some(sym) = id2sym.get(tuple[i]) {
                    if &e.name != sym { return false; }
                } else {
                    return false;
                }
            }
            PatternElem::Wildcard => {}
            PatternElem::Var(_) => {}
        }
    }
    true
}
