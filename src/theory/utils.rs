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

pub fn eval<'a>(
    func_name: &str,
    args: &[usize],
    func_interps: &'a [crate::lang::model::FuncInterp],
    id2sym: &Vec<String>,
) -> Option<&'a crate::lang::model::InterpedValue> {
    // 0-arity: SimpleInterp
    if args.is_empty() {
        for interp in func_interps {
            if let crate::lang::model::FuncInterp::SimpleInterp(name, val) = interp {
                if name == func_name {
                    return Some(val);
                }
            }
        }
    }
    // MatchInterp
    for interp in func_interps {
        if let crate::lang::model::FuncInterp::MatchInterp(name, cases) = interp {
            if name == func_name {
                for case in cases { // precise match
                    if let Some(pat) = &case.pattern {
                        if pattern_match(&args.to_vec(), pat, id2sym) {
                            return Some(&case.value);
                        }
                    }
                }
                for case in cases { // fallback (_)
                    if let Some(pat) = &case.pattern {
                        if pat.elements.iter().all(|e| matches!(e, crate::lang::model::PatternElem::Wildcard)) {
                            return Some(&case.value);
                        }
                    }
                }
            }
        }
    }
    None
}

/// 自动识别universe sort, 返回(domain_syms, id2sym, universe_sort)
pub fn get_universe_domain_and_map(model: &crate::lang::model::ModelDef) -> (Vec<String>, Vec<String>, String) {
    let universe_sort = model.univ_defs.get(0).map(|u| u.sort.to_string()).unwrap_or_else(|| "U".to_string());
    let mut domain_syms = vec![];
    for univ in &model.univ_defs {
        if univ.sort.to_string() == universe_sort {
            for elem in &univ.elems {
                domain_syms.push(elem.name.clone());
            }
        }
    }
    domain_syms.sort();
    let (_sym2id, id2sym) = build_symbol_map(&domain_syms);
    (domain_syms, id2sym, universe_sort)
}
