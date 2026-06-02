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

/// Each model's universes are stored as `Arc<Vec<String>>` per sort so that
/// downstream lookups can hand callers a cheap reference-counted clone
/// instead of duplicating the element list. Cloning a `Vec<String>` of N
/// elements allocates N+1 times; cloning an Arc is one atomic increment.
/// On test_col this is hot — bisim's eval path calls `univ_for_sort` 15M+
/// times.
fn model_univ_cache() -> &'static Mutex<HashMap<u64, Arc<HashMap<String, Arc<Vec<String>>>>>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Arc<HashMap<String, Arc<Vec<String>>>>>>> =
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

fn build_model_univs(model: &Model) -> HashMap<String, Arc<Vec<String>>> {
    let mut buckets: HashMap<String, Vec<String>> = HashMap::default();
    for univ in &model.univ_defs {
        let entry = buckets
            .entry(univ.sort.to_string())
            .or_insert_with(Vec::new);
        entry.extend(univ.elems.iter().map(|elem| elem.name.clone()));
    }
    buckets
        .into_iter()
        .map(|(k, v)| (k, Arc::new(v)))
        .collect()
}

/// Returns the cached universe list for `sort` as a cheap `Arc<Vec<String>>`
/// clone. Used directly on the hot path; callers that want an owned
/// `Vec<String>` go through `Model::univ_for_sort`.
fn cached_univ_arc(model: &Model, sort: &str) -> Option<Arc<Vec<String>>> {
    let key = model.cache_id;
    let mut cache = model_univ_cache()
        .lock()
        .expect("model univ cache poisoned");
    let entry = cache
        .entry(key)
        .or_insert_with(|| Arc::new(build_model_univs(model)));
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

impl Model {
    pub fn eval_symbols(&self, func_name: &str, args: &[String]) -> Option<&InterpedValue> {
        let cache = interp_cache_for(self);
        let index = cache.by_name.get(func_name)?;
        match index {
            FuncInterpIndex::Simple { interp_idx } if args.is_empty() => {
                match &self.func_interps[*interp_idx] {
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
                let FuncInterp::MatchInterp(_, cases) = &self.func_interps[*interp_idx] else {
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

    /// Returns the universe element list for `sort`. Clones the cached
    /// `Vec<String>` — callers in hot loops should prefer `univ_contains` /
    /// `univ_element_at` to avoid this clone, or `univ_for_sort_arc` to
    /// share via reference counting.
    pub fn univ_for_sort(&self, sort: &str) -> Option<Vec<String>> {
        cached_univ_arc(self, sort).map(|arc| (*arc).clone())
    }

    /// Cheap shared borrow of the cached universe list (one atomic
    /// refcount bump per call).
    pub fn univ_for_sort_arc(&self, sort: &str) -> Option<Arc<Vec<String>>> {
        cached_univ_arc(self, sort)
    }

    /// Membership check without cloning the whole list.
    pub fn univ_contains(&self, sort: &str, elem: &str) -> bool {
        cached_univ_arc(self, sort)
            .map(|arc| arc.iter().any(|c| c == elem))
            .unwrap_or(false)
    }

    /// Returns the element at `idx` in the universe (cloned String, since
    /// the borrow can't outlive the cache lookup). One String alloc instead
    /// of N+1.
    pub fn univ_element_at(&self, sort: &str, idx: usize) -> Option<String> {
        cached_univ_arc(self, sort).and_then(|arc| arc.get(idx).cloned())
    }

    pub fn contains_element(&self, sort: &str, elem: &str) -> bool {
        self.univ_contains(sort, elem)
    }

    pub fn max_univ_len(&self) -> usize {
        self.univ_defs
            .iter()
            .map(|univ| univ.elems.len())
            .max()
            .unwrap_or(0)
    }
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
