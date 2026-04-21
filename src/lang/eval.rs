use crate::lang::model::{FuncInterp, InterpedValue, Model, Pattern, PatternElem};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SliceKey {
    ptr: usize,
    len: usize,
}

#[derive(Clone, Copy)]
enum FuncInterpIndex {
    Simple {
        interp_idx: usize,
    },
    Match {
        interp_idx: usize,
        default_case_idx: Option<usize>,
    },
}

#[derive(Default, Clone)]
struct FuncInterpCache {
    by_name: HashMap<String, FuncInterpIndex>,
}

fn interp_cache() -> &'static Mutex<HashMap<SliceKey, FuncInterpCache>> {
    static CACHE: OnceLock<Mutex<HashMap<SliceKey, FuncInterpCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn domain_cache() -> &'static Mutex<HashMap<usize, HashMap<String, Vec<String>>>> {
    static CACHE: OnceLock<Mutex<HashMap<usize, HashMap<String, Vec<String>>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn slice_key(func_interps: &[FuncInterp]) -> SliceKey {
    SliceKey {
        ptr: func_interps.as_ptr() as usize,
        len: func_interps.len(),
    }
}

fn build_interp_cache(func_interps: &[FuncInterp]) -> FuncInterpCache {
    let mut by_name = HashMap::new();
    for (interp_idx, interp) in func_interps.iter().enumerate() {
        match interp {
            FuncInterp::SimpleInterp(name, _) => {
                by_name.insert(name.clone(), FuncInterpIndex::Simple { interp_idx });
            }
            FuncInterp::MatchInterp(name, cases) => {
                let default_case_idx = cases.iter().position(|case| {
                    case.pattern.as_ref().is_some_and(|pat| {
                        pat.elements
                            .iter()
                            .all(|elem| matches!(elem, PatternElem::Wildcard))
                    })
                });
                by_name.insert(
                    name.clone(),
                    FuncInterpIndex::Match {
                        interp_idx,
                        default_case_idx,
                    },
                );
            }
        }
    }
    FuncInterpCache { by_name }
}

fn interp_cache_for(func_interps: &[FuncInterp]) -> FuncInterpCache {
    let key = slice_key(func_interps);
    let mut cache = interp_cache().lock().expect("interp cache poisoned");
    cache
        .entry(key)
        .or_insert_with(|| build_interp_cache(func_interps))
        .clone()
}

fn build_model_domains(model: &Model) -> HashMap<String, Vec<String>> {
    let mut domains = HashMap::new();
    for univ in &model.univ_defs {
        let entry = domains
            .entry(univ.sort.to_string())
            .or_insert_with(Vec::new);
        entry.extend(univ.elems.iter().map(|elem| elem.name.clone()));
    }
    for values in domains.values_mut() {
        values.sort();
    }
    domains
}

fn cached_domain_for_sort(model: &Model, sort: &str) -> Option<Vec<String>> {
    let key = model as *const Model as usize;
    let mut cache = domain_cache().lock().expect("domain cache poisoned");
    cache
        .entry(key)
        .or_insert_with(|| build_model_domains(model))
        .get(sort)
        .cloned()
}

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
    let cache = interp_cache_for(func_interps);
    let index = cache.by_name.get(func_name)?;
    match index {
        FuncInterpIndex::Simple { interp_idx } if args.is_empty() => {
            match &func_interps[*interp_idx] {
                FuncInterp::SimpleInterp(_, val) => Some(val),
                FuncInterp::MatchInterp(..) => None,
            }
        }
        FuncInterpIndex::Simple { .. } => None,
        FuncInterpIndex::Match {
            interp_idx,
            default_case_idx,
        } => {
            let FuncInterp::MatchInterp(_, cases) = &func_interps[*interp_idx] else {
                return None;
            };
            for case in cases {
                if let Some(pat) = &case.pattern {
                    if pattern_match_symbols(args, pat) {
                        return Some(&case.value);
                    }
                }
            }
            default_case_idx.and_then(|idx| cases.get(idx).map(|case| &case.value))
        }
    }
}

pub fn get_domain_for_sort(model: &Model, sort: &str) -> Option<Vec<String>> {
    cached_domain_for_sort(model, sort)
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
