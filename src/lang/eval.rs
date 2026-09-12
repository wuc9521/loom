use crate::lang::model::{FuncInterp, InterpedValue, Model, Pattern, PatternElem};
use crate::fxmap::HashMap;
use std::cell::RefCell;
use std::sync::Arc;

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

thread_local! {
    /// Per-thread memo of each model's parsed function-interpretation index,
    /// keyed by `Model::cache_id`. Thread-local (rather than a global
    /// `Mutex<HashMap>`) so the hot single-threaded bisim/compile path pays
    /// no atomic lock per `eval_symbols` call; other threads rebuild their
    /// own copy lazily, which is sound because the index is a pure function
    /// of the model's immutable `func_interps`.
    static INTERP_CACHE: RefCell<HashMap<u64, Arc<FuncInterpCache>>> =
        RefCell::new(HashMap::default());
    /// Per-thread memo of each model's per-sort universe lists (same keying
    /// and rationale as `INTERP_CACHE`).
    static UNIV_CACHE: RefCell<HashMap<u64, Arc<HashMap<String, Arc<Vec<String>>>>>> =
        RefCell::new(HashMap::default());
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
    INTERP_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| Arc::new(build_interp_cache(&model.func_interps)))
            .clone()
    })
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
    for sort_interp in &model.sort_interps {
        let entry = buckets
            .entry(sort_interp.sort.to_string())
            .or_insert_with(Vec::new);
        entry.extend(sort_interp.elems.iter().map(|elem| elem.name.clone()));
    }
    buckets
        .into_iter()
        .map(|(k, v)| (k, Arc::new(v)))
        .collect()
}

/// Returns the cached universe list for `sort` as a cheap `Arc<Vec<String>>`
/// clone. Used directly on the hot path; callers that want an owned
/// `Vec<String>` go through `Model::univ_for_sort`. Each model's universes
/// are stored as `Arc<Vec<String>>` per sort so lookups hand back a cheap
/// reference-counted clone instead of duplicating the element list (cloning
/// a `Vec<String>` of N elements allocates N+1 times; cloning an Arc is one
/// atomic increment). On test_col this is hot — bisim's eval path calls
/// `univ_for_sort` 15M+ times.
fn cached_univ_arc(model: &Model, sort: &str) -> Option<Arc<Vec<String>>> {
    let key = model.cache_id;
    UNIV_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| Arc::new(build_model_univs(model)))
            .get(sort)
            .cloned()
    })
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
    /// `Vec<String>` — callers in hot loops should prefer `univ_for_sort_arc`
    /// to share via reference counting.
    pub fn univ_for_sort(&self, sort: &str) -> Option<Vec<String>> {
        cached_univ_arc(self, sort).map(|arc| (*arc).clone())
    }

    /// Cheap shared borrow of the cached universe list (one atomic
    /// refcount bump per call).
    pub fn univ_for_sort_arc(&self, sort: &str) -> Option<Arc<Vec<String>>> {
        cached_univ_arc(self, sort)
    }

    pub fn max_univ_len(&self) -> usize {
        self.sort_interps
            .iter()
            .map(|sort_interp| sort_interp.elems.len())
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
