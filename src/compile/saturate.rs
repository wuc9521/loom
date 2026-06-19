//! Saturation for the per-bit Horn CHC IR.
//!
//! Given a Horn program with push rules (`head.arg = cons(...)`) and pop
//! rules (one body atom has `arg = cons(...)`), saturation closes the two
//! directions by repeated resolution. Each push×pop resolution unifies the
//! push head's constructor pattern with the pop body atom, substitutes the
//! head's variables in the push body, and concatenates that into the pop
//! rule's body in place of the pop atom. Push rules are kept; pop rules
//! are intermediate. The end result is a closed set of one-way push rules
//! whose bodies reference only variables that appear in their heads.
//!
//! This is the standard one-way conversion for two-way alternating tree
//! automata (Vardi 1998; Comon et al. 2008 §7.6.3) applied at the Horn IR
//! level. The saturated rules emit cleanly as `define-fun` bodies — every
//! constructor's per-bit semantics becomes a closed Boolean formula in its
//! children's bits, with no aux synth-fun and no closure constraint.
//!
//! ## Body atom roles
//!
//! Each body atom is classified once at index time (see [`AtomRole`]):
//!
//! - [`AtomRole::Pop`]: contains a variable not bound by the head — comes
//!   from `parent_contexts` lowering (the fresh `__up<n>` slots of a parent
//!   constructor pattern). Saturation MUST eliminate it by resolving.
//! - [`AtomRole::Intersection`]: every variable is already bound by the
//!   head. These are the helper-pred (`__chc.*`) clauses from
//!   `chc.rs::emit_helper_atom` — 1-way *alternating* TA transitions in
//!   Comon's §7.6.1 sense. Resolving them would distribute an AND-of-OR
//!   into DNF, which is the source of the 2^K blowup. We never resolve them.
//!
//! All resolution decisions flow from `body_atom_role`: only `Pop` atoms
//! enter `pop_index`, so no other site needs to know about the distinction.
//!
//! ## Other corrections over earlier in-tree attempts
//!  - Push body variables that don't appear in the head's cons pattern are
//!    alpha-renamed to fresh names per resolution, so they can't capture
//!    a same-named variable that happens to live in the pop rule.
//!  - Subsumption / dedup uses an alpha-normalized rule key, so two rules
//!    equivalent up to renaming collapse to one (otherwise the worklist
//!    may not terminate).

use crate::backends::smt2::ast::{AtomRole, HornAtom, HornProg, HornRule, HornTerm};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Saturate `prog.rules` in place. Pop rules are eliminated; closed push
/// rules (body all-vars referencing only head-vars) remain.
pub fn saturate(prog: &mut HornProg) {
    let saturated = saturate_rules(&prog.rules);
    prog.rules = saturated;
}

/// Saturate a rule list and return the closed push rules. Public for tests.
pub fn saturate_rules(input: &[HornRule]) -> Vec<HornRule> {
    saturate_rules_with(input, SaturateConfig::from_env())
}

/// Like `saturate_rules` but with explicit configuration. Tests pass
/// `SaturateConfig::default()`; the binary path constructs from env.
pub fn saturate_rules_with(input: &[HornRule], config: SaturateConfig) -> Vec<HornRule> {
    let mut state = SaturationState::new(input, config);
    state.run();
    state.finish()
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// External knobs for a saturation run. Read once at entry from
/// `DELIS_SATURATE_*` env vars (binary path) or constructed via
/// `Default::default()` (tests).
#[derive(Clone, Debug, Default)]
pub struct SaturateConfig {
    /// Bail out and dump diagnostics when the rule store reaches this size.
    /// `None` = no cap.
    pub cap: Option<usize>,
    /// On cap-hit, write a sample of saturated rules here (one rule per line,
    /// with body atoms tagged by [`AtomRole`]).
    pub dump_path: Option<PathBuf>,
}

impl SaturateConfig {
    pub fn from_env() -> Self {
        Self {
            cap: std::env::var("DELIS_SATURATE_CAP")
                .ok()
                .and_then(|s| s.parse().ok()),
            dump_path: std::env::var("DELIS_SATURATE_DUMP").ok().map(PathBuf::from),
        }
    }
}

// ---------------------------------------------------------------------------
// Diagnostic counters
// ---------------------------------------------------------------------------

/// Per-run counters. Centralised here so adding/removing a stat is a
/// one-place change and `log_row` is the single owner of the print format.
#[derive(Default, Debug, Clone)]
pub struct SaturateStats {
    /// `try_resolve` invocations.
    pub attempts: usize,
    /// Attempts that failed at unification (head pattern didn't fit pop atom).
    pub unify_fail: usize,
    /// Attempts that produced a rule that passed dedup + subsumption.
    pub kept: usize,
    /// Attempts whose alpha-key matched an existing rule.
    pub dup: usize,
    /// Attempts whose resolved rule was subsumed by some existing rule.
    pub subsumed_in: usize,
    /// Body atoms classified `Intersection` at index time — i.e. excluded
    /// from `pop_index` to prevent the AND-of-OR → DNF blowup. A healthy
    /// run on a `(parent n)`-using spec has this far larger than `attempts`.
    pub intersections_protected: usize,
}

impl SaturateStats {
    fn log_row(&self, label: &str, rules: usize, active: usize, seen: usize, pending: usize) {
        eprintln!(
            "[saturate {label}] rules={rules} (active={active}) seen={seen} pending={pending} \
             | attempts={} unify_fail={} kept={} dup={} subsumed_in={} intr_protected={}",
            self.attempts,
            self.unify_fail,
            self.kept,
            self.dup,
            self.subsumed_in,
            self.intersections_protected,
        );
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

struct SaturationState {
    /// All rules ever produced. `active[i] == false` means subsumed/removed.
    rules: Vec<HornRule>,
    active: Vec<bool>,
    /// Alpha-normalized keys of every active rule, for dedup.
    seen: HashSet<RuleKey>,
    /// Worklist of rule ids still to process.
    pending: VecDeque<usize>,
    /// Constructor key (pred, fun, arity) → list of active push rule ids
    /// whose head matches.
    push_index: HashMap<CtorKey, Vec<usize>>,
    /// Constructor key → list of (rule_id, body_idx) where a **Pop** atom
    /// matches. `Intersection` atoms are never inserted here, so resolution
    /// can't reach them.
    pop_index: HashMap<CtorKey, Vec<(usize, usize)>>,
    /// Head atom → ids of every active rule with that head. Used by
    /// subsumption to find candidates with matching head only.
    by_head: HashMap<HornAtom, Vec<usize>>,
    /// Counter for fresh variable names.
    fresh_counter: usize,
    /// All counters in one place.
    stats: SaturateStats,
    /// Read-only knobs.
    config: SaturateConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CtorKey {
    pred: String,
    fun: String,
    arity: usize,
}

/// Alpha-normalized rule shape: rename every variable to its first-occurrence
/// ordinal in a deterministic walk of (head, sorted body). Two rules with the
/// same alpha-key are equivalent up to renaming.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RuleKey {
    head: HornAtom,
    body: Vec<HornAtom>,
}

impl SaturationState {
    fn new(input: &[HornRule], config: SaturateConfig) -> Self {
        let mut state = Self {
            rules: Vec::with_capacity(input.len()),
            active: Vec::with_capacity(input.len()),
            seen: HashSet::default(),
            pending: VecDeque::new(),
            push_index: HashMap::default(),
            pop_index: HashMap::default(),
            by_head: HashMap::default(),
            fresh_counter: 0,
            stats: SaturateStats::default(),
            config,
        };
        for rule in input {
            state.insert(rule.clone());
        }
        state
    }

    /// Add a rule if it's not already present (modulo alpha) AND no existing
    /// rule with the same head has a body that is a subset of this one
    /// (i.e., an existing rule that subsumes this one). If accepted, also
    /// deactivate any existing rules that the new rule subsumes.
    fn insert(&mut self, rule: HornRule) -> Option<usize> {
        let key = alpha_normalize(&rule);
        if self.seen.contains(&key) {
            return None;
        }
        // Subsumption check: is there an active rule with same head whose
        // body is a subset of the new rule's body? If so, drop the new rule.
        let new_body_set: HashSet<&HornAtom> = rule.body_atoms.iter().collect();
        if let Some(ids) = self.by_head.get(&rule.head) {
            for &existing_id in ids {
                if !self.active[existing_id] {
                    continue;
                }
                let existing = &self.rules[existing_id];
                if existing
                    .body_atoms
                    .iter()
                    .all(|a| new_body_set.contains(a))
                {
                    return None;
                }
            }
        }
        // Mark every existing-rule-with-same-head whose body is a SUPERSET
        // of the new rule's body as inactive (the new rule subsumes them).
        // We iterate a snapshot to avoid borrowing conflicts.
        if let Some(ids) = self.by_head.get(&rule.head).cloned() {
            let new_body_atoms = rule.body_atoms.clone();
            for existing_id in ids {
                if !self.active[existing_id] {
                    continue;
                }
                let existing_body: HashSet<&HornAtom> =
                    self.rules[existing_id].body_atoms.iter().collect();
                if new_body_atoms
                    .iter()
                    .all(|a| existing_body.contains(a))
                {
                    self.active[existing_id] = false;
                    let existing_key = alpha_normalize(&self.rules[existing_id]);
                    self.seen.remove(&existing_key);
                }
            }
        }
        self.seen.insert(key);
        let id = self.rules.len();
        // Index into `by_head` immediately (subsumption needs to see this
        // rule even while it's still pending). `push_index` / `pop_index`
        // are populated only when the rule transitions to "active partner
        // for resolution" via `mark_active(id)` — given-clause style, so
        // each (push, pop) pair is tried exactly once.
        self.by_head.entry(rule.head.clone()).or_default().push(id);
        self.rules.push(rule);
        self.active.push(true);
        self.pending.push_back(id);
        Some(id)
    }

    /// Add `id` to `push_index` / `pop_index`. Called when the rule is
    /// popped from the worklist and starts participating as a resolution
    /// partner. **Intersection** body atoms (all vars bound by the head)
    /// are deliberately NOT inserted into `pop_index` — they are
    /// alternating clauses in Comon's §7.6.1 sense and must stay put.
    fn mark_active_partner(&mut self, id: usize) {
        let rule = &self.rules[id];
        if let Some(k) = ctor_key(&rule.head) {
            self.push_index.entry(k).or_default().push(id);
        }
        let head_vars = rule.head_vars();
        for (idx, atom) in rule.body_atoms.iter().enumerate() {
            let Some(k) = ctor_key(atom) else { continue };
            match atom.role_against(&head_vars) {
                AtomRole::Pop => {
                    self.pop_index.entry(k).or_default().push((id, idx));
                }
                AtomRole::Intersection => {
                    self.stats.intersections_protected += 1;
                }
            }
        }
    }

    fn run(&mut self) {
        let mut last_log = 0usize;
        let cap = self.config.cap.unwrap_or(usize::MAX);
        while let Some(id) = self.pending.pop_front() {
            if !self.active[id] {
                continue;
            }
            // Resolve `id` against currently-indexed (= already-processed)
            // partners, THEN index `id` so future-popped rules can see it.
            // This way each (push, pop) pair is attempted exactly once,
            // by whichever rule was popped last.
            self.resolve_with_partners(id);
            self.mark_active_partner(id);
            if self.rules.len() >= last_log + 1000 {
                self.log_progress("tick");
                last_log = self.rules.len();
            }
            if self.rules.len() >= cap {
                eprintln!("[saturate] hit cap={cap}; bailing");
                self.log_progress("cap-hit");
                self.write_dump();
                break;
            }
        }
    }

    fn log_progress(&self, label: &str) {
        let active: usize = self.active.iter().filter(|a| **a).count();
        self.stats
            .log_row(label, self.rules.len(), active, self.seen.len(), self.pending.len());
    }

    /// On cap-hit, write a sample of late-indexed active rules to
    /// `config.dump_path`. Each rule's body atom is tagged by its
    /// [`AtomRole`] so you can see at a glance which atoms saturation
    /// would have resolved (`pop`) vs. left alone (`intr`).
    fn write_dump(&self) {
        let Some(path) = &self.config.dump_path else {
            return;
        };
        use std::io::Write;
        let Ok(mut f) = std::fs::File::create(path) else {
            return;
        };
        let from = self.rules.len().saturating_sub(60);
        let mut written = 0;
        for (i, (rule, active)) in self
            .rules
            .iter()
            .zip(self.active.iter())
            .enumerate()
            .skip(from)
        {
            if !*active {
                continue;
            }
            let _ = writeln!(f, "{i}: {}", format_rule_with_roles(rule));
            written += 1;
            if written >= 30 {
                break;
            }
        }
    }

    fn resolve_with_partners(&mut self, id: usize) {
        let rule = self.rules[id].clone();

        // (a) If `id` is a push rule, look at every pop site whose ctor
        // matches. `pop_index` only contains `Pop` atoms, so intersection
        // atoms are skipped by construction.
        if let Some(head_key) = ctor_key(&rule.head) {
            if let Some(pops) = self.pop_index.get(&head_key).cloned() {
                for (pop_id, body_idx) in pops {
                    if !self.active[pop_id] {
                        continue;
                    }
                    self.try_resolve(pop_id, body_idx, id);
                }
            }
        }

        // (b) If `id` has pop body atoms, look at every push rule whose
        // head matches. Intersection body atoms never reach this branch.
        let head_vars = rule.head_vars();
        for (body_idx, atom) in rule.body_atoms.iter().enumerate() {
            let Some(k) = ctor_key(atom) else { continue };
            if atom.role_against(&head_vars) != AtomRole::Pop {
                continue;
            }
            let Some(pushes) = self.push_index.get(&k).cloned() else {
                continue;
            };
            for push_id in pushes {
                if !self.active[push_id] {
                    continue;
                }
                // Self-resolution is allowed; the alpha-renaming in
                // `resolve` handles capture.
                self.try_resolve(id, body_idx, push_id);
            }
        }
    }

    /// Try to resolve `pop_rule.body[body_idx]` against `push_rule.head`.
    /// On success, add the resolved rule to the worklist.
    fn try_resolve(&mut self, pop_id: usize, body_idx: usize, push_id: usize) {
        self.stats.attempts += 1;
        let pop_rule = &self.rules[pop_id];
        let push_rule = &self.rules[push_id];
        let Some(resolved) = resolve(pop_rule, body_idx, push_rule, &mut self.fresh_counter)
        else {
            self.stats.unify_fail += 1;
            return;
        };
        let key = alpha_normalize(&resolved);
        if self.seen.contains(&key) {
            self.stats.dup += 1;
            return;
        }
        if self.insert(resolved).is_some() {
            self.stats.kept += 1;
        } else {
            self.stats.subsumed_in += 1;
        }
    }



    fn finish(self) -> Vec<HornRule> {
        self.rules
            .into_iter()
            .zip(self.active)
            .filter(|(rule, active)| {
                *active
                    && !rule.is_pop_clause()
                    && matches!(&rule.head.arg, HornTerm::App { .. })
            })
            .map(|(rule, _)| rule)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Resolution primitive
// ---------------------------------------------------------------------------

/// Compute the matcher `push.head.cons-args → pop.body[idx].cons-args`. The
/// patterns must be a constructor app on both sides with the same fun and arity.
/// Push args are *patterns* (assumed to be all `Var`s, which is how
/// `parent_contexts` and the chc lowering build them) — each push var is
/// bound to the corresponding pop arg `HornTerm`.
fn unify_push_head_with_pop(
    push_head: &HornAtom,
    pop_atom: &HornAtom,
) -> Option<HashMap<String, HornTerm>> {
    let HornTerm::App { fun: pf, args: pa } = &push_head.arg else {
        return None;
    };
    let HornTerm::App { fun: cf, args: ca } = &pop_atom.arg else {
        return None;
    };
    if push_head.pred != pop_atom.pred || pf != cf || pa.len() != ca.len() {
        return None;
    }
    let mut subst: HashMap<String, HornTerm> = HashMap::default();
    for (pattern, candidate) in pa.iter().zip(ca.iter()) {
        match pattern {
            HornTerm::Var(name) => match subst.get(name) {
                Some(existing) if existing != candidate => return None,
                _ => {
                    subst.insert(name.clone(), candidate.clone());
                }
            },
            HornTerm::App { .. } => {
                if pattern != candidate {
                    return None;
                }
            }
        }
    }
    Some(subst)
}

/// Build the resolved rule: pop_rule[body_idx ← (push_rule.body with subst)].
fn resolve(
    pop_rule: &HornRule,
    body_idx: usize,
    push_rule: &HornRule,
    fresh: &mut usize,
) -> Option<HornRule> {
    let pop_atom = pop_rule.body_atoms.get(body_idx)?;
    let subst = unify_push_head_with_pop(&push_rule.head, pop_atom)?;

    // Variables in push.body that are NOT bound by `subst` are push-local —
    // alpha-rename them to fresh names so they don't shadow pop's vars.
    let head_vars = push_rule.head_vars();
    let mut renaming: HashMap<String, HornTerm> = HashMap::default();
    for atom in &push_rule.body_atoms {
        collect_vars(&atom.arg, &mut renaming, &head_vars, fresh);
    }

    // Combined substitution: head vars → pop terms, then local vars → fresh.
    // `subst` and `renaming` are disjoint (head_vars vs non-head-vars).
    let mut combined = subst;
    for (k, v) in renaming {
        combined.insert(k, v);
    }

    // Build resolved body: pop's body minus the resolved atom, plus push's body
    // under the combined substitution.
    let mut body: Vec<HornAtom> = pop_rule
        .body_atoms
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != body_idx)
        .map(|(_, a)| a.clone())
        .collect();
    for atom in &push_rule.body_atoms {
        body.push(atom.substitute(&combined));
    }

    Some(HornRule {
        body_atoms: body,
        head: pop_rule.head.clone(),
    })
}

/// For every variable in `term` that isn't in `head_vars`, insert a
/// fresh-name mapping into `renaming` (if not already present).
fn collect_vars(
    term: &HornTerm,
    renaming: &mut HashMap<String, HornTerm>,
    head_vars: &HashSet<String>,
    fresh: &mut usize,
) {
    match term {
        HornTerm::Var(name) => {
            if !head_vars.contains(name) && !renaming.contains_key(name) {
                let next = format!("__sat{fresh}");
                *fresh += 1;
                renaming.insert(name.clone(), HornTerm::Var(next));
            }
        }
        HornTerm::App { args, .. } => {
            for arg in args {
                collect_vars(arg, renaming, head_vars, fresh);
            }
        }
    }
}

fn ctor_key(atom: &HornAtom) -> Option<CtorKey> {
    let HornTerm::App { fun, args } = &atom.arg else {
        return None;
    };
    Some(CtorKey {
        pred: atom.pred.clone(),
        fun: fun.clone(),
        arity: args.len(),
    })
}

// ---------------------------------------------------------------------------
// Diagnostic formatting
// ---------------------------------------------------------------------------

/// Render a rule for the cap-hit dump, prefixing each body atom with its
/// classified [`AtomRole`] (`pop` / `intr`). Reads exactly like the rule's
/// `Display` impl otherwise; intended for human eyeballing.
fn format_rule_with_roles(rule: &HornRule) -> String {
    let head_vars = rule.head_vars();
    let body_len = rule.body_atoms.len();
    let body = rule
        .body_atoms
        .iter()
        .map(|atom| {
            let tag = match atom.role_against(&head_vars) {
                AtomRole::Pop => "pop ",
                AtomRole::Intersection => "intr",
            };
            format!("[{tag}] {atom}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("body_len={body_len} head={} <= {body}", rule.head)
}

// ---------------------------------------------------------------------------
// Alpha-normalization (for dedup)
// ---------------------------------------------------------------------------

/// Alpha-normalize a rule for dedup. Two passes:
///
/// 1. Identify *isolated existentials* — variables that appear in exactly
///    one body atom and not in the head. These are truly existential
///    (purely "∃ x. ...") and any two body atoms `P(e1)` / `P(e2)` with
///    different isolated existentials `e1, e2` are logically equivalent
///    to a single `P(_)`. Without this, resolution chains accumulate
///    arbitrarily many `P(__sat<i>)` atoms with the same `P` but fresh
///    distinct vars, blowing up alpha-distinct rule count without
///    adding logical content.
/// 2. Rename surviving variables by first-appearance ordinal in a
///    deterministic walk (head, then sorted body).
fn alpha_normalize(rule: &HornRule) -> RuleKey {
    let head_vars = rule.head_vars();
    let counts = count_var_occurrences(rule);
    // A var is an isolated existential iff it appears once total AND not in
    // the head. We canonicalize all isolated existentials to the same
    // sentinel name "_" so duplicate `P(iso1)` / `P(iso2)` atoms collapse.
    let isolated: HashSet<String> = counts
        .iter()
        .filter(|(name, count)| **count == 1 && !head_vars.contains(*name))
        .map(|(name, _)| name.clone())
        .collect();

    let mut order: HashMap<String, usize> = HashMap::default();
    let head = normalize_atom(&rule.head, &mut order, &isolated);
    let mut body: Vec<HornAtom> = rule
        .body_atoms
        .iter()
        .map(|a| normalize_atom(a, &mut order, &isolated))
        .collect();
    body.sort();
    body.dedup();
    RuleKey { head, body }
}

/// Count every variable occurrence across a rule (head and all body atoms).
/// A var that appears multiple times is "linked" — it represents a constraint
/// shared across atoms — and stays a real variable. A var that appears once
/// (and not in head) is an isolated existential.
fn count_var_occurrences(rule: &HornRule) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::default();
    count_term_occurrences(&rule.head.arg, &mut counts);
    for atom in &rule.body_atoms {
        count_term_occurrences(&atom.arg, &mut counts);
    }
    counts
}

fn count_term_occurrences(term: &HornTerm, counts: &mut HashMap<String, usize>) {
    match term {
        HornTerm::Var(name) => {
            *counts.entry(name.clone()).or_insert(0) += 1;
        }
        HornTerm::App { args, .. } => {
            for arg in args {
                count_term_occurrences(arg, counts);
            }
        }
    }
}

fn normalize_atom(
    atom: &HornAtom,
    order: &mut HashMap<String, usize>,
    isolated: &HashSet<String>,
) -> HornAtom {
    HornAtom {
        pred: atom.pred.clone(),
        arg: normalize_term(&atom.arg, order, isolated),
    }
}

fn normalize_term(
    term: &HornTerm,
    order: &mut HashMap<String, usize>,
    isolated: &HashSet<String>,
) -> HornTerm {
    match term {
        HornTerm::Var(name) => {
            if isolated.contains(name) {
                HornTerm::Var("_".to_string())
            } else {
                let next = order.len();
                let idx = *order.entry(name.clone()).or_insert(next);
                HornTerm::Var(format!("v{idx}"))
            }
        }
        HornTerm::App { fun, args } => HornTerm::App {
            fun: fun.clone(),
            args: args.iter().map(|a| normalize_term(a, order, isolated)).collect(),
        },
    }
}

// ---------------------------------------------------------------------------
// Tiny forward-chaining evaluator (for tests only)
// ---------------------------------------------------------------------------

#[cfg(test)]
fn collect_subterms(term: &HornTerm, out: &mut Vec<HornTerm>) {
    if !out.contains(term) {
        out.push(term.clone());
    }
    if let HornTerm::App { args, .. } = term {
        for arg in args {
            collect_subterms(arg, out);
        }
    }
}

/// Structural match of a rule head's term pattern against a ground candidate.
/// Returns the substitution head-vars → candidate-subterms on success, else None.
#[cfg(test)]
fn match_term(pattern: &HornTerm, candidate: &HornTerm) -> Option<HashMap<String, HornTerm>> {
    let mut subst = HashMap::default();
    if match_term_into(pattern, candidate, &mut subst) {
        Some(subst)
    } else {
        None
    }
}

#[cfg(test)]
fn match_term_into(
    pattern: &HornTerm,
    candidate: &HornTerm,
    subst: &mut HashMap<String, HornTerm>,
) -> bool {
    match (pattern, candidate) {
        (HornTerm::Var(name), c) => match subst.get(name) {
            Some(existing) if existing != c => false,
            Some(_) => true,
            None => {
                subst.insert(name.clone(), c.clone());
                true
            }
        },
        (
            HornTerm::App { fun: pf, args: pa },
            HornTerm::App { fun: cf, args: ca },
        ) => {
            if pf != cf || pa.len() != ca.len() {
                return false;
            }
            for (p, c) in pa.iter().zip(ca.iter()) {
                if !match_term_into(p, c, subst) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Forward-chain a saturated push-rule program over the subterms of `term`.
/// Each rule has head shape `Pred(cons(x1, ..., xk))`. We unify head with
/// every subterm, check body atoms (with the substitution applied) are in
/// the derived set, and add the head if so. Iterate until fixpoint.
///
/// Body atoms after substitution must be either ground (`Pred(c)`) or a
/// `Pred(Var)` with the var bound by the substitution. Range-restricted
/// rules guarantee this.
#[cfg(test)]
fn evaluate(saturated: &[HornRule], term: &HornTerm) -> rustc_hash::FxHashSet<HornAtom> {
    let mut subterms = Vec::new();
    collect_subterms(term, &mut subterms);
    let mut derived: rustc_hash::FxHashSet<HornAtom> = HashSet::default();
    loop {
        let mut changed = false;
        for rule in saturated {
            for candidate in &subterms {
                let Some(subst) = match_term(&rule.head.arg, candidate) else {
                    continue;
                };
                let body_satisfied = rule.body_atoms.iter().all(|atom| {
                    let grounded = atom.substitute(&subst);
                    derived.contains(&grounded)
                });
                if body_satisfied {
                    let head = HornAtom {
                        pred: rule.head.pred.clone(),
                        arg: rule.head.arg.substitute(&subst),
                    };
                    if derived.insert(head) {
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    derived
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(pred: &str, arg: HornTerm) -> HornAtom {
        HornAtom {
            pred: pred.to_string(),
            arg,
        }
    }

    fn v(name: &str) -> HornTerm {
        HornTerm::var(name)
    }

    fn app(fun: &str, args: Vec<HornTerm>) -> HornTerm {
        HornTerm::app(fun, args)
    }

    /// Alpha-normalization is stable: two rules equivalent up to renaming
    /// give the same key.
    #[test]
    fn alpha_normalize_collapses_renaming() {
        let r1 = HornRule {
            body_atoms: vec![atom("P", v("x")), atom("Q", v("y"))],
            head: atom("R", app("f", vec![v("x"), v("y")])),
        };
        let r2 = HornRule {
            body_atoms: vec![atom("P", v("a")), atom("Q", v("b"))],
            head: atom("R", app("f", vec![v("a"), v("b")])),
        };
        assert_eq!(alpha_normalize(&r1), alpha_normalize(&r2));
    }

    /// Two rules with *different* variable identifications give different keys.
    #[test]
    fn alpha_normalize_distinguishes_distinct_links() {
        // x linked to itself in head vs distinct x,y.
        let r1 = HornRule {
            body_atoms: vec![],
            head: atom("R", app("f", vec![v("x"), v("x")])),
        };
        let r2 = HornRule {
            body_atoms: vec![],
            head: atom("R", app("f", vec![v("x"), v("y")])),
        };
        assert_ne!(alpha_normalize(&r1), alpha_normalize(&r2));
    }

    /// Resolution: push `P(f(a,b)) :- Q(a)`. Pop `S(x) :- P(f(x, y))`.
    /// Result: `S(x) :- Q(x)`. (y becomes fresh, but Q is on a only, so dropped.)
    #[test]
    fn resolve_single_step() {
        let push = HornRule {
            body_atoms: vec![atom("Q", v("a"))],
            head: atom("P", app("f", vec![v("a"), v("b")])),
        };
        let pop = HornRule {
            body_atoms: vec![atom("P", app("f", vec![v("x"), v("y")]))],
            head: atom("S", v("x")),
        };
        let mut fresh = 0;
        let resolved = resolve(&pop, 0, &push, &mut fresh).expect("must resolve");
        assert_eq!(resolved.head, atom("S", v("x")));
        assert_eq!(resolved.body_atoms, vec![atom("Q", v("x"))]);
    }

    /// Push body has a var that's *not* in the head — must be alpha-renamed
    /// to avoid capturing pop's `y`.
    #[test]
    fn resolve_fresh_renaming_avoids_capture() {
        let push = HornRule {
            // `b` is bound to pop's `y` via head unification, but `c` is a
            // push-local var — it must be renamed to fresh.
            body_atoms: vec![atom("Q", v("c"))],
            head: atom("P", app("f", vec![v("a"), v("b")])),
        };
        let pop = HornRule {
            // Pop's `y` happens to be named the same as a hypothetical push
            // var; the renaming should not unify them.
            body_atoms: vec![
                atom("R", v("y")),
                atom("P", app("f", vec![v("x"), v("y")])),
            ],
            head: atom("S", v("x")),
        };
        let mut fresh = 0;
        let resolved = resolve(&pop, 1, &push, &mut fresh).expect("must resolve");
        // pop's R(y) is kept; push's Q(c) becomes Q(__sat0); no clash.
        assert_eq!(resolved.head, atom("S", v("x")));
        assert!(resolved
            .body_atoms
            .iter()
            .any(|a| a.pred == "R" && a.arg == v("y")));
        assert!(resolved.body_atoms.iter().any(|a| a.pred == "Q"
            && matches!(&a.arg, HornTerm::Var(name) if name.starts_with("__sat"))));
    }

    /// REGRESSION: end-to-end saturation chain that mirrors the empirical
    /// 482-atom body case found in `/tmp/anbn_min.sl`. The chain:
    ///   R1 (pop): H :- pop(C(K, __up1))
    ///   P  (push): pop(C(x, y)) :- a1(x), a2(y)     where a1's arg becomes K (App, pop)
    /// Resolving R1 vs P gives R2 with body [a1(K), a2(__up1)] — a1(K) is a pop
    /// site that matches R1's head (because K is the same nullary App).
    /// Resolving R2 vs R1 then vs P alternates and grows the body.
    /// Without isolated-existential collapsing, this chain runs forever.
    /// With it, the chain must terminate (alpha-key stabilises).
    #[test]
    fn saturation_chain_terminates_isolated_existential_loop() {
        use rustc_hash::FxHashSet as Set;
        let _ = Set::<u8>::default();
        let rules = vec![
            // R1: H(K()) :- pop(C(K(), __up1))     [pop rule, K is nullary App]
            HornRule {
                body_atoms: vec![atom(
                    "pop_pred",
                    app("C", vec![app("K", vec![]), v("__up1")]),
                )],
                head: atom("H", app("K", vec![])),
            },
            // P: pop_pred(C(x, y)) :- H(x), a2(y)  [push, range-restricted]
            HornRule {
                body_atoms: vec![
                    atom("H", v("x")),
                    atom("a2", v("y")),
                ],
                head: atom("pop_pred", app("C", vec![v("x"), v("y")])),
            },
        ];
        let saturated = saturate_rules(&rules);
        // Saturation must finish (terminate) and produce no pop clauses.
        for rule in &saturated {
            assert!(
                !rule.is_pop_clause(),
                "pop clause survived: {rule}"
            );
        }
    }

    /// REGRESSION: isolated existentials must canonicalize to the same key.
    /// Two rules with the same head and same predicates in the body but with
    /// different "throwaway" fresh var names must be alpha-equivalent.
    /// Without this, saturation chains where each resolution adds a fresh
    /// var to the body produce unbounded distinct rules.
    #[test]
    fn alpha_normalize_collapses_isolated_existentials() {
        // R3: nqRhs.Sem(cfg.rhs.S0) :- nqRules.Sem.find.S0(__up1),
        //                              nqCFG.Sem.reset.S0(cfg.top.S0(cfg.rhs.S0, __sat1))
        let r3 = HornRule {
            body_atoms: vec![
                atom("nqRules.Sem.find.S0", v("__up1")),
                atom(
                    "nqCFG.Sem.reset.S0",
                    app("cfg.top.S0", vec![app("cfg.rhs.S0", vec![]), v("__sat1")]),
                ),
            ],
            head: atom("nqRhs.Sem.base", app("cfg.rhs.S0", vec![])),
        };
        // R5: same shape but with one EXTRA find.S0 atom on a different
        // fresh var. Should normalize to the same key as R3.
        let r5 = HornRule {
            body_atoms: vec![
                atom("nqRules.Sem.find.S0", v("__up1")),
                atom("nqRules.Sem.find.S0", v("__sat1")),
                atom(
                    "nqCFG.Sem.reset.S0",
                    app("cfg.top.S0", vec![app("cfg.rhs.S0", vec![]), v("__sat2")]),
                ),
            ],
            head: atom("nqRhs.Sem.base", app("cfg.rhs.S0", vec![])),
        };
        let k3 = alpha_normalize(&r3);
        let k5 = alpha_normalize(&r5);
        assert_eq!(
            k3, k5,
            "R3 and R5 differ only in extra Pred(IsolatedExistential) atoms; \
             they must alpha-normalize to the same key.\nR3 key: {k3:?}\nR5 key: {k5:?}"
        );
    }

    /// Smoke test for the worklist: a trivial cycle that does NOT diverge
    /// (no new alpha-distinct rules ever appear).
    #[test]
    fn saturate_terminates_no_pops() {
        let rules = vec![
            HornRule {
                body_atoms: vec![],
                head: atom("P", app("a", vec![])),
            },
            HornRule {
                body_atoms: vec![atom("P", v("x"))],
                head: atom("P", app("g", vec![v("x")])),
            },
        ];
        let out = saturate_rules(&rules);
        assert_eq!(out.len(), 2);
    }

    /// CFG-shaped resolution: pop rule with a NESTED cons in body resolves
    /// against the push rule for the outer cons. From `old_saturate.rs`.
    #[test]
    fn cfg_style_one_step_resolution() {
        // Push: qCfgResetS(cfg.top.S(x, y)) :- qRhsBase(x)
        let push = HornRule {
            body_atoms: vec![atom("qRhsBase", v("x"))],
            head: atom("qCfgResetS", app("cfg.top.S", vec![v("x"), v("y")])),
        };
        // Pop body atom: qCfgResetS(cfg.top.S(cfg.rhs.S(), __up1))
        let pop = HornRule {
            body_atoms: vec![atom(
                "qCfgResetS",
                app(
                    "cfg.top.S",
                    vec![app("cfg.rhs.S", vec![]), v("__up1")],
                ),
            )],
            head: atom("qRhsBase", app("cfg.rhs.S", vec![])),
        };
        let mut fresh = 0;
        let resolved = resolve(&pop, 0, &push, &mut fresh).expect("must resolve");
        // Push var x → cfg.rhs.S(), y → __up1. Body x becomes cfg.rhs.S().
        // Expected resolved: qRhsBase(cfg.rhs.S()) :- qRhsBase(cfg.rhs.S())
        assert_eq!(
            resolved.head,
            atom("qRhsBase", app("cfg.rhs.S", vec![]))
        );
        assert_eq!(
            resolved.body_atoms,
            vec![atom("qRhsBase", app("cfg.rhs.S", vec![]))]
        );
    }

    /// End-to-end CFG-style saturation with two push rules + one pop rule.
    /// Verifies saturation TERMINATES, produces NO pop clauses, and includes
    /// the expected resolved rules. Mirrors `old_saturate.rs::cfg_style_parent_pop_clause_saturates`.
    #[test]
    fn cfg_style_full_saturation() {
        let rules = vec![
            // Push 1: qCfgResetS(cfg.top.S(x, y)) :- qRhsBase(x)
            HornRule {
                body_atoms: vec![atom("qRhsBase", v("x"))],
                head: atom("qCfgResetS", app("cfg.top.S", vec![v("x"), v("y")])),
            },
            // Push 2: qCfgResetS(cfg.top.S(x, y)) :- qRulesFindS(y)
            HornRule {
                body_atoms: vec![atom("qRulesFindS", v("y"))],
                head: atom("qCfgResetS", app("cfg.top.S", vec![v("x"), v("y")])),
            },
            // Pop: qRhsBase(cfg.rhs.S()) :- qCfgResetS(cfg.top.S(cfg.rhs.S(), __up1))
            HornRule {
                body_atoms: vec![atom(
                    "qCfgResetS",
                    app(
                        "cfg.top.S",
                        vec![app("cfg.rhs.S", vec![]), v("__up1")],
                    ),
                )],
                head: atom("qRhsBase", app("cfg.rhs.S", vec![])),
            },
        ];
        let saturated = saturate_rules(&rules);
        // No pop clauses should survive.
        for rule in &saturated {
            assert!(
                !rule.is_pop_clause(),
                "saturated rule still has a pop clause: {rule}"
            );
        }
        // Both original push rules survive (no Var heads, they have App heads).
        let original_pushes: Vec<&HornRule> = saturated
            .iter()
            .filter(|r| {
                matches!(&r.head.arg, HornTerm::App { fun, .. } if fun == "cfg.top.S")
            })
            .collect();
        assert!(
            original_pushes.len() >= 2,
            "expected both cfg.top.S pushes to survive, got {} ({saturated:#?})",
            original_pushes.len()
        );
        // Resolved rule with head qRhsBase(cfg.rhs.S()) should appear, with
        // body qRulesFindS(__sat0) (from push 2; push 1 produces a tautology
        // qRhsBase(cfg.rhs.S()) :- qRhsBase(cfg.rhs.S()) which is self-subsumed).
        let resolved: Vec<&HornRule> = saturated
            .iter()
            .filter(|r| {
                matches!(&r.head.arg, HornTerm::App { fun, args } if fun == "cfg.rhs.S" && args.is_empty())
            })
            .collect();
        assert!(
            !resolved.is_empty(),
            "expected a resolved rule with head qRhsBase(cfg.rhs.S()); got none ({saturated:#?})"
        );
    }

    /// SOUNDNESS/COMPLETENESS test: a tiny 2-way TA that accepts trees
    /// whose first child is `a`. We saturate the Horn encoding, then
    /// forward-evaluate on two trees and check Goal acceptance matches the
    /// TA's intended semantics.
    ///
    /// Initial Horn rules (encoding the 2-way TA):
    ///   q_start(top(x, y)) :- q_check_a(x).        // start: check first child
    ///   q_check_a(a) :- q_seen_a(top(a, __up)).    // up-direction from a
    ///   q_check_a(a) :- q_seen_a(top(__up, a)).    // up-direction (other slot)
    ///   q_seen_a(top(x, y)).                       // parent receives signal
    ///
    /// After saturation, no pop clauses should remain. The closed rules
    /// must derive q_start(top(a, b)) but not q_start(top(b, a)).
    #[test]
    fn saturated_program_accepts_iff_ta_accepts() {
        let rules = vec![
            // q_start(top(x, y)) :- q_check_a(x)
            HornRule {
                body_atoms: vec![atom("q_check_a", v("x"))],
                head: atom("q_start", app("top", vec![v("x"), v("y")])),
            },
            // q_check_a(a) :- q_seen_a(top(a, __up))
            HornRule {
                body_atoms: vec![atom(
                    "q_seen_a",
                    app("top", vec![app("a", vec![]), v("__up")]),
                )],
                head: atom("q_check_a", app("a", vec![])),
            },
            // q_check_a(a) :- q_seen_a(top(__up, a))
            HornRule {
                body_atoms: vec![atom(
                    "q_seen_a",
                    app("top", vec![v("__up"), app("a", vec![])]),
                )],
                head: atom("q_check_a", app("a", vec![])),
            },
            // q_seen_a(top(x, y))  -- the parent always signals OK
            HornRule {
                body_atoms: vec![],
                head: atom("q_seen_a", app("top", vec![v("x"), v("y")])),
            },
        ];
        let saturated = saturate_rules(&rules);

        // No pop clauses should survive saturation.
        for rule in &saturated {
            assert!(
                !rule.is_pop_clause(),
                "pop clause survived saturation: {rule}"
            );
        }

        // Construct two ground trees.
        let tree_ab = app("top", vec![app("a", vec![]), app("b", vec![])]);
        let tree_ba = app("top", vec![app("b", vec![]), app("a", vec![])]);

        // top(a, b): first child is `a`, TA accepts.
        let truths_ab = evaluate(&saturated, &tree_ab);
        let goal_ab = HornAtom {
            pred: "q_start".to_string(),
            arg: tree_ab.clone(),
        };
        assert!(
            truths_ab.contains(&goal_ab),
            "expected saturated program to derive q_start(top(a, b)); derived: {truths_ab:#?}\n\nsaturated rules: {saturated:#?}"
        );

        // top(b, a): first child is `b`, TA rejects.
        let truths_ba = evaluate(&saturated, &tree_ba);
        let goal_ba = HornAtom {
            pred: "q_start".to_string(),
            arg: tree_ba.clone(),
        };
        assert!(
            !truths_ba.contains(&goal_ba),
            "expected saturated program to NOT derive q_start(top(b, a)); derived: {truths_ba:#?}"
        );
    }

    /// REGRESSION (intersection protection): a helper-pred intersection
    /// clause `__h(f(x,y)) ← Q(x)` looks identical to a push rule from
    /// `ctor_key`'s perspective. Before the fix, saturation would resolve
    /// a main rule's body atom `__h(f(x,y))` against it, inlining the
    /// helper's body and destroying alternation. After the fix, the body
    /// atom is classified `Intersection` (all vars `{x, y}` ⊆ head vars
    /// `{x, y}`) and never indexed as a pop site.
    #[test]
    fn intersection_clause_is_protected_from_resolution() {
        let rules = vec![
            // Main rule:  H(f(x, y)) :- __h(f(x, y)).   [intersection body]
            HornRule {
                body_atoms: vec![atom("__h", app("f", vec![v("x"), v("y")]))],
                head: atom("H", app("f", vec![v("x"), v("y")])),
            },
            // Helper push:  __h(f(x, y)) :- Q(x).
            HornRule {
                body_atoms: vec![atom("Q", v("x"))],
                head: atom("__h", app("f", vec![v("x"), v("y")])),
            },
        ];
        let saturated = saturate_rules_with(&rules, SaturateConfig::default());
        // The pre-fix algorithm would resolve the main rule's body atom
        // against the helper push, producing a third rule
        // `H(f(x, y)) :- Q(x)`. With intersection protection, no resolution
        // fires: we keep exactly the two input rules.
        assert_eq!(
            saturated.len(),
            2,
            "intersection clause was resolved; got {saturated:#?}"
        );
        let inlined = saturated.iter().any(|r| {
            r.head.pred == "H" && r.body_atoms.iter().any(|a| a.pred == "Q")
        });
        assert!(
            !inlined,
            "main rule's intersection body was inlined: {saturated:#?}"
        );
    }

    /// REGRESSION (no AND-of-OR → DNF blowup): the encoding of a single
    /// `AND_{k=1..K} of (OR (p_k_x x) (p_k_y y))` produced by
    /// `chc.rs::emit_helper_atom` is 1 main rule + 2K helper push rules.
    /// Before the fix, saturation resolved each helper's K body sites in
    /// the main rule against 2 helper pushes each, producing 2^K rules.
    /// After the fix, the helpers stay as intersection clauses; the rule
    /// count stays at 1 + 2K.
    #[test]
    fn and_of_k_disjunctions_does_not_blow_up() {
        const K: usize = 6; // 2^6 = 64 would be bad; expected 1 + 2K = 13.
        let mut rules = Vec::new();
        let head_args = vec![v("x"), v("y")];
        let helper_name = |k: usize| format!("__h{k}");
        // Main rule:  H(f(x, y)) :- __h1(f(x,y)), ..., __hK(f(x,y)).
        rules.push(HornRule {
            body_atoms: (0..K)
                .map(|k| atom(&helper_name(k), app("f", head_args.clone())))
                .collect(),
            head: atom("H", app("f", head_args.clone())),
        });
        // For each k, two helper pushes (OR of two disjuncts).
        for k in 0..K {
            rules.push(HornRule {
                body_atoms: vec![atom(&format!("p{k}x"), v("x"))],
                head: atom(&helper_name(k), app("f", head_args.clone())),
            });
            rules.push(HornRule {
                body_atoms: vec![atom(&format!("p{k}y"), v("y"))],
                head: atom(&helper_name(k), app("f", head_args.clone())),
            });
        }
        let saturated = saturate_rules_with(&rules, SaturateConfig::default());
        // 1 main + 2K helpers, no DNF expansion.
        assert_eq!(
            saturated.len(),
            1 + 2 * K,
            "expected 1 + 2K = {} saturated rules, got {} \
             (DNF expansion of AND-of-OR would give 1 + 2K + 2^K = {})",
            1 + 2 * K,
            saturated.len(),
            1 + 2 * K + (1usize << K),
        );
    }

    /// End-to-end micro saturation: pop rule resolves against push rule once,
    /// producing a closed push rule.
    #[test]
    fn saturate_micro_close_one_pop() {
        let rules = vec![
            // Push: P(f(a)) :- Q(a)
            HornRule {
                body_atoms: vec![atom("Q", v("a"))],
                head: atom("P", app("f", vec![v("a")])),
            },
            // Push: Q(g(b))  (a base for Q)
            HornRule {
                body_atoms: vec![],
                head: atom("Q", app("g", vec![v("b")])),
            },
            // Pop: S(x) :- P(f(x))
            HornRule {
                body_atoms: vec![atom("P", app("f", vec![v("x")]))],
                head: atom("S", v("x")),
            },
        ];
        let out = saturate_rules(&rules);
        // Expected closed rules: the two original push rules + one resolved
        // S(x) :- Q(x). All have head=App, so they survive `finish`.
        let s_rules: Vec<_> = out
            .iter()
            .filter(|r| r.head.pred == "S")
            .collect();
        // S head is Var, so it's filtered out by finish() (we keep push only).
        // Verify the resolved S rule did NOT come through the filter.
        assert!(s_rules.is_empty());
    }
}
