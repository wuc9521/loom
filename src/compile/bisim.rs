use crate::compile::eval_ctx::{EnvView, SemanticContext};
use crate::compile::semantics::{parse_cases, Case};
use crate::compile::types::{AspectKey, AspectValue, DomainBit, SemanticKind, SemanticProgram};
use crate::fxmap::{FxHasher, HashMap, HashSet};
use crate::lang::model::{InterpedValue, Model};
use crate::parser::{LoomCmd, LoomProg};
use std::cell::RefCell;
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Instant;
use sygus_parser::Literal;
use sygus_parser::SExpr as Value;

type Fingerprint = Vec<u8>;
type ClassId = usize;
type ClassKey = (SemKey, AspectKey, Option<AspectValue>);
type DenseClassKey = (usize, usize, Option<usize>);
type SigHash = u64;
type WalkCache = HashMap<WalkKey, (u64, Vec<ClassId>)>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SemKey {
    relation: String,
    term_sort: String,
}

impl SemKey {
    fn new(relation: impl Into<String>, term_sort: impl Into<String>) -> Self {
        Self {
            relation: relation.into(),
            term_sort: term_sort.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum QuotientMode {
    FingerprintSeed,
    Coarsest,
}

fn dedup_deps(mut deps: Vec<ClassId>) -> Vec<ClassId> {
    deps.sort_unstable();
    deps.dedup();
    deps
}

/// `walk` memoization key: `(expr_ptr, env_restricted_to_free_vars_hash)`.
/// The expr_ptr uniquely identifies a sub-expression of a parsed case body
/// (parsed once and never re-allocated). The hash restricts to the free
/// variables of that sub-expression so unrelated outer quantifier bindings
/// don't pollute the key.
type WalkKey = (usize, u64);

thread_local! {
    static WALK_CACHE_HITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static WALK_CACHE_MISSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl AspectKey {
    /// Per-position leaf-function readout. Used only to seed the initial
    /// partition; refinement is what guarantees soundness.
    pub fn fingerprint(&self, model: &Model, func_names: &[String]) -> Fingerprint {
        let mut fp = Vec::new();

        for (name, value) in &self.fields {
            match value {
                AspectValue::Int(_) | AspectValue::Model(_) => {}
                AspectValue::Symbol(s) => {
                    fp.extend_from_slice(name.as_bytes());
                    fp.push(b'=');
                    fp.extend_from_slice(s.as_bytes());
                    fp.push(b';');
                }
                AspectValue::Univ { sort, name: n } => {
                    fp.extend_from_slice(name.as_bytes());
                    fp.push(b'=');
                    fp.extend_from_slice(sort.as_bytes());
                    fp.push(b':');
                    fp.extend_from_slice(n.as_bytes());
                    fp.push(b';');
                }
            }
        }

        let int_fields: Vec<usize> = self
            .fields
            .iter()
            .filter_map(|(_, v)| match v {
                AspectValue::Int(i) => Some(*i),
                _ => None,
            })
            .collect();

        let (start, end) = match int_fields.as_slice() {
            [i, j] => (*i, *j),
            [i] => (*i, *i + 1),
            _ => return fp,
        };

        fp.push(b'[');
        for pos in start..end {
            let pos_name = format!("${pos}");
            for func in func_names {
                match model.eval_symbols(func, &[pos_name.clone()]) {
                    Some(InterpedValue::Literal(Literal::Bool(true))) => fp.push(b'1'),
                    Some(InterpedValue::Literal(Literal::Bool(false))) => fp.push(b'0'),
                    Some(InterpedValue::UnivElem(e)) => {
                        fp.extend_from_slice(e.name.as_bytes());
                        fp.push(b',');
                    }
                    Some(_) => fp.push(b'?'),
                    None => fp.push(b'_'),
                }
            }
            fp.push(b'|');
        }
        fp.push(b']');
        fp
    }
}

impl SemanticProgram {
    /// Compute the largest sound bisimulation over aspect bits via partition
    /// refinement, then collapse equivalent bits onto shared positions.
    /// Recursive relation references resolve to class IDs from the previous
    /// iteration; aspects whose symbolic case-body signatures all match stay
    /// merged.
    pub fn quotient_by_bisim(&mut self, problem: &LoomProg, mode: QuotientMode) {
        let start = Instant::now();
        let total_before: usize = self.domains.iter().map(|d| d.bits.len()).sum();
        if total_before == 0 {
            return;
        }

        let mut cases_by_relation: HashMap<String, HashMap<String, Case>> = HashMap::default();
        let mut cases_by_domain: HashMap<SemKey, HashMap<String, Case>> = HashMap::default();
        for spec in &self.semantics {
            if let Ok(cases) = parse_cases(&spec.body, &spec.aspect_var) {
                let entry = cases_by_relation.entry(spec.relation.clone()).or_default();
                let domain_entry = cases_by_domain
                    .entry(SemKey::new(&spec.relation, &spec.term_sort))
                    .or_default();
                for (cons, case) in cases {
                    entry.insert(cons.clone(), case.clone());
                    domain_entry.insert(cons, case);
                }
            }
        }

        let ctx = match SemanticContext::with_cases(problem, self, &cases_by_relation) {
            Ok(ctx) => ctx,
            Err(e) => {
                log::warn!("bisim: skipped, semantic context unavailable: {e}");
                return;
            }
        };

        let mut sem_ids: HashMap<SemKey, usize> = HashMap::default();
        let mut aspect_ids: HashMap<AspectKey, usize> = HashMap::default();
        let mut value_ids: HashMap<AspectValue, usize> = HashMap::default();
        for domain in &self.domains {
            let sem = SemKey::new(&domain.relation, &domain.term_sort);
            let next_sem = sem_ids.len();
            sem_ids.entry(sem).or_insert(next_sem);
            for bit in &domain.bits {
                let next_aspect = aspect_ids.len();
                aspect_ids.entry(bit.aspect.clone()).or_insert(next_aspect);
                if let Some(value) = &bit.value {
                    let next_value = value_ids.len();
                    value_ids.entry(value.clone()).or_insert(next_value);
                }
            }
        }

        let mut classes: HashMap<ClassKey, ClassId> = HashMap::default();
        match mode {
            QuotientMode::FingerprintSeed => {
                let func_names: Vec<String> = ctx.func_sigs.keys().cloned().collect();
                let mut leaf_to_class: HashMap<
                    (SemKey, Fingerprint, Option<AspectValue>),
                    ClassId,
                > = HashMap::default();
                for domain in &self.domains {
                    let sem = SemKey::new(&domain.relation, &domain.term_sort);
                    for bit in &domain.bits {
                        let fp = ctx
                            .models
                            .get(&bit.aspect.model)
                            .map(|m| bit.aspect.fingerprint(m, &func_names))
                            .unwrap_or_default();
                        let key = (sem.clone(), fp, bit.value.clone());
                        let next_id = leaf_to_class.len();
                        let class = *leaf_to_class.entry(key).or_insert(next_id);
                        classes.insert((sem.clone(), bit.aspect.clone(), bit.value.clone()), class);
                    }
                }
            }
            QuotientMode::Coarsest => {
                let mut seed_to_class: HashMap<(SemKey, Option<AspectValue>), ClassId> =
                    HashMap::default();
                for domain in &self.domains {
                    let sem = SemKey::new(&domain.relation, &domain.term_sort);
                    for bit in &domain.bits {
                        let key = (sem.clone(), bit.value.clone());
                        let next_id = seed_to_class.len();
                        let class = *seed_to_class.entry(key).or_insert(next_id);
                        classes.insert((sem.clone(), bit.aspect.clone(), bit.value.clone()), class);
                    }
                }
            }
        }

        let member_keys = classes.keys().cloned().collect::<Vec<_>>();
        let member_dense_keys = member_keys
            .iter()
            .map(|(sem, aspect, value)| {
                (
                    sem_ids[sem],
                    aspect_ids[aspect],
                    value.as_ref().map(|v| value_ids[v]),
                )
            })
            .collect::<Vec<_>>();
        let mut dense_classes: HashMap<DenseClassKey, ClassId> = HashMap::default();
        for (idx, key) in member_keys.iter().enumerate() {
            dense_classes.insert(member_dense_keys[idx], classes[key]);
        }
        drop(classes);

        let domain_kinds: HashMap<SemKey, SemanticKind> = self
            .domains
            .iter()
            .map(|d| (SemKey::new(&d.relation, &d.term_sort), d.kind.clone()))
            .collect();
        let refiner = Refiner::new(
            problem,
            &ctx,
            &cases_by_domain,
            &domain_kinds,
            &sem_ids,
            &aspect_ids,
        );

        // Refinement only ever *splits* classes — it cannot merge two
        // already-distinct classes. So the only aspects whose class can
        // possibly change are those currently sharing a class with at least
        // one other aspect. Singleton-class aspects are at their final
        // position after the leaf seed; re-signaturing them is pure waste.
        //
        // We track membership and only re-signature aspects in multi-aspect
        // classes. When a class splits, the resulting sub-classes inherit
        // multi-aspect status only if they still have >1 member; singleton
        // sub-classes drop out of the worklist.
        let mut class_members: HashMap<ClassId, Vec<usize>> = HashMap::default();
        for (idx, key) in member_dense_keys.iter().enumerate() {
            let cid = dense_classes[key];
            class_members.entry(cid).or_default().push(idx);
        }
        let mut multi_aspect: Vec<ClassId> = class_members
            .iter()
            .filter(|(_, members)| members.len() > 1)
            .map(|(&id, _)| id)
            .collect();
        let mut class_deps: HashMap<ClassId, Vec<ClassId>> = HashMap::default();
        let mut next_class_id = dense_classes
            .values()
            .max()
            .copied()
            .map(|m| m + 1)
            .unwrap_or(0);

        // Scratch reused across every signature call (~95k per quotient):
        // signature() clear()s both per call, which keeps bucket capacity
        // instead of re-allocating five HashMaps each time.
        let mut env = WalkEnv {
            aspects: HashMap::default(),
            values: HashMap::default(),
            subterms: HashMap::default(),
            subterm_sorts: HashMap::default(),
            current_sort: String::new(),
            current_model: None,
        };
        let mut cache: WalkCache = HashMap::default();
        let mut iters = 0usize;
        while !multi_aspect.is_empty() {
            iters += 1;
            let mut any_split = false;
            let mut changed: HashSet<ClassId> = HashSet::default();
            for cid in std::mem::take(&mut multi_aspect) {
                let members = match class_members.get(&cid) {
                    Some(m) if m.len() > 1 => m.clone(),
                    _ => continue,
                };
                // Group members of this class by their refined signature.
                let mut by_sig: HashMap<SigHash, (Vec<usize>, Vec<ClassId>)> = HashMap::default();
                for &member_id in &members {
                    let member = &member_keys[member_id];
                    let (sig, deps) = refiner.signature(
                        &member.0,
                        &member.1,
                        &member.2,
                        &dense_classes,
                        &mut env,
                        &mut cache,
                    );
                    let entry = by_sig.entry(sig).or_default();
                    entry.0.push(member_id);
                    entry.1.extend(deps);
                }
                if by_sig.len() == 1 {
                    // No split: the class is stable under current partition.
                    class_deps.insert(cid, dedup_deps(by_sig.into_values().next().unwrap().1));
                    continue;
                }
                any_split = true;
                changed.insert(cid);
                // First group keeps `cid`; remaining groups get fresh IDs.
                let mut groups: Vec<(Vec<usize>, Vec<ClassId>)> = by_sig.into_values().collect();
                groups.sort_by(|a, b| a.0.first().cmp(&b.0.first()));
                let first = groups.remove(0);
                class_deps.insert(cid, dedup_deps(first.1));
                class_members.insert(cid, first.0);
                for (group, deps) in groups {
                    let new_id = next_class_id;
                    next_class_id += 1;
                    for &member_id in &group {
                        dense_classes.insert(member_dense_keys[member_id], new_id);
                    }
                    class_deps.insert(new_id, dedup_deps(deps));
                    class_members.insert(new_id, group);
                }
            }
            if !any_split {
                // Stable: this iter produced no splits, so no referenced
                // class can change either. Done.
                break;
            }
            multi_aspect = class_members
                .iter()
                .filter(|(_, members)| members.len() > 1)
                .filter_map(|(&id, _)| {
                    class_deps
                        .get(&id)
                        .is_some_and(|deps| deps.iter().any(|dep| changed.contains(dep)))
                        .then_some(id)
                })
                .collect();
            // No iteration cap: refinement only ever splits (≤ #aspects times)
            // and the `any_split` check above detects the fixpoint, so this
            // always converges to the true bisimulation. A cap here would stop
            // at a partition coarser than the bisimulation — an unsound
            // over-merge.
        }

        let eval_hits = ctx.eval_cache_hits();
        let eval_misses = ctx.eval_cache_misses();
        drop(refiner);
        drop(ctx);

        // Reassign each domain's bit indices to local class IDs.
        let mut total_after = 0usize;
        for domain in self.domains.iter_mut() {
            if domain.bits.is_empty() {
                continue;
            }
            let mut local_remap: HashMap<ClassId, usize> = HashMap::default();
            let mut new_bits: Vec<DomainBit> = Vec::with_capacity(domain.bits.len());
            for bit in &domain.bits {
                let sem = SemKey::new(&domain.relation, &domain.term_sort);
                let dense_key = (
                    sem_ids[&sem],
                    aspect_ids[&bit.aspect],
                    bit.value.as_ref().map(|v| value_ids[v]),
                );
                let class = dense_classes.get(&dense_key).copied().unwrap_or(usize::MAX);
                let next_local = local_remap.len();
                let local = *local_remap.entry(class).or_insert(next_local);
                new_bits.push(DomainBit {
                    bit: local,
                    aspect: bit.aspect.clone(),
                    value: bit.value.clone(),
                });
            }
            total_after += local_remap.len();
            domain.bits = new_bits;
        }

        WALK_CACHE_HITS.with(|c| c.set(0));
        WALK_CACHE_MISSES.with(|c| c.set(0));
        let eval_total = eval_hits + eval_misses;
        let eval_pct = if eval_total > 0 {
            100.0 * eval_hits as f64 / eval_total as f64
        } else {
            0.0
        };
        log::info!(
            "{} quotient: {} -> {} bits ({:.1}% reduction) in {:?} ({} iters, eval {}/{} = {:.1}% hits)",
            match mode {
                QuotientMode::FingerprintSeed => "fingerprint",
                QuotientMode::Coarsest => "bisim",
            },
            total_before,
            total_after,
            if total_before > 0 {
                (1.0 - total_after as f64 / total_before as f64) * 100.0
            } else {
                0.0
            },
            start.elapsed(),
            iters,
            eval_hits,
            eval_total,
            eval_pct,
        );
    }
}

/// Walks a semantic case body symbolically, emitting bytes that distinguish
/// aspects when their case-body shape differs under the current partition.
/// Concrete sub-expressions are evaluated via `SemanticContext`; relation
/// calls and quantifiers are handled here because they need partition lookup
/// and range expansion respectively.
///
/// `walk` results are memoized per signature() call: walking the same
/// sub-expression with the same env restricted to its free vars produces the
/// same hash. The free-variable set per sub-expression is precomputed once at
/// construction time. The hot win is in deeply nested quantifier bodies
/// (e.g. str.replace's 5-level `(exists xlo ... (exists k ...))`) where most
/// inner conjuncts depend on only a subset of the enclosing quantifier vars
/// — the cache collapses the cross-product enumeration down to the unique
/// (free-var-restricted) bindings actually seen.
/// `'p` is the problem-AST lifetime carried inside `SemanticContext` (kept
/// separate from the borrow lifetime `'a`: the context is invariant in `'p`
/// since its op memo stores borrowed `&'p FuncSig`s).
struct Refiner<'a, 'p> {
    ctx: &'a SemanticContext<'p>,
    cases_by_domain: &'a HashMap<SemKey, HashMap<String, Case>>,
    domain_kinds: &'a HashMap<SemKey, SemanticKind>,
    sem_ids: &'a HashMap<SemKey, usize>,
    aspect_ids: &'a HashMap<AspectKey, usize>,
    cons_arg_sorts: HashMap<String, Vec<String>>,
    parent_sorts: HashMap<String, Vec<String>>,
    /// `expr_ptr → sorted list of free symbols`. Populated once at
    /// construction by walking every case body. A symbol is "free" in a
    /// sub-expression if it appears textually but is not bound by a
    /// quantifier *inside* that sub-expression (so quantifier bindings from
    /// enclosing scopes are kept — they appear in `env.values` at walk time
    /// and need to be part of the cache key).
    free_vars: HashMap<usize, Vec<String>>,
    /// Model universes for bare-sort quantifier ranges, keyed by
    /// (range-expr ptr, model). The universe is static, but the refinement
    /// loop rebuilt it per expansion — a fresh `Arc::from(name)` per element,
    /// millions of allocations per quotient. Values are the ctx `univ_cache`
    /// Arcs themselves — this memo only skips re-resolving the range expr.
    /// Nested so the hot probe needs no owned key: the outer usize (expr ptr)
    /// and inner &str (model name) both borrow from live data, while the old
    /// flat `(usize, String)` key materialized a String per expansion.
    range_cache: RefCell<HashMap<usize, HashMap<String, Arc<Vec<AspectValue>>>>>,
}

impl<'a, 'p> Refiner<'a, 'p> {
    fn new(
        problem: &LoomProg,
        ctx: &'a SemanticContext<'p>,
        cases_by_domain: &'a HashMap<SemKey, HashMap<String, Case>>,
        domain_kinds: &'a HashMap<SemKey, SemanticKind>,
        sem_ids: &'a HashMap<SemKey, usize>,
        aspect_ids: &'a HashMap<AspectKey, usize>,
    ) -> Self {
        let mut free_vars: HashMap<usize, Vec<String>> = HashMap::default();
        for cases in cases_by_domain.values() {
            for case in cases.values() {
                Self::compute_free_vars(&case.body, &mut free_vars);
            }
        }
        let mut cons_arg_sorts = HashMap::default();
        let mut parent_sorts: HashMap<String, Vec<String>> = HashMap::default();
        for cmd in &problem.cmds {
            let LoomCmd::TermTypes(term_types) = cmd else {
                continue;
            };
            for (sort_decl, type_decl) in term_types
                .sort_decls
                .iter()
                .zip(term_types.term_type_decls.iter())
            {
                let result_sort = sort_decl.symbol.clone();
                for cons in &type_decl.constructors {
                    let arg_sorts = cons
                        .sorts
                        .iter()
                        .map(|sort| sort.to_string())
                        .collect::<Vec<_>>();
                    for arg_sort in &arg_sorts {
                        let entry = parent_sorts.entry(arg_sort.clone()).or_default();
                        if !entry.contains(&result_sort) {
                            entry.push(result_sort.clone());
                        }
                    }
                    cons_arg_sorts.insert(cons.symbol.to_string(), arg_sorts);
                }
            }
        }
        for sorts in parent_sorts.values_mut() {
            sorts.sort();
            sorts.dedup();
        }
        Self {
            ctx,
            cases_by_domain,
            domain_kinds,
            sem_ids,
            aspect_ids,
            cons_arg_sorts,
            parent_sorts,
            free_vars,
            range_cache: RefCell::new(HashMap::default()),
        }
    }

    /// Returns the set of symbols appearing in `expr` that are *not* bound
    /// by a quantifier inside `expr`. The set is stored in `free_vars`
    /// keyed by the expression's pointer; recursive sub-expressions get
    /// stored as a side effect.
    ///
    /// Symbols that aren't actually quantifier bindings (function names,
    /// reserved words, aspect field names, etc.) get included too — they're
    /// harmless because the cache-key construction looks them up in
    /// `env.values` and skips any that aren't present.
    fn compute_free_vars(
        expr: &Value,
        free_vars: &mut HashMap<usize, Vec<String>>,
    ) -> HashSet<String> {
        let expr_ptr = expr as *const Value as usize;
        let result: HashSet<String> = if let Some(s) = expr.as_symbol() {
            let mut set = HashSet::default();
            set.insert(s.to_string());
            set
        } else if let Some(items) = expr.to_ref_vec() {
            let quant_op = items
                .first()
                .and_then(|v| v.as_symbol())
                .filter(|op| matches!(*op, "exists" | "forall"));
            // For (exists ((name range)) body): items[1] = ((name range)), items[2] = body.
            // Try to extract the bound name; on any structural mismatch, fall through to the
            // generic "union of children" path so the analysis stays conservative.
            let bound_name = if quant_op.is_some() && items.len() == 3 {
                items[1]
                    .to_ref_vec()
                    .and_then(|bindings| bindings.first().and_then(|b| b.to_ref_vec()))
                    .filter(|parts| parts.len() == 2)
                    .and_then(|parts| parts[0].as_symbol().map(str::to_string))
            } else {
                None
            };
            if let Some(name) = bound_name {
                // We know items[1] parses as ((name range)). Pull out the
                // range expression as a direct sub-reference of items[1].
                // Walk children for free vars; afterwards, remove the
                // bound name from the body's free set.
                let mut result = HashSet::default();
                // Range expression and body both contribute, but the
                // quantifier's binding name shadows references in body.
                if let Some(bindings) = items[1].to_ref_vec() {
                    if let Some(first_binding) = bindings.first() {
                        if let Some(parts) = first_binding.to_ref_vec() {
                            if parts.len() == 2 {
                                let range_free = Self::compute_free_vars(parts[1], free_vars);
                                result.extend(range_free);
                            }
                        }
                    }
                }
                let body_free = Self::compute_free_vars(items[2], free_vars);
                result.extend(body_free);
                result.remove(&name);
                result
            } else {
                let mut result = HashSet::default();
                for child in items.iter() {
                    let child_free = Self::compute_free_vars(child, free_vars);
                    result.extend(child_free);
                }
                result
            }
        } else {
            HashSet::default()
        };
        let mut sorted: Vec<String> = result.iter().cloned().collect();
        sorted.sort();
        free_vars.insert(expr_ptr, sorted);
        result
    }
}

#[derive(Clone)]
struct WalkEnv {
    aspects: HashMap<String, AspectKey>,  // "asp" → current aspect
    values: HashMap<String, AspectValue>, // quantifier bindings
    subterms: HashMap<String, usize>,     // constructor pattern vars
    subterm_sorts: HashMap<String, String>,
    current_sort: String,
    current_model: Option<String>,
}

impl EnvView for WalkEnv {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        &self.aspects
    }
    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
    fn current_model(&self) -> Option<&str> {
        self.current_model.as_deref()
    }
}

const SELF_SUBTERM: usize = usize::MAX;

impl<'a, 'p> Refiner<'a, 'p> {
    fn signature(
        &self,
        sem: &SemKey,
        aspect: &AspectKey,
        value: &Option<AspectValue>,
        classes: &HashMap<DenseClassKey, ClassId>,
        mut env: &mut WalkEnv,
        mut cache: &mut WalkCache,
    ) -> (SigHash, Vec<ClassId>) {
        let mut out = FxHasher::default();
        let mut deps = Vec::new();
        out.write(sem.relation.as_bytes());
        out.write_u8(b'/');
        out.write(sem.term_sort.as_bytes());
        out.write_u8(b'#');
        if let Some(v) = value {
            self.emit_value(&mut out, v);
        }
        out.write_u8(b'@');
        let Some(cases) = self.cases_by_domain.get(sem) else {
            return (out.finish(), deps);
        };
        let mut names: Vec<&str> = cases.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        // Env is reused across cases and calls. `values` starts empty;
        // emit_quantifier restores bindings on the way out so it stays
        // empty between cases. `aspects` doesn't change inside walk.
        // `subterms` is rebuilt per case.
        env.aspects.clear();
        env.values.clear();
        env.subterms.clear();
        env.subterm_sorts.clear();
        env.current_sort = sem.term_sort.clone();
        env.current_model = Some(aspect.model.clone());
        let aspect_var = cases
            .values()
            .next()
            .map(|case| case.aspect_var.clone())
            .unwrap_or_else(|| "asp".to_string());
        env.aspects.insert(aspect_var, aspect.clone());
        // Walk cache: contents scoped to this call — cleared here, because
        // across calls `aspect` and `classes` change and stale hashes would
        // be unsound. Within a call the current aspect is fixed, so a cached
        // hash for (expr_ptr, env_values_restricted_to_free_vars) is sound.
        cache.clear();
        for cons in names {
            let case = &cases[cons];
            out.write(cons.as_bytes());
            out.write_u8(b'{');
            env.subterms.clear();
            env.subterm_sorts.clear();
            env.subterms.insert(case.self_var.clone(), SELF_SUBTERM);
            env.subterm_sorts
                .insert(case.self_var.clone(), sem.term_sort.clone());
            let arg_sorts = self.cons_arg_sorts.get(cons);
            for (i, v) in case.vars.iter().enumerate() {
                env.subterms.insert(v.clone(), i);
                if let Some(sort) = arg_sorts.and_then(|sorts| sorts.get(i)) {
                    env.subterm_sorts.insert(v.clone(), sort.clone());
                }
            }
            let case_hash = self.walk_hash(&case.body, &mut env, classes, &mut cache, &mut deps);
            out.write_u64(case_hash);
            out.write_u8(b'}');
        }
        deps.sort_unstable();
        deps.dedup();
        (out.finish(), deps)
    }

    /// Returns true iff the expression is worth caching. We cache every
    /// list expression and skip atoms. Atom walks are faster than the
    /// cache-key construction + HashMap probe, and an atom's value is
    /// fully determined by its single free var so the cache wouldn't add
    /// information anyway.
    ///
    /// I tried being more selective (skip caching for combiners like
    /// `and`/`or`/`exists` since their hit rate is near 0%), but it
    /// regressed test_col 2× — caching the combiner short-circuits the
    /// recursive walk-and-collect work on subsequent visits even when the
    /// outer key only collides occasionally, and skipping it forces the
    /// inner walks to happen far more often than the hit-rate suggests.
    fn worth_caching(expr: &Value) -> bool {
        expr.as_slice().is_some()
    }

    /// Cache wrapper for `walk`. The first call for a given
    /// (expr_ptr, free-var bindings) writes a fresh hash into the cache;
    /// later calls with the same key short-circuit. Bypasses the cache
    /// for cheap sub-expressions (see `worth_caching`) where the
    /// bookkeeping cost would dominate the saved work.
    fn walk_hash(
        &self,
        expr: &Value,
        env: &mut WalkEnv,
        classes: &HashMap<DenseClassKey, ClassId>,
        cache: &mut WalkCache,
        deps: &mut Vec<ClassId>,
    ) -> u64 {
        if !Self::worth_caching(expr) {
            let mut h = FxHasher::default();
            self.walk(&mut h, expr, env, classes, cache, deps);
            return h.finish();
        }
        let expr_ptr = expr as *const Value as usize;
        let env_h = self.env_key(expr_ptr, env);
        let key = (expr_ptr, env_h);
        if let Some((hash, cached_deps)) = cache.get(&key) {
            WALK_CACHE_HITS.with(|c| c.set(c.get() + 1));
            deps.extend(cached_deps.iter().copied());
            return *hash;
        }
        WALK_CACHE_MISSES.with(|c| c.set(c.get() + 1));
        let mut h = FxHasher::default();
        let mut local_deps = Vec::new();
        self.walk(&mut h, expr, env, classes, cache, &mut local_deps);
        let hash = h.finish();
        deps.extend(local_deps.iter().copied());
        cache.insert(key, (hash, local_deps));
        hash
    }

    /// Builds a u64 fingerprint of the bindings in `env.values` for the
    /// free variables of `expr_ptr`. Symbols listed in `free_vars` that
    /// aren't present in `env.values` are silently skipped — they're
    /// either function names, declared funcs, or other non-binding
    /// references, none of which affect the walk hash.
    fn env_key(&self, expr_ptr: usize, env: &WalkEnv) -> u64 {
        let mut h = FxHasher::default();
        if let Some(free) = self.free_vars.get(&expr_ptr) {
            for name in free {
                if let Some(v) = env.values.get(name) {
                    h.write(name.as_bytes());
                    h.write_u8(b'=');
                    self.emit_value(&mut h, v);
                    h.write_u8(b';');
                }
            }
        }
        h.finish()
    }

    fn walk<H: Hasher>(
        &self,
        out: &mut H,
        expr: &Value,
        env: &mut WalkEnv,
        classes: &HashMap<DenseClassKey, ClassId>,
        cache: &mut WalkCache,
        deps: &mut Vec<ClassId>,
    ) {
        // Use `as_slice` instead of `to_ref_vec` — the latter allocates a
        // `Vec<&SExpr>` per call. `walk` is called 20M+ times on test_col
        // bisim, so the saved heap traffic adds up.
        let Some(items) = expr.as_slice() else {
            // Atom: `eval_value` covers literals and bound symbols; fall
            // back to `eval_bool_value` for the literal `true`/`false`
            // cases it doesn't already subsume.
            if self.try_emit_value(out, expr, env) {
                return;
            }
            if self.try_emit_bool(out, expr, env) {
                return;
            }
            out.write_u8(b'?');
            out.write(format!("{expr}").as_bytes());
            return;
        };
        let Some(op) = items.first().and_then(|v| v.as_symbol()) else {
            out.write_u8(b'?');
            return;
        };

        // Relation calls and quantifiers always need bespoke handling.
        // classify_op: memoized by op-node ptr — the string-keyed
        // `has_relation` ran on all 20M+ walk calls.
        if self.ctx.classify_op(&items[0], op).0 && items.len() >= 3 {
            self.emit_relation(out, op, &items[1], &items[2], env, classes, deps);
            return;
        }
        if matches!(op, "exists" | "forall") && items.len() == 3 {
            self.emit_quantifier(out, op, &items[1], &items[2], env, classes, cache, deps);
            return;
        }

        // For every other op, dispatch on `op`: try only the evaluator
        // that has a matching arm for this shape, then fall through to
        // structural recursion when the expression isn't fully grounded.
        // Previously we tried both `eval_value` and `eval_bool_value`
        // blindly; that wasted ~80% of `eval_value` probes (no `and`/
        // `or`/`not`/`=` arms) and ~67% of `eval_bool_value` probes (no
        // `+`/`-`/`size`/model-fn-call arms) on test_col.
        match op {
            "and" => {
                // Paper δ: concrete (constant) conjuncts evaluate immediately.
                // A `true` conjunct is ∧-identity and drops; a `false` conjunct
                // annihilates; only the dynamic conjuncts (state references)
                // remain. Sort+dedup so the signature is the *set* of operands
                // (∧ is commutative and idempotent).
                let mut dyn_hashes = Vec::new();
                for arg in &items[1..] {
                    match self.ctx.eval_bool_value(arg, env) {
                        Ok(true) => {}
                        Ok(false) => {
                            out.write_u8(b'F');
                            return;
                        }
                        Err(_) => dyn_hashes.push(self.walk_hash(arg, env, classes, cache, deps)),
                    }
                }
                dyn_hashes.sort_unstable();
                dyn_hashes.dedup();
                out.write_u8(b'A');
                out.write_u8(b'(');
                for sub in dyn_hashes {
                    out.write_u64(sub);
                    out.write_u8(b',');
                }
                out.write_u8(b')');
            }
            "or" => {
                // Dual of `and`: `false` drops (∨-identity), `true` annihilates.
                let mut dyn_hashes = Vec::new();
                for arg in &items[1..] {
                    match self.ctx.eval_bool_value(arg, env) {
                        Ok(false) => {}
                        Ok(true) => {
                            out.write_u8(b'T');
                            return;
                        }
                        Err(_) => dyn_hashes.push(self.walk_hash(arg, env, classes, cache, deps)),
                    }
                }
                dyn_hashes.sort_unstable();
                dyn_hashes.dedup();
                out.write_u8(b'O');
                out.write_u8(b'(');
                for sub in dyn_hashes {
                    out.write_u64(sub);
                    out.write_u8(b',');
                }
                out.write_u8(b')');
            }
            "not" if items.len() == 2 => {
                if self.try_emit_bool(out, expr, env) {
                    return;
                }
                out.write_u8(b'N');
                out.write_u8(b'(');
                let sub = self.walk_hash(&items[1], env, classes, cache, deps);
                out.write_u64(sub);
                out.write_u8(b')');
            }
            "ite" if items.len() == 4 => {
                // `ite` straddles both evaluators (value-typed vs bool-typed
                // branches), so try value first to match the previous order.
                if self.try_emit_value(out, expr, env) {
                    return;
                }
                if self.try_emit_bool(out, expr, env) {
                    return;
                }
                out.write_u8(b'I');
                out.write_u8(b'(');
                let a = self.walk_hash(&items[1], env, classes, cache, deps);
                let b = self.walk_hash(&items[2], env, classes, cache, deps);
                let c = self.walk_hash(&items[3], env, classes, cache, deps);
                out.write_u64(a);
                out.write_u8(b',');
                out.write_u64(b);
                out.write_u8(b',');
                out.write_u64(c);
                out.write_u8(b')');
            }
            "=" | "<" | "<=" if items.len() == 3 => {
                if self.try_emit_bool(out, expr, env) {
                    return;
                }
                out.write_u8(b'=');
                out.write(op.as_bytes());
                out.write_u8(b'(');
                let a = self.walk_hash(&items[1], env, classes, cache, deps);
                let b = self.walk_hash(&items[2], env, classes, cache, deps);
                out.write_u64(a);
                out.write_u8(b',');
                out.write_u64(b);
                out.write_u8(b')');
            }
            _ => {
                // Model-fn call, aspect field accessor, or `+`/`-`/`size`
                // — all value-typed; only `eval_value` has matching arms.
                if self.try_emit_value(out, expr, env) {
                    return;
                }
                out.write_u8(b'(');
                out.write(op.as_bytes());
                for arg in &items[1..] {
                    out.write_u8(b' ');
                    let sub = self.walk_hash(arg, env, classes, cache, deps);
                    out.write_u64(sub);
                }
                out.write_u8(b')');
            }
        }
    }

    fn try_emit_value<H: Hasher>(&self, out: &mut H, expr: &Value, env: &WalkEnv) -> bool {
        if let Ok(v) = self.ctx.eval_value(expr, env) {
            out.write_u8(b'V');
            self.emit_value(out, &v);
            true
        } else {
            false
        }
    }

    fn try_emit_bool<H: Hasher>(&self, out: &mut H, expr: &Value, env: &WalkEnv) -> bool {
        if let Ok(b) = self.ctx.eval_bool_value(expr, env) {
            out.write_u8(if b { b'T' } else { b'F' });
            true
        } else {
            false
        }
    }

    fn emit_relation<H: Hasher>(
        &self,
        out: &mut H,
        relation: &str,
        subterm_expr: &Value,
        aspect_expr: &Value,
        env: &mut WalkEnv,
        classes: &HashMap<DenseClassKey, ClassId>,
        deps: &mut Vec<ClassId>,
    ) {
        out.write_u8(b'R');
        out.write(relation.as_bytes());
        out.write_u8(b'@');
        self.emit_term_nav(out, subterm_expr, env);
        out.write_u8(b':');
        match self.ctx.eval_aspect(aspect_expr, relation, env) {
            Ok(asp) => {
                let mut targets = self.target_sorts(relation, subterm_expr, env);
                targets.sort();
                targets.dedup();
                out.write_u8(b'[');
                for target_sort in targets {
                    let sem = SemKey::new(relation, &target_sort);
                    out.write(target_sort.as_bytes());
                    out.write_u8(b'=');
                    match self.domain_kinds.get(&sem) {
                        Some(SemanticKind::Bool) => {
                            let class = self
                                .sem_ids
                                .get(&sem)
                                .and_then(|&sem_id| {
                                    self.aspect_ids.get(&asp).and_then(|&aspect_id| {
                                        classes.get(&(sem_id, aspect_id, None))
                                    })
                                })
                                .copied()
                                .unwrap_or(usize::MAX);
                            if class != usize::MAX {
                                deps.push(class);
                            }
                            out.write_u8(b'C');
                            out.write_u64(class as u64);
                        }
                        Some(_) => {
                            // FiniteValue relations: the call returns a value,
                            // not a boolean, so the surrounding context picks the
                            // (asp, value) tuple. Emit the target sort and aspect
                            // bytes directly rather than a class id.
                            out.write_u8(b'*');
                            out.write(asp.model.as_bytes());
                            out.write_u8(b'|');
                            for (k, v) in &asp.fields {
                                out.write(k.as_bytes());
                                out.write_u8(b'=');
                                self.emit_value(out, v);
                                out.write_u8(b';');
                            }
                        }
                        None => out.write_u8(b'?'),
                    }
                    out.write_u8(b';');
                }
                out.write_u8(b']');
            }
            Err(_) => out.write_u8(b'?'),
        }
    }

    fn emit_term_nav<H: Hasher>(&self, out: &mut H, expr: &Value, env: &WalkEnv) {
        if let Some(subterm) = expr.as_symbol() {
            match env.subterms.get(subterm).copied() {
                Some(SELF_SUBTERM) => out.write_u8(b'r'),
                Some(i) => {
                    out.write_u8(b'#');
                    out.write_u64(i as u64);
                }
                None => out.write_u8(b'?'),
            }
            return;
        }
        let Some(items) = expr.as_slice() else {
            out.write_u8(b'?');
            return;
        };
        match items {
            [op, inner] if op.as_symbol() == Some("parent") && inner.as_symbol().is_some() => {
                out.write_u8(b'^');
            }
            _ => out.write_u8(b'?'),
        }
    }

    fn target_sorts(&self, relation: &str, expr: &Value, env: &WalkEnv) -> Vec<String> {
        let mut sorts = if let Some(subterm) = expr.as_symbol() {
            env.subterm_sorts
                .get(subterm)
                .cloned()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            match expr.as_slice() {
                Some([op, inner])
                    if op.as_symbol() == Some("parent") && inner.as_symbol().is_some() =>
                {
                    self.parent_sorts
                        .get(&env.current_sort)
                        .cloned()
                        .unwrap_or_default()
                }
                _ => Vec::new(),
            }
        };
        sorts.retain(|sort| {
            self.domain_kinds
                .contains_key(&SemKey::new(relation, sort.as_str()))
        });
        sorts
    }

    fn emit_quantifier<H: Hasher>(
        &self,
        out: &mut H,
        quant: &str,
        binding: &Value,
        body: &Value,
        env: &mut WalkEnv,
        classes: &HashMap<DenseClassKey, ClassId>,
        cache: &mut WalkCache,
        deps: &mut Vec<ClassId>,
    ) {
        out.write_u8(if quant == "exists" { b'E' } else { b'F' });
        out.write_u8(b'{');
        // `as_slice` not `to_ref_vec`, and the name stays borrowed: this parse
        // ran once per emit_quantifier call (millions per quotient) and the two
        // Vec allocations plus the owned name String were pure hit-path cost.
        let parsed = binding
            .as_slice()
            .and_then(|bs| bs.first().and_then(|b| b.as_slice()))
            .filter(|parts| parts.len() == 2)
            .and_then(|parts| parts[0].as_symbol().map(|n| (n, &parts[1])));
        if let Some((name, range_expr)) = parsed {
            // Paper δ: `∃v:σ.E` / `∀v:σ.E` expand to `⋁_{d∈D_σ} E[v↦d]` /
            // `⋀_{d} E[v↦d]`, so the bound witness `d` disappears. A body that
            // evaluates to a constant folds: for ∃, `false` witnesses drop
            // (∨-identity) and any `true` witness annihilates to ⊤; dually ∀.
            // The surviving dynamic bodies form a set (sort+dedup), never keyed
            // by the witness value — so states differing only in absolute
            // witness positions merge.
            let mut dyn_hashes = Vec::new();
            let mut annihilated = false;
            let range = self.expand_range(range_expr, env);
            // Bind the key once and overwrite its slot per value: inserting
            // per iteration re-allocates the key String every time, and the
            // save/restore dance folds into one pair around the loop.
            let prev = env.values.get(name).cloned();
            for val in range.iter() {
                match env.values.get_mut(name) {
                    Some(slot) => *slot = val.clone(),
                    None => {
                        env.values.insert(name.to_string(), val.clone());
                    }
                }
                match (quant, self.ctx.eval_bool_value(body, env)) {
                    ("exists", Ok(true)) | ("forall", Ok(false)) => annihilated = true,
                    ("exists", Ok(false)) | ("forall", Ok(true)) => {}
                    _ => dyn_hashes.push(self.walk_hash(body, env, classes, cache, deps)),
                }
                if annihilated {
                    break;
                }
            }
            match prev {
                Some(p) => {
                    env.values.insert(name.to_string(), p);
                }
                None => {
                    env.values.remove(name);
                }
            }
            if annihilated {
                out.write_u8(if quant == "exists" { b'T' } else { b'F' });
            } else {
                dyn_hashes.sort_unstable();
                dyn_hashes.dedup();
                for sub in dyn_hashes {
                    out.write_u64(sub);
                    out.write_u8(b',');
                }
            }
        } else {
            out.write_u8(b'?');
        }
        out.write_u8(b'}');
    }

    fn expand_range(&self, expr: &Value, env: &WalkEnv) -> Arc<Vec<AspectValue>> {
        // Bare sort symbol: resolve through the current model, matching the
        // compiler's and CHC backends' handling of `(forall ((k Pos)) …)`.
        if expr.as_symbol().is_some() {
            // Probe with both key halves borrowed: for a bare sort the model
            // is exactly the ambient current_model (what parse_univ_expr
            // resolves), so hits allocate nothing. parse_univ_expr runs only
            // on a miss, to own the model/sort Strings for the insert key.
            let ptr = expr as *const Value as usize;
            if let Ok(model_name) = self.ctx.current_model(env) {
                if let Some(cached) = self
                    .range_cache
                    .borrow()
                    .get(&ptr)
                    .and_then(|by_model| by_model.get(model_name))
                {
                    return cached.clone();
                }
            }
            let Ok((model_name, sort)) = self.ctx.parse_univ_expr(expr, env) else {
                return Arc::new(Vec::new());
            };
            let Ok(model) = self.ctx.model(&model_name) else {
                return Arc::new(Vec::new());
            };
            // Share the ctx universe — same Arc the eval path converts
            // through; no second build, no per-element Arc::from.
            let Some(values) = self.ctx.univ_values(model, &sort) else {
                return Arc::new(Vec::new());
            };
            self.range_cache
                .borrow_mut()
                .entry(ptr)
                .or_default()
                .insert(model_name, values.clone());
            return values;
        }
        let Some(items) = expr.as_slice() else {
            return Arc::new(Vec::new());
        };
        let Some(op) = items.first().and_then(|v| v.as_symbol()) else {
            return Arc::new(Vec::new());
        };
        match op {
            "int-range" if items.len() == 3 => {
                // Mirror compiler.rs / chc.rs: inclusive upper bound. The
                // compositional formulas emitted by the compiler enumerate
                // k ∈ [lo, hi], so the bisim signature must use the same
                // range or the partition won't reflect the formulas the
                // synthesizer actually sees.
                let lo = self.ctx.eval_int_expr(&items[1], env).unwrap_or(0).max(0) as usize;
                let hi = self.ctx.eval_int_expr(&items[2], env).unwrap_or(0).max(0) as usize;
                if hi < lo {
                    Arc::new(Vec::new())
                } else {
                    Arc::new((lo..=hi).map(AspectValue::Int).collect())
                }
            }
            _ => Arc::new(Vec::new()),
        }
    }

    fn emit_value<H: Hasher>(&self, out: &mut H, v: &AspectValue) {
        match v {
            AspectValue::Int(i) => {
                out.write_u8(b'#');
                out.write_u64(*i as u64);
            }
            AspectValue::Model(m) => {
                out.write_u8(b'M');
                out.write(m.as_bytes());
            }
            AspectValue::Symbol(s) => {
                out.write_u8(b'S');
                out.write(s.as_bytes());
            }
            AspectValue::Univ { sort, name } => {
                out.write_u8(b'U');
                out.write(sort.as_bytes());
                out.write_u8(b':');
                out.write(name.as_bytes());
            }
        }
    }
}
