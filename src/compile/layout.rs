//! Per-sort BV layout: one bit per predicate in `HornProg.predicates`.
//!
//! Pred order per sort is deterministic — keyed on a (group, declaration
//! index) tuple where group separates positive (q), negative (nq), and
//! helper (`__chc.*`) preds. Stable across runs for the same input.

use crate::backends::smt2::ast::{HornProg, PredicatePolarity};
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Clone)]
pub struct BitLayout {
    sort_widths: HashMap<String, usize>,
    bit_of_pred: HashMap<String, usize>,
}

impl BitLayout {
    pub fn from_horn_prog(prog: &HornProg) -> Self {
        let mut by_sort: HashMap<String, Vec<(u8, usize, String)>> = HashMap::default();
        for (idx, (name, info)) in prog.predicates.iter().enumerate() {
            if name == "Goal" {
                continue;
            }
            let helper = info.state == *name;
            let group: u8 = if helper {
                2
            } else {
                match info.polarity {
                    PredicatePolarity::Positive => 0,
                    PredicatePolarity::Negative => 1,
                }
            };
            by_sort
                .entry(info.sort.clone())
                .or_default()
                .push((group, idx, name.clone()));
        }
        let mut sort_widths = HashMap::default();
        let mut bit_of_pred = HashMap::default();
        for (sort, mut preds) in by_sort {
            preds.sort_by_key(|(g, i, _)| (*g, *i));
            for (sort_local, (_, _, name)) in preds.iter().enumerate() {
                bit_of_pred.insert(name.clone(), sort_local);
            }
            sort_widths.insert(sort, preds.len());
        }
        Self {
            sort_widths,
            bit_of_pred,
        }
    }

    pub fn width(&self, sort: &str) -> usize {
        self.sort_widths.get(sort).copied().unwrap_or(0)
    }

    pub fn bit_of(&self, predicate_name: &str) -> Option<usize> {
        self.bit_of_pred.get(predicate_name).copied()
    }

    pub fn bit_of_pred_for_debug(&self) -> &HashMap<String, usize> {
        &self.bit_of_pred
    }

    pub fn sort_widths(&self) -> impl Iterator<Item = (&str, usize)> {
        self.sort_widths
            .iter()
            .map(|(name, width)| (name.as_str(), *width))
    }
}
