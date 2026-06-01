//! Counter-example-guided inductive synthesis (CEGIS) strategy for the
//! SyGuS backend.  Solves a small subset of the full constraint set, asks
//! the caller for a candidate, finds which full constraints the candidate
//! fails on, then adaptively extends the active subset.
//!
//! The subset-extension policy is the only knob.  See [`CegisFeedback`] for
//! the three signals (√-batch, plateau backstop, fail-history priority) and
//! the reasoning behind them.

use delis::{compile, parser::DelisProg};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{io, time::Instant};
use sygus_parser::SyGuSCmd;

/// Drive the CEGIS loop.
///
/// `solve_subset(cmds)` must return either the decoded candidate string for
/// the given subset, or the literal `"infeasible"` if the subset has no
/// solution.  All solver-process / decode concerns live in the caller's
/// closure; this function just orchestrates the round structure and the
/// subset-extension policy.
pub fn cegis_solve<F>(
    delis_file: &DelisProg,
    seed_size: usize,
    full_cmds: &[SyGuSCmd],
    mut solve_subset: F,
) -> Result<String, io::Error>
where
    F: FnMut(&[SyGuSCmd]) -> Result<String, io::Error>,
{
    let seed_size = seed_size.max(1);
    let constraints = delis_file.constraints();
    let constraint_count = constraints.len();
    if constraint_count <= seed_size {
        return solve_subset(full_cmds);
    }

    let polarities = constraints
        .iter()
        .map(constraint_polarity)
        .collect::<Vec<_>>();
    let mut active = initial_cegis_constraints(&polarities, seed_size);
    let mut feedback = CegisFeedback::new();
    let mut round = 0usize;
    loop {
        round += 1;
        log::info!(
            "cegis round {}: {}/{} active",
            round,
            active.len(),
            constraint_count
        );
        let round_start = Instant::now();
        let subset_cmds = subset_sygus_problem(full_cmds, &active, constraint_count);
        log::debug!(
            "cegis round {}: subset built in {:?}",
            round,
            round_start.elapsed()
        );
        let decoded = solve_subset(&subset_cmds)?;
        log::debug!(
            "cegis round {}: raw output = {}",
            round,
            decoded.chars().take(300).collect::<String>()
        );
        if decoded.trim() == "infeasible" {
            log::info!("cegis round {}: subset infeasible", round);
            return Ok("infeasible".to_string());
        }
        let failing =
            compile::failing_constraint_indices(full_cmds, &decoded).map_err(io::Error::other)?;
        let failing: Vec<usize> = failing
            .into_iter()
            .filter(|&idx| idx < constraint_count)
            .collect();
        if failing.is_empty() {
            log::info!("cegis round {}: solved", round);
            return Ok(decoded);
        }
        let added = feedback.extend_active(&mut active, &failing);
        log::info!(
            "cegis round {}: {} failed, +{}",
            round,
            failing.len(),
            added
        );
        if added == 0 {
            return Err(io::Error::other(
                "CEGIS made no progress while verification still fails",
            ));
        }
    }
}

/// Adaptive CEGIS feedback state.
///
/// Replaces the old "add 1 positive + 1 negative per round" rule with three
/// principled signals:
///   P1 — batch size = ⌈√|failing|⌉.  Strong candidate (few failing) -> small
///        batch, each constraint does precise work.  Weak candidate (many
///        failing) -> big batch injected, so the synthesizer stops picking
///        cheap candidates that don't generalize.  Purely a function of the
///        observed signal, not an offline heuristic about model size.
///   P2 — stagnation backstop.  If two consecutive rounds make ≤10% progress
///        on the failing count, double the batch multiplier (capped at 16).
///        Resets on any real progress.  On by default — no flag.
///   P3 — within the batch, prioritise constraints by `fail_history` (times
///        they have failed across past rounds).  Constraints that keep
///        failing are structurally hard and most likely essential.
struct CegisFeedback {
    fail_history: HashMap<usize, usize>,
    prev_failing: Option<usize>,
    plateau_streak: usize,
    batch_multiplier: usize,
}

impl CegisFeedback {
    fn new() -> Self {
        Self {
            fail_history: HashMap::default(),
            prev_failing: None,
            plateau_streak: 0,
            batch_multiplier: 1,
        }
    }

    fn batch_size(&self, failing_count: usize) -> usize {
        if failing_count == 0 {
            return 0;
        }
        let base = (failing_count as f64).sqrt().ceil() as usize;
        base.saturating_mul(self.batch_multiplier)
            .max(1)
            .min(failing_count)
    }

    fn select(&self, failing: &[usize], batch: usize, active: &HashSet<usize>) -> Vec<usize> {
        let mut candidates: Vec<usize> = failing
            .iter()
            .copied()
            .filter(|c| !active.contains(c))
            .collect();
        candidates.sort_by_key(|c| std::cmp::Reverse(*self.fail_history.get(c).unwrap_or(&0)));
        candidates.into_iter().take(batch).collect()
    }

    fn extend_active(&mut self, active: &mut HashSet<usize>, failing: &[usize]) -> usize {
        let batch = self.batch_size(failing.len());
        let chosen = self.select(failing, batch, active);
        let added = chosen.len();
        for c in chosen {
            active.insert(c);
        }
        self.update_after_round(failing);
        added
    }

    fn update_after_round(&mut self, failing: &[usize]) {
        for &c in failing {
            *self.fail_history.entry(c).or_default() += 1;
        }
        let curr = failing.len();
        if let Some(prev) = self.prev_failing {
            if prev > 0 && (curr as f64) / (prev as f64) > 0.9 {
                self.plateau_streak += 1;
                if self.plateau_streak >= 2 {
                    let next = self.batch_multiplier.saturating_mul(2).min(16);
                    if next != self.batch_multiplier {
                        log::info!(
                            "cegis: plateau detected ({} -> {} failing for 3 rounds); batch_multiplier {} -> {}",
                            prev, curr, self.batch_multiplier, next
                        );
                    }
                    self.batch_multiplier = next;
                    self.plateau_streak = 0;
                }
            } else {
                self.plateau_streak = 0;
                self.batch_multiplier = 1;
            }
        }
        self.prev_failing = Some(curr);
    }
}

fn initial_cegis_constraints(polarities: &[Option<bool>], seed_size: usize) -> HashSet<usize> {
    let seed_size = seed_size.max(1);
    let mut active = HashSet::default();
    let target_pos = seed_size.div_ceil(2);
    let target_neg = seed_size / 2;
    for (target, class_target) in [(Some(true), target_pos), (Some(false), target_neg)] {
        let mut class_count = 0usize;
        for (idx, polarity) in polarities.iter().enumerate() {
            if *polarity == target && !active.contains(&idx) {
                active.insert(idx);
                class_count += 1;
                if active.len() >= seed_size {
                    return active;
                }
                if class_count >= class_target {
                    break;
                }
            }
        }
    }
    for idx in 0..polarities.len() {
        if active.len() >= seed_size {
            break;
        }
        active.insert(idx);
    }
    active
}

fn subset_sygus_problem(
    cmds: &[SyGuSCmd],
    active: &HashSet<usize>,
    constraint_count: usize,
) -> Vec<SyGuSCmd> {
    let mut reduced = Vec::with_capacity(cmds.len());
    let mut constraint_idx = 0usize;
    for cmd in cmds {
        match cmd {
            SyGuSCmd::Constraint(_) if constraint_idx < constraint_count => {
                if active.contains(&constraint_idx) {
                    reduced.push(cmd.clone());
                }
                constraint_idx += 1;
            }
            _ => reduced.push(cmd.clone()),
        }
    }
    reduced
}

fn constraint_polarity(term: &sygus_parser::SyGuSTerm) -> Option<bool> {
    constraint_expected_truth(term, false)
}

fn constraint_expected_truth(term: &sygus_parser::SyGuSTerm, negated: bool) -> Option<bool> {
    if let sygus_parser::SyGuSTerm::Application(id, args) = term {
        if id.get_name() == "not" && args.len() == 1 {
            return constraint_expected_truth(&args[0], !negated);
        }
        if id.get_name() == "=" && args.len() == 2 {
            if let Some(value) = binary_truth_value(&args[0]) {
                return constraint_expected_truth(&args[1], negated ^ !value);
            }
            if let Some(value) = binary_truth_value(&args[1]) {
                return constraint_expected_truth(&args[0], negated ^ !value);
            }
        }
    }
    Some(!negated)
}

fn binary_truth_value(term: &sygus_parser::SyGuSTerm) -> Option<bool> {
    match term {
        sygus_parser::SyGuSTerm::Literal(sygus_parser::Literal::Bool(value)) => Some(*value),
        sygus_parser::SyGuSTerm::Literal(sygus_parser::Literal::BinConst(bits)) => match bits
            .as_str()
        {
            "#b1" => Some(true),
            "#b0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sygus_parser::ast::{Identifier, Literal, SyGuSTerm};

    fn app(name: &str, args: Vec<SyGuSTerm>) -> SyGuSTerm {
        SyGuSTerm::Application(Identifier::from_str(name).unwrap(), args)
    }

    fn bool_lit(value: bool) -> SyGuSTerm {
        SyGuSTerm::Literal(Literal::Bool(value))
    }

    fn bin_lit(bits: &str) -> SyGuSTerm {
        SyGuSTerm::Literal(Literal::BinConst(bits.into()))
    }

    fn atom(name: &str) -> SyGuSTerm {
        app(name, vec![])
    }

    #[test]
    fn constraint_polarity_handles_nested_negation_and_truth_equalities() {
        assert_eq!(constraint_polarity(&atom("p")), Some(true));
        assert_eq!(
            constraint_polarity(&app("not", vec![atom("p")])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("not", vec![app("not", vec![atom("p")])])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bool_lit(true)])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bool_lit(false)])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bin_lit("#b1")])),
            Some(true)
        );
        assert_eq!(
            constraint_polarity(&app("=", vec![atom("p"), bin_lit("#b0")])),
            Some(false)
        );
        assert_eq!(
            constraint_polarity(&app("not", vec![app("=", vec![atom("p"), bin_lit("#b0")])])),
            Some(true)
        );
    }
}
