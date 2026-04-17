use crate::lang::model::{FuncInterp, InterpedValue, Model, Pattern, PatternElem};
use std::collections::HashMap;

pub fn pattern_match_symbols(tuple: &[String], pat: &Pattern) -> bool {
    if pat.elements.len() != tuple.len() {
        return false;
    }
    for (value, elem) in tuple.iter().zip(pat.elements.iter()) {
        match elem {
            PatternElem::UnivElem(e) if &e.name != value => return false,
            PatternElem::UnivElem(_) | PatternElem::Wildcard | PatternElem::Var(_) => {}
        }
    }
    true
}

pub fn eval_symbols<'a>(
    func_name: &str,
    args: &[String],
    func_interps: &'a [FuncInterp],
) -> Option<&'a InterpedValue> {
    if args.is_empty() {
        for interp in func_interps {
            if let FuncInterp::SimpleInterp(name, val) = interp {
                if name == func_name {
                    return Some(val);
                }
            }
        }
    }
    for interp in func_interps {
        if let FuncInterp::MatchInterp(name, cases) = interp {
            if name == func_name {
                for case in cases {
                    if let Some(pat) = &case.pattern {
                        if pattern_match_symbols(args, pat) {
                            return Some(&case.value);
                        }
                    }
                }
                for case in cases {
                    if let Some(pat) = &case.pattern {
                        if pat
                            .elements
                            .iter()
                            .all(|e| matches!(e, crate::lang::model::PatternElem::Wildcard))
                        {
                            return Some(&case.value);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_domain_for_sort(model: &Model, sort: &str) -> Option<Vec<String>> {
    let mut domain_syms = Vec::new();
    for univ in &model.univ_defs {
        if univ.sort.to_string() == sort {
            domain_syms.extend(univ.elems.iter().map(|elem| elem.name.clone()));
        }
    }
    if domain_syms.is_empty() {
        return None;
    }
    domain_syms.sort();
    Some(domain_syms)
}

pub fn element_belongs_to_sort(model: &Model, sort: &str, elem: &str) -> bool {
    get_domain_for_sort(model, sort)
        .map(|domain| domain.iter().any(|candidate| candidate == elem))
        .unwrap_or(false)
}

pub fn max_domain_len(model: &Model) -> usize {
    model
        .univ_defs
        .iter()
        .map(|univ| univ.elems.len())
        .max()
        .unwrap_or(0)
}

pub fn collect_loaded_models<'a>(
    all_model_names: &[String],
    models: &'a HashMap<String, &Model>,
) -> Vec<&'a Model> {
    all_model_names
        .iter()
        .filter_map(|n| models.get(n).copied())
        .collect()
}
