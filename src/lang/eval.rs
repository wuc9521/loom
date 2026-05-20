use crate::lang::model::{FuncInterp, InterpedValue, Model, Pattern, PatternElem};
use rustc_hash::FxHashMap as HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone)]
enum FuncInterpIndex {
    Simple {
        interp_idx: usize,
    },
    Match {
        interp_idx: usize,
        default_case_idx: Option<usize>,
        exact_case_idx: HashMap<Vec<String>, usize>,
    },
}

#[derive(Default)]
struct FuncInterpCache {
    by_name: HashMap<String, FuncInterpIndex>,
}

#[derive(Clone)]
struct CachedFuncInterp {
    cache: Arc<FuncInterpCache>,
}

fn interp_cache() -> &'static Mutex<HashMap<u64, CachedFuncInterp>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, CachedFuncInterp>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::default()))
}

fn model_domain_cache() -> &'static Mutex<HashMap<u64, Arc<HashMap<String, Vec<String>>>>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Arc<HashMap<String, Vec<String>>>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::default()))
}

fn build_interp_cache(func_interps: &[FuncInterp]) -> FuncInterpCache {
    let mut by_name = HashMap::default();
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
                let exact_case_idx = cases
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, case)| {
                        case.pattern
                            .as_ref()
                            .and_then(pattern_as_exact_tuple)
                            .map(|tuple| (tuple, idx))
                    })
                    .collect::<HashMap<_, _>>();
                by_name.insert(
                    name.clone(),
                    FuncInterpIndex::Match {
                        interp_idx,
                        default_case_idx,
                        exact_case_idx,
                    },
                );
            }
        }
    }
    FuncInterpCache { by_name }
}

fn interp_cache_for(model: &Model) -> Arc<FuncInterpCache> {
    let key = model.cache_id;
    let mut cache = interp_cache().lock().expect("interp cache poisoned");
    let entry = cache.entry(key).or_insert_with(|| CachedFuncInterp {
        cache: Arc::new(build_interp_cache(&model.func_interps)),
    });
    entry.cache.clone()
}

fn pattern_as_exact_tuple(pat: &Pattern) -> Option<Vec<String>> {
    pat.elements
        .iter()
        .map(|elem| match elem {
            PatternElem::UnivElem(e) => Some(e.name.clone()),
            PatternElem::Wildcard | PatternElem::Var(_) => None,
        })
        .collect()
}

fn build_model_domains(model: &Model) -> HashMap<String, Vec<String>> {
    let mut domains = HashMap::default();
    for univ in &model.univ_defs {
        let entry = domains
            .entry(univ.sort.to_string())
            .or_insert_with(Vec::new);
        entry.extend(univ.elems.iter().map(|elem| elem.name.clone()));
    }
    domains
}

fn cached_domain_for_sort(model: &Model, sort: &str) -> Option<Vec<String>> {
    let key = model.cache_id;
    let mut cache = model_domain_cache()
        .lock()
        .expect("model domain cache poisoned");
    let entry = cache
        .entry(key)
        .or_insert_with(|| Arc::new(build_model_domains(model)));
    entry.get(sort).cloned()
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
    model: &'a Model,
) -> Option<&'a InterpedValue> {
    let cache = interp_cache_for(model);
    let index = cache.by_name.get(func_name)?;
    match index {
        FuncInterpIndex::Simple { interp_idx } if args.is_empty() => {
            match &model.func_interps[*interp_idx] {
                FuncInterp::SimpleInterp(_, val) => Some(val),
                FuncInterp::MatchInterp(..) => None,
            }
        }
        FuncInterpIndex::Simple { .. } => None,
        FuncInterpIndex::Match {
            interp_idx,
            default_case_idx,
            exact_case_idx,
        } => {
            let FuncInterp::MatchInterp(_, cases) = &model.func_interps[*interp_idx] else {
                return None;
            };
            if let Some(case_idx) = exact_case_idx.get(args) {
                return cases.get(*case_idx).map(|case| &case.value);
            }
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
