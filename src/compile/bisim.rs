use crate::compile::eval_ctx::{EnvView, SemanticContext};
use crate::compile::semantics::{parse_cases, Case};
use crate::compile::types::{AspectKey, AspectValue, DomainBit, SemanticKind, SemanticProgram};
use crate::lang::model::{InterpedValue, Model};
use crate::parser::DelisProg;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Instant;
use sygus_parser::Literal;
use sygus_parser::SExpr as Value;

type Fingerprint = Vec<u8>;
type ClassId = usize;
type ClassKey = (String, AspectKey, Option<AspectValue>);
type SigHash = u64;

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
    pub fn quotient_by_bisim(&mut self, problem: &DelisProg) {
        let start = Instant::now();
        let total_before: usize = self.domains.iter().map(|d| d.bits.len()).sum();
        if total_before == 0 {
            return;
        }

        let mut cases_by_relation: HashMap<String, HashMap<String, Case>> = HashMap::default();
        for spec in &self.semantics {
            if let Ok(cases) = parse_cases(&spec.body, &spec.aspect_var) {
                let entry = cases_by_relation.entry(spec.relation.clone()).or_default();
                for (cons, case) in cases {
                    entry.insert(cons, case);
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

        let func_names: Vec<String> = ctx.func_sigs.keys().cloned().collect();

        // Initial partition: leaf fingerprint per (relation, value).
        let mut classes: HashMap<ClassKey, ClassId> = HashMap::default();
        let mut leaf_to_class: HashMap<(String, Fingerprint, Option<AspectValue>), ClassId> =
            HashMap::default();
        for domain in &self.domains {
            for bit in &domain.bits {
                let fp = ctx
                    .models
                    .get(&bit.aspect.model)
                    .map(|m| bit.aspect.fingerprint(m, &func_names))
                    .unwrap_or_default();
                let key = (domain.relation.clone(), fp, bit.value.clone());
                let next_id = leaf_to_class.len();
                let class = *leaf_to_class.entry(key).or_insert(next_id);
                classes.insert(
                    (
                        domain.relation.clone(),
                        bit.aspect.clone(),
                        bit.value.clone(),
                    ),
                    class,
                );
            }
        }

        let domain_kinds: HashMap<String, SemanticKind> = self
            .domains
            .iter()
            .map(|d| (d.relation.clone(), d.kind.clone()))
            .collect();
        let refiner = Refiner::new(&ctx, &cases_by_relation, &domain_kinds);

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
        let mut class_members: HashMap<ClassId, Vec<ClassKey>> = HashMap::default();
        for (key, &cid) in &classes {
            class_members.entry(cid).or_default().push(key.clone());
        }
        let mut multi_aspect: Vec<ClassId> = class_members
            .iter()
            .filter(|(_, members)| members.len() > 1)
            .map(|(&id, _)| id)
            .collect();
        let mut next_class_id = classes.values().max().copied().map(|m| m + 1).unwrap_or(0);

        let mut iters = 0usize;
        const MAX_ITERS: usize = 64;
        while !multi_aspect.is_empty() {
            iters += 1;
            let mut any_split = false;
            let mut next_multi: Vec<ClassId> = Vec::new();
            for cid in std::mem::take(&mut multi_aspect) {
                let members = match class_members.get(&cid) {
                    Some(m) if m.len() > 1 => m.clone(),
                    _ => continue,
                };
                // Group members of this class by their refined signature.
                let mut by_sig: HashMap<SigHash, Vec<ClassKey>> = HashMap::default();
                for member in &members {
                    let sig = refiner.signature(&member.0, &member.1, &member.2, &classes);
                    by_sig.entry(sig).or_default().push(member.clone());
                }
                if by_sig.len() == 1 {
                    // No split: the class is stable under current partition.
                    // It might still split in a later iter if a referenced
                    // class splits, so keep it in the worklist.
                    next_multi.push(cid);
                    continue;
                }
                any_split = true;
                // First group keeps `cid`; remaining groups get fresh IDs.
                let mut groups: Vec<Vec<ClassKey>> = by_sig.into_values().collect();
                groups.sort_by(|a, b| a.first().cmp(&b.first()));
                let first = groups.remove(0);
                if first.len() > 1 {
                    next_multi.push(cid);
                }
                class_members.insert(cid, first);
                for group in groups {
                    let new_id = next_class_id;
                    next_class_id += 1;
                    for key in &group {
                        classes.insert(key.clone(), new_id);
                    }
                    if group.len() > 1 {
                        next_multi.push(new_id);
                    }
                    class_members.insert(new_id, group);
                }
            }
            if !any_split {
                // Stable: this iter produced no splits, so no referenced
                // class can change either. Done.
                break;
            }
            multi_aspect = next_multi;
            if iters >= MAX_ITERS {
                log::warn!(
                    "bisim: refinement did not converge in {} iterations; using current partition",
                    MAX_ITERS
                );
                break;
            }
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
                let class = classes
                    .get(&(
                        domain.relation.clone(),
                        bit.aspect.clone(),
                        bit.value.clone(),
                    ))
                    .copied()
                    .unwrap_or(usize::MAX);
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

        let hits = WALK_CACHE_HITS.with(|c| c.replace(0));
        let misses = WALK_CACHE_MISSES.with(|c| c.replace(0));
        let total_calls = hits + misses;
        let hit_pct = if total_calls > 0 {
            100.0 * hits as f64 / total_calls as f64
        } else {
            0.0
        };
        let eval_total = eval_hits + eval_misses;
        let eval_pct = if eval_total > 0 {
            100.0 * eval_hits as f64 / eval_total as f64
        } else {
            0.0
        };
        log::info!(
            "bisim quotient: {} -> {} bits ({:.1}% reduction) in {:?} ({} iters, walk {}/{} = {:.1}% hits, eval {}/{} = {:.1}% hits)",
            total_before,
            total_after,
            if total_before > 0 {
                (1.0 - total_after as f64 / total_before as f64) * 100.0
            } else {
                0.0
            },
            start.elapsed(),
            iters,
            hits,
            total_calls,
            hit_pct,
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
struct Refiner<'a> {
    ctx: &'a SemanticContext<'a>,
    cases_by_relation: &'a HashMap<String, HashMap<String, Case>>,
    domain_kinds: &'a HashMap<String, SemanticKind>,
    /// `expr_ptr → sorted list of free symbols`. Populated once at
    /// construction by walking every case body. A symbol is "free" in a
    /// sub-expression if it appears textually but is not bound by a
    /// quantifier *inside* that sub-expression (so quantifier bindings from
    /// enclosing scopes are kept — they appear in `env.values` at walk time
    /// and need to be part of the cache key).
    free_vars: HashMap<usize, Vec<String>>,
}

impl<'a> Refiner<'a> {
    fn new(
        ctx: &'a SemanticContext<'a>,
        cases_by_relation: &'a HashMap<String, HashMap<String, Case>>,
        domain_kinds: &'a HashMap<String, SemanticKind>,
    ) -> Self {
        let mut free_vars: HashMap<usize, Vec<String>> = HashMap::default();
        for cases in cases_by_relation.values() {
            for case in cases.values() {
                Self::compute_free_vars(&case.body, &mut free_vars);
            }
        }
        Self {
            ctx,
            cases_by_relation,
            domain_kinds,
            free_vars,
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
}

impl EnvView for WalkEnv {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        &self.aspects
    }
    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
}

const SELF_SUBTERM: usize = usize::MAX;

impl<'a> Refiner<'a> {
    fn signature(
        &self,
        relation: &str,
        aspect: &AspectKey,
        value: &Option<AspectValue>,
        classes: &HashMap<ClassKey, ClassId>,
    ) -> SigHash {
        let mut out = FxHasher::default();
        out.write(relation.as_bytes());
        out.write_u8(b'#');
        if let Some(v) = value {
            self.emit_value(&mut out, v);
        }
        out.write_u8(b'@');
        let Some(cases) = self.cases_by_relation.get(relation) else {
            return out.finish();
        };
        let mut names: Vec<&str> = cases.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        // The env is reused across cases. `values` starts empty; emit_quantifier
        // restores bindings on the way out so it stays empty between cases.
        // `aspects` doesn't change inside walk. `subterms` is rebuilt per case.
        let mut env = WalkEnv {
            aspects: HashMap::default(),
            values: HashMap::default(),
            subterms: HashMap::default(),
        };
        let aspect_var = cases
            .values()
            .next()
            .map(|case| case.aspect_var.clone())
            .unwrap_or_else(|| "asp".to_string());
        env.aspects.insert(aspect_var, aspect.clone());
        // Walk cache: local to this signature() call. The current aspect
        // is fixed for the duration, so a cached hash for a given
        // (expr_ptr, env_values_restricted_to_free_vars) is sound. Lives
        // on the stack so it's dropped at end-of-call — across calls,
        // `aspect` and `classes` change and we'd need to re-walk anyway.
        let mut cache: HashMap<WalkKey, u64> = HashMap::default();
        for cons in names {
            let case = &cases[cons];
            out.write(cons.as_bytes());
            out.write_u8(b'{');
            env.subterms.clear();
            env.subterms.insert(case.self_var.clone(), SELF_SUBTERM);
            for (i, v) in case.vars.iter().enumerate() {
                env.subterms.insert(v.clone(), i);
            }
            let case_hash = self.walk_hash(&case.body, &mut env, classes, &mut cache);
            out.write_u64(case_hash);
            out.write_u8(b'}');
        }
        out.finish()
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
        classes: &HashMap<ClassKey, ClassId>,
        cache: &mut HashMap<WalkKey, u64>,
    ) -> u64 {
        if !Self::worth_caching(expr) {
            let mut h = FxHasher::default();
            self.walk(&mut h, expr, env, classes, cache);
            return h.finish();
        }
        let expr_ptr = expr as *const Value as usize;
        let env_h = self.env_key(expr_ptr, env);
        let key = (expr_ptr, env_h);
        if let Some(&hash) = cache.get(&key) {
            WALK_CACHE_HITS.with(|c| c.set(c.get() + 1));
            return hash;
        }
        WALK_CACHE_MISSES.with(|c| c.set(c.get() + 1));
        let mut h = FxHasher::default();
        self.walk(&mut h, expr, env, classes, cache);
        let hash = h.finish();
        cache.insert(key, hash);
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
        classes: &HashMap<ClassKey, ClassId>,
        cache: &mut HashMap<WalkKey, u64>,
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
        if self.ctx.has_relation(op) && items.len() >= 3 {
            self.emit_relation(out, op, &items[1], &items[2], env, classes);
            return;
        }
        if matches!(op, "exists" | "forall") && items.len() == 3 {
            self.emit_quantifier(out, op, &items[1], &items[2], env, classes, cache);
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
            "and" | "or" => {
                if self.try_emit_bool(out, expr, env) {
                    return;
                }
                out.write_u8(if op == "and" { b'A' } else { b'O' });
                out.write_u8(b'(');
                for arg in &items[1..] {
                    let sub = self.walk_hash(arg, env, classes, cache);
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
                let sub = self.walk_hash(&items[1], env, classes, cache);
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
                let a = self.walk_hash(&items[1], env, classes, cache);
                let b = self.walk_hash(&items[2], env, classes, cache);
                let c = self.walk_hash(&items[3], env, classes, cache);
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
                let a = self.walk_hash(&items[1], env, classes, cache);
                let b = self.walk_hash(&items[2], env, classes, cache);
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
                    let sub = self.walk_hash(arg, env, classes, cache);
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
        classes: &HashMap<ClassKey, ClassId>,
    ) {
        out.write_u8(b'R');
        out.write(relation.as_bytes());
        out.write_u8(b'@');
        if let Some(subterm) = subterm_expr.as_symbol() {
            match env.subterms.get(subterm).copied() {
                Some(SELF_SUBTERM) => out.write_u8(b'r'),
                Some(i) => {
                    out.write_u8(b'#');
                    out.write_u64(i as u64);
                }
                None => out.write_u8(b'?'),
            }
        } else {
            out.write_u8(b'?');
        }
        out.write_u8(b':');
        match self.ctx.eval_aspect(aspect_expr, relation, env) {
            Ok(asp) => match self.domain_kinds.get(relation) {
                Some(SemanticKind::Bool) => {
                    let class = classes
                        .get(&(relation.to_string(), asp, None))
                        .copied()
                        .unwrap_or(usize::MAX);
                    out.write_u8(b'C');
                    out.write_u64(class as u64);
                }
                _ => {
                    // FiniteValue relations: the call returns a value, not a
                    // boolean, so the surrounding context picks the (asp,
                    // value) tuple. Emit the aspect bytes directly rather
                    // than a class id — never aliases distinct aspects (sound
                    // but coarser than the Bool case).
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
            },
            Err(_) => out.write_u8(b'?'),
        }
    }

    fn emit_quantifier<H: Hasher>(
        &self,
        out: &mut H,
        quant: &str,
        binding: &Value,
        body: &Value,
        env: &mut WalkEnv,
        classes: &HashMap<ClassKey, ClassId>,
        cache: &mut HashMap<WalkKey, u64>,
    ) {
        out.write_u8(if quant == "exists" { b'E' } else { b'F' });
        out.write_u8(b'{');
        let parsed = binding
            .to_ref_vec()
            .and_then(|bs| bs.first().and_then(|b| b.to_ref_vec()))
            .filter(|parts| parts.len() == 2)
            .and_then(|parts| parts[0].as_symbol().map(|n| (n.to_string(), parts[1])));
        if let Some((name, range_expr)) = parsed {
            // Push the binding before walking the body, then restore. Avoids
            // cloning the whole WalkEnv per iteration (hot for nested quants).
            // The body is hashed via walk_hash so that repeated bindings of
            // the bound var with the same free-var snapshot reuse cached
            // results.
            for val in self.expand_range(range_expr, env) {
                let prev = env.values.insert(name.clone(), val.clone());
                out.write_u8(b'[');
                self.emit_value(out, &val);
                out.write_u8(b':');
                let sub = self.walk_hash(body, env, classes, cache);
                out.write_u64(sub);
                out.write_u8(b']');
                match prev {
                    Some(p) => {
                        env.values.insert(name.clone(), p);
                    }
                    None => {
                        env.values.remove(&name);
                    }
                }
            }
        } else {
            out.write_u8(b'?');
        }
        out.write_u8(b'}');
    }

    fn expand_range(&self, expr: &Value, env: &WalkEnv) -> Vec<AspectValue> {
        let Some(items) = expr.to_ref_vec() else {
            return Vec::new();
        };
        let Some(op) = items.first().and_then(|v| v.as_symbol()) else {
            return Vec::new();
        };
        match op {
            "int-range" if items.len() == 3 => {
                // Mirror compiler.rs / chc.rs: inclusive upper bound. The
                // compositional formulas emitted by the compiler enumerate
                // k ∈ [lo, hi], so the bisim signature must use the same
                // range or the partition won't reflect the formulas the
                // synthesizer actually sees.
                let lo = self.ctx.eval_int_expr(items[1], env).unwrap_or(0).max(0) as usize;
                let hi = self.ctx.eval_int_expr(items[2], env).unwrap_or(0).max(0) as usize;
                if hi < lo {
                    Vec::new()
                } else {
                    (lo..=hi).map(AspectValue::Int).collect()
                }
            }
            "univ" => {
                let Ok((model_name, sort)) = self.ctx.parse_univ_expr(expr, env) else {
                    return Vec::new();
                };
                let Ok(model) = self.ctx.model(&model_name) else {
                    return Vec::new();
                };
                let sort_arc: Arc<str> = Arc::from(sort.as_str());
                model
                    .univ_for_sort(&sort)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort_arc.clone(),
                        name: Arc::from(name),
                    })
                    .collect()
            }
            _ => Vec::new(),
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
