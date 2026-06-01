use crate::compile::eval_ctx::{EnvView, SemanticContext};
use crate::compile::semantic::{parse_cases, Case};
use crate::compile::types::{
    AspectKey, AspectValue, DomainBit, SemanticKind, SemanticProgram,
};
use crate::lang::model::{InterpedValue, Model};
use crate::parser::DelisProg;
use lexpr::Value;
use rustc_hash::FxHashMap as HashMap;
use std::time::Instant;
use sygus_parser::Literal;

type Fingerprint = Vec<u8>;
type ClassId = usize;
type ClassKey = (String, AspectKey, Option<AspectValue>);

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
            if let Ok(cases) = parse_cases(&spec.body) {
                cases_by_relation.insert(spec.relation.clone(), cases);
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
        let refiner = Refiner {
            ctx: &ctx,
            cases_by_relation: &cases_by_relation,
            domain_kinds: &domain_kinds,
        };

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
        let mut next_class_id = classes
            .values()
            .max()
            .copied()
            .map(|m| m + 1)
            .unwrap_or(0);

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
                let mut by_sig: HashMap<Vec<u8>, Vec<ClassKey>> = HashMap::default();
                for member in &members {
                    let sig =
                        refiner.signature(&member.0, &member.1, &member.2, &classes);
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

        log::info!(
            "bisim quotient: {} -> {} bits ({:.1}% reduction) in {:?} ({} iters)",
            total_before,
            total_after,
            if total_before > 0 {
                (1.0 - total_after as f64 / total_before as f64) * 100.0
            } else {
                0.0
            },
            start.elapsed(),
            iters,
        );
    }
}

/// Walks a semantic case body symbolically, emitting bytes that distinguish
/// aspects when their case-body shape differs under the current partition.
/// Concrete sub-expressions are evaluated via `SemanticContext`; relation
/// calls and quantifiers are handled here because they need partition lookup
/// and range expansion respectively.
struct Refiner<'a> {
    ctx: &'a SemanticContext<'a>,
    cases_by_relation: &'a HashMap<String, HashMap<String, Case>>,
    domain_kinds: &'a HashMap<String, SemanticKind>,
}

#[derive(Clone)]
struct WalkEnv {
    aspects: HashMap<String, AspectKey>, // "asp" → current aspect
    values: HashMap<String, AspectValue>, // quantifier bindings
    subterms: HashMap<String, usize>,    // constructor pattern vars
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
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(relation.as_bytes());
        out.push(b'#');
        if let Some(v) = value {
            out.extend_from_slice(v.display().as_bytes());
        }
        out.push(b'@');
        let Some(cases) = self.cases_by_relation.get(relation) else {
            return out;
        };
        let mut names: Vec<&str> = cases.keys().map(|s| s.as_str()).collect();
        names.sort();
        for cons in names {
            let case = &cases[cons];
            out.extend_from_slice(cons.as_bytes());
            out.push(b'{');
            let mut env = WalkEnv {
                aspects: HashMap::default(),
                values: HashMap::default(),
                subterms: HashMap::default(),
            };
            env.aspects.insert("asp".to_string(), aspect.clone());
            env.subterms.insert(case.self_var.clone(), SELF_SUBTERM);
            for (i, v) in case.vars.iter().enumerate() {
                env.subterms.insert(v.clone(), i);
            }
            self.walk(&mut out, &case.body, &env, classes);
            out.push(b'}');
        }
        out
    }

    fn walk(
        &self,
        out: &mut Vec<u8>,
        expr: &Value,
        env: &WalkEnv,
        classes: &HashMap<ClassKey, ClassId>,
    ) {
        // 1. Relation calls and quantifiers need bespoke handling.
        if let Some(items) = expr.to_ref_vec() {
            if let Some(op) = items.first().and_then(|v| v.as_symbol()) {
                if self.ctx.has_relation(op) && items.len() >= 3 {
                    self.emit_relation(out, op, items[1], items[2], env, classes);
                    return;
                }
                if matches!(op, "exists" | "forall") && items.len() == 3 {
                    self.emit_quantifier(out, op, items[1], items[2], env, classes);
                    return;
                }
            }
        }

        // 2. Try fully-concrete evaluation via the shared evaluator.
        if let Ok(v) = self.ctx.eval_value(expr, env) {
            out.push(b'V');
            self.emit_value(out, &v);
            return;
        }
        if let Ok(b) = self.ctx.eval_bool_value(expr, env) {
            out.push(if b { b'T' } else { b'F' });
            return;
        }

        // 3. Structural fallback for connectives whose subtrees include
        //    relation calls.
        let Some(items) = expr.to_ref_vec() else {
            out.push(b'?');
            out.extend_from_slice(format!("{expr}").as_bytes());
            return;
        };
        let Some(op) = items.first().and_then(|v| v.as_symbol()) else {
            out.push(b'?');
            return;
        };
        match op {
            "and" | "or" => {
                out.push(if op == "and" { b'A' } else { b'O' });
                out.push(b'(');
                for arg in &items[1..] {
                    self.walk(out, arg, env, classes);
                    out.push(b',');
                }
                out.push(b')');
            }
            "not" if items.len() == 2 => {
                out.push(b'N');
                out.push(b'(');
                self.walk(out, items[1], env, classes);
                out.push(b')');
            }
            "ite" if items.len() == 4 => {
                out.push(b'I');
                out.push(b'(');
                self.walk(out, items[1], env, classes);
                out.push(b',');
                self.walk(out, items[2], env, classes);
                out.push(b',');
                self.walk(out, items[3], env, classes);
                out.push(b')');
            }
            "=" | "<" | "<=" if items.len() == 3 => {
                out.push(b'=');
                out.extend_from_slice(op.as_bytes());
                out.push(b'(');
                self.walk(out, items[1], env, classes);
                out.push(b',');
                self.walk(out, items[2], env, classes);
                out.push(b')');
            }
            _ => {
                out.push(b'(');
                out.extend_from_slice(op.as_bytes());
                for arg in &items[1..] {
                    out.push(b' ');
                    self.walk(out, arg, env, classes);
                }
                out.push(b')');
            }
        }
    }

    fn emit_relation(
        &self,
        out: &mut Vec<u8>,
        relation: &str,
        subterm_expr: &Value,
        aspect_expr: &Value,
        env: &WalkEnv,
        classes: &HashMap<ClassKey, ClassId>,
    ) {
        out.push(b'R');
        out.extend_from_slice(relation.as_bytes());
        out.push(b'@');
        if let Some(subterm) = subterm_expr.as_symbol() {
            match env.subterms.get(subterm).copied() {
                Some(SELF_SUBTERM) => out.push(b'r'),
                Some(i) => out.extend_from_slice(i.to_string().as_bytes()),
                None => out.push(b'?'),
            }
        } else {
            out.push(b'?');
        }
        out.push(b':');
        match self.ctx.eval_aspect(aspect_expr, relation, env) {
            Ok(asp) => match self.domain_kinds.get(relation) {
                Some(SemanticKind::Bool) => {
                    let class = classes
                        .get(&(relation.to_string(), asp, None))
                        .copied()
                        .unwrap_or(usize::MAX);
                    out.extend_from_slice(class.to_string().as_bytes());
                }
                _ => {
                    // FiniteValue relations: the call returns a value, not a
                    // boolean, so the surrounding context picks the (asp,
                    // value) tuple. Emit the aspect bytes directly rather
                    // than a class id — never aliases distinct aspects (sound
                    // but coarser than the Bool case).
                    out.push(b'*');
                    out.extend_from_slice(asp.model.as_bytes());
                    out.push(b'|');
                    for (k, v) in &asp.fields {
                        out.extend_from_slice(k.as_bytes());
                        out.push(b'=');
                        out.extend_from_slice(v.display().as_bytes());
                        out.push(b';');
                    }
                }
            },
            Err(_) => out.push(b'?'),
        }
    }

    fn emit_quantifier(
        &self,
        out: &mut Vec<u8>,
        quant: &str,
        binding: &Value,
        body: &Value,
        env: &WalkEnv,
        classes: &HashMap<ClassKey, ClassId>,
    ) {
        out.push(if quant == "exists" { b'E' } else { b'F' });
        out.push(b'{');
        let parsed = binding
            .to_ref_vec()
            .and_then(|bs| bs.first().and_then(|b| b.to_ref_vec()))
            .filter(|parts| parts.len() == 2)
            .and_then(|parts| parts[0].as_symbol().map(|n| (n.to_string(), parts[1])));
        if let Some((name, range_expr)) = parsed {
            for val in self.expand_range(range_expr, env) {
                let mut sub = env.clone();
                sub.values.insert(name.clone(), val.clone());
                out.push(b'[');
                self.emit_value(out, &val);
                out.push(b':');
                self.walk(out, body, &sub, classes);
                out.push(b']');
            }
        } else {
            out.push(b'?');
        }
        out.push(b'}');
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
                let lo = self
                    .ctx
                    .eval_int_expr(items[1], env)
                    .unwrap_or(0)
                    .max(0) as usize;
                let hi = self
                    .ctx
                    .eval_int_expr(items[2], env)
                    .unwrap_or(0)
                    .max(0) as usize;
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
                model
                    .univ_for_sort(&sort)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort.clone(),
                        name,
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn emit_value(&self, out: &mut Vec<u8>, v: &AspectValue) {
        match v {
            AspectValue::Int(i) => {
                out.push(b'#');
                out.extend_from_slice(i.to_string().as_bytes());
            }
            AspectValue::Model(m) => {
                out.push(b'M');
                out.extend_from_slice(m.as_bytes());
            }
            AspectValue::Symbol(s) => {
                out.push(b'S');
                out.extend_from_slice(s.as_bytes());
            }
            AspectValue::Univ { sort, name } => {
                out.push(b'U');
                out.extend_from_slice(sort.as_bytes());
                out.push(b':');
                out.extend_from_slice(name.as_bytes());
            }
        }
    }
}
