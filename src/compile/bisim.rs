use crate::compile::semantic::{parse_cases, Case};
use crate::compile::types::{
    AspectKey, AspectValue, DomainBit, FieldDomain, FiniteSpace, SemanticKind, SemanticProgram,
    SemanticSpec,
};
use crate::lang::model::{InterpedValue, Model};
use lexpr::Value;
use rustc_hash::FxHashMap as HashMap;
use std::time::Instant;
use sygus_parser::Literal;

type Fingerprint = Vec<u8>;
type ClassId = usize;
type ClassKey = (String, AspectKey, Option<AspectValue>);

impl AspectKey {
    /// Per-position leaf-function readout. Used only to seed the initial
    /// partition before refinement; the refinement loop is what guarantees
    /// soundness.
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
                let result = model.eval_symbols(func, &[pos_name.clone()]);
                match result {
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
    /// Compute the largest sound bisimulation over the aspect bits via
    /// partition refinement, then collapse equivalent bits onto shared
    /// positions. Recursive relation references are replaced with class IDs
    /// at each refinement step; aspects that produce the same symbolic body
    /// signature (under the current partition) remain merged.
    pub fn quotient_by_bisim(&mut self, models: &[&Model], func_names: &[String]) {
        let start = Instant::now();
        let mut total_before = 0usize;
        for d in &self.domains {
            total_before += d.bits.len();
        }
        if total_before == 0 {
            return;
        }

        let models_by_name: HashMap<&str, &Model> =
            models.iter().map(|m| (m.name.as_str(), *m)).collect();

        let mut cases_by_relation: HashMap<String, HashMap<String, Case>> = HashMap::default();
        for spec in &self.semantics {
            if let Ok(cases) = parse_cases(&spec.body) {
                cases_by_relation.insert(spec.relation.clone(), cases);
            }
        }

        let spec_by_relation: HashMap<String, SemanticSpec> = self
            .semantics
            .iter()
            .map(|s| (s.relation.clone(), s.clone()))
            .collect();

        let spaces_by_name: HashMap<String, FiniteSpace> = self
            .finite_spaces
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();

        // Initial partition: leaf fingerprint per (relation, value, kind).
        let mut classes: HashMap<ClassKey, ClassId> = HashMap::default();
        let mut leaf_to_class: HashMap<(String, Fingerprint, Option<AspectValue>), ClassId> =
            HashMap::default();
        for domain in &self.domains {
            for bit in &domain.bits {
                let fp = match models_by_name.get(bit.aspect.model.as_str()) {
                    Some(m) => bit.aspect.fingerprint(m, func_names),
                    None => Vec::new(),
                };
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

        let refiner = Refiner {
            models_by_name: &models_by_name,
            cases_by_relation: &cases_by_relation,
            spaces_by_name: &spaces_by_name,
            spec_by_relation: &spec_by_relation,
            domain_kinds: self
                .domains
                .iter()
                .map(|d| (d.relation.clone(), d.kind.clone()))
                .collect(),
        };

        let mut iters = 0usize;
        const MAX_ITERS: usize = 64;
        loop {
            iters += 1;
            let mut sig_to_class: HashMap<Vec<u8>, ClassId> = HashMap::default();
            let mut new_classes: HashMap<ClassKey, ClassId> = HashMap::default();
            for domain in &self.domains {
                for bit in &domain.bits {
                    let sig = refiner.signature(
                        &domain.relation,
                        &bit.aspect,
                        &bit.value,
                        &classes,
                    );
                    let next_id = sig_to_class.len();
                    let class = *sig_to_class.entry(sig).or_insert(next_id);
                    new_classes.insert(
                        (
                            domain.relation.clone(),
                            bit.aspect.clone(),
                            bit.value.clone(),
                        ),
                        class,
                    );
                }
            }
            if new_classes == classes {
                break;
            }
            classes = new_classes;
            if iters >= MAX_ITERS {
                log::warn!(
                    "bisim: refinement did not converge in {} iterations; using current partition",
                    MAX_ITERS
                );
                break;
            }
        }

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

// Refiner: symbolic walker that computes a per-aspect signature given the
// current class partition. Two aspects with identical signatures across all
// constructor cases are equivalent under the next refinement step.
struct Refiner<'a> {
    models_by_name: &'a HashMap<&'a str, &'a Model>,
    cases_by_relation: &'a HashMap<String, HashMap<String, Case>>,
    spaces_by_name: &'a HashMap<String, FiniteSpace>,
    spec_by_relation: &'a HashMap<String, SemanticSpec>,
    domain_kinds: HashMap<String, SemanticKind>,
}

#[derive(Clone)]
struct WalkEnv {
    aspect: AspectKey,
    // Constructor pattern vars (subterm placeholders) → positional index.
    // The match scrutinee (`r` in `(r RegLan)`) is bound to usize::MAX.
    subterms: HashMap<String, usize>,
    // Quantifier-bound variables holding concrete AspectValues.
    bound: HashMap<String, AspectValue>,
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
                aspect: aspect.clone(),
                subterms: HashMap::default(),
                bound: HashMap::default(),
            };
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
        // Concrete-value shortcut.
        if let Some(value) = self.try_eval_value(expr, env) {
            self.emit_value(out, &value);
            return;
        }
        if let Some(b) = self.try_eval_bool(expr, env) {
            out.push(if b { b'T' } else { b'F' });
            return;
        }

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
            "exists" | "forall" if items.len() == 3 => {
                out.push(if op == "exists" { b'E' } else { b'F' });
                out.push(b'{');
                let mut binding_emitted = false;
                if let Some(bindings) = items[1].to_ref_vec() {
                    if let Some(first) = bindings.first() {
                        if let Some(parts) = first.to_ref_vec() {
                            if parts.len() == 2 {
                                if let Some(name) = parts[0].as_symbol() {
                                    let range = self.expand_range(parts[1], env);
                                    for val in range {
                                        let mut sub = env.clone();
                                        sub.bound.insert(name.to_string(), val.clone());
                                        out.push(b'[');
                                        self.emit_value(out, &val);
                                        out.push(b':');
                                        self.walk(out, items[2], &sub, classes);
                                        out.push(b']');
                                    }
                                    binding_emitted = true;
                                }
                            }
                        }
                    }
                }
                if !binding_emitted {
                    out.push(b'?');
                }
                out.push(b'}');
            }
            _ if self.spec_by_relation.contains_key(op) => {
                // (R subterm_var aspect_expr)
                if items.len() >= 3 {
                    out.push(b'R');
                    out.extend_from_slice(op.as_bytes());
                    out.push(b'@');
                    if let Some(subterm) = items[1].as_symbol() {
                        let idx = env.subterms.get(subterm).copied();
                        match idx {
                            Some(SELF_SUBTERM) => out.push(b'r'),
                            Some(i) => out.extend_from_slice(i.to_string().as_bytes()),
                            None => out.push(b'?'),
                        }
                    } else {
                        out.push(b'?');
                    }
                    out.push(b':');
                    match self.eval_aspect_expr(items[2], op, env) {
                        Ok(asp) => {
                            // For Bool relations we use value=None. For
                            // FiniteValue relations the call returns a value;
                            // since the surrounding context determines which
                            // (R, asp, value) tuple is being read, we can't
                            // produce a single class id here — emit '*' to
                            // collapse all values for that aspect into one
                            // marker. This is sound (it loses information but
                            // never aliases distinct aspects).
                            let kind = self.domain_kinds.get(op);
                            match kind {
                                Some(SemanticKind::Bool) => {
                                    let class = classes
                                        .get(&(op.to_string(), asp, None))
                                        .copied()
                                        .unwrap_or(usize::MAX);
                                    out.extend_from_slice(class.to_string().as_bytes());
                                }
                                _ => {
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
                            }
                        }
                        Err(_) => out.push(b'?'),
                    }
                } else {
                    out.push(b'?');
                }
            }
            _ => {
                // Unrecognized: emit raw structure with recursive walks.
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

    fn try_eval_bool(&self, expr: &Value, env: &WalkEnv) -> Option<bool> {
        if let Some(b) = expr.as_bool() {
            return Some(b);
        }
        if let Some(sym) = expr.as_symbol() {
            return match sym {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
        let items = expr.to_ref_vec()?;
        let op = items.first().and_then(|v| v.as_symbol())?;
        match op {
            "and" => items[1..]
                .iter()
                .try_fold(true, |acc, e| self.try_eval_bool(e, env).map(|b| acc && b)),
            "or" => items[1..]
                .iter()
                .try_fold(false, |acc, e| self.try_eval_bool(e, env).map(|b| acc || b)),
            "not" if items.len() == 2 => self.try_eval_bool(items[1], env).map(|b| !b),
            "=" if items.len() == 3 => {
                let lhs = self.try_eval_value(items[1], env)?;
                let rhs = self.try_eval_value(items[2], env)?;
                Some(values_equal(&lhs, &rhs, self.current_model(env)?))
            }
            "<" if items.len() == 3 => {
                let lhs = self.try_eval_int(items[1], env)?;
                let rhs = self.try_eval_int(items[2], env)?;
                Some(lhs < rhs)
            }
            "<=" if items.len() == 3 => {
                let lhs = self.try_eval_int(items[1], env)?;
                let rhs = self.try_eval_int(items[2], env)?;
                Some(lhs <= rhs)
            }
            "ite" if items.len() == 4 => {
                let c = self.try_eval_bool(items[1], env)?;
                if c {
                    self.try_eval_bool(items[2], env)
                } else {
                    self.try_eval_bool(items[3], env)
                }
            }
            _ => {
                // Leaf func call: (is.X arg ...) — try to eval concretely.
                if !self.spec_by_relation.contains_key(op) {
                    self.try_eval_func_bool(op, &items[1..], env)
                } else {
                    None
                }
            }
        }
    }

    fn try_eval_value(&self, expr: &Value, env: &WalkEnv) -> Option<AspectValue> {
        if let Some(n) = expr.as_i64() {
            if n < 0 {
                return None;
            }
            return Some(AspectValue::Int(n as usize));
        }
        if let Some(sym) = expr.as_symbol() {
            if let Some(v) = env.bound.get(sym) {
                return Some(v.clone());
            }
            // Common atomic forms used by FiniteValue bodies — bare
            // constructor names (`s`, `t`, ...) are treated as Symbols.
            // Numeric literal "true"/"false" handled by Bool path.
            return None;
        }
        let items = expr.to_ref_vec()?;
        let op = items.first().and_then(|v| v.as_symbol())?;
        // Aspect field accessor: (field-name asp-var)
        if items.len() == 2 {
            if let Some(asp_var) = items[1].as_symbol() {
                if asp_var == "asp" {
                    if op == "model" {
                        return Some(AspectValue::Model(env.aspect.model.clone()));
                    }
                    if let Some((_, v)) = env.aspect.fields.iter().find(|(n, _)| n == op) {
                        return Some(v.clone());
                    }
                }
            }
        }
        match op {
            "+" => items[1..]
                .iter()
                .try_fold(0isize, |acc, e| self.try_eval_int(e, env).map(|i| acc + i))
                .filter(|i| *i >= 0)
                .map(|i| AspectValue::Int(i as usize)),
            "-" if items.len() == 2 => self
                .try_eval_int(items[1], env)
                .map(|i| -i)
                .filter(|i| *i >= 0)
                .map(|i| AspectValue::Int(i as usize)),
            "-" if items.len() >= 3 => {
                let mut it = items[1..].iter();
                let first = self.try_eval_int(it.next()?, env)?;
                let result = it.try_fold(first, |acc, e| self.try_eval_int(e, env).map(|i| acc - i))?;
                if result >= 0 {
                    Some(AspectValue::Int(result as usize))
                } else {
                    None
                }
            }
            "size" if items.len() == 2 => {
                let domain = items[1].to_ref_vec()?;
                let [sort_expr, model_expr] = domain.as_slice() else {
                    return None;
                };
                let sort = sort_expr.as_symbol()?;
                let model_name = self.eval_model_name(model_expr, env)?;
                let model = self.models_by_name.get(model_name.as_str())?;
                let values = model.domain_for_sort(sort)?;
                Some(AspectValue::Int(values.len()))
            }
            "ite" if items.len() == 4 => {
                let c = self.try_eval_bool(items[1], env)?;
                if c {
                    self.try_eval_value(items[2], env)
                } else {
                    self.try_eval_value(items[3], env)
                }
            }
            _ => self.try_eval_func_value(op, &items[1..], env),
        }
    }

    fn try_eval_int(&self, expr: &Value, env: &WalkEnv) -> Option<isize> {
        if let Some(n) = expr.as_i64() {
            return Some(n as isize);
        }
        match self.try_eval_value(expr, env)? {
            AspectValue::Int(i) => Some(i as isize),
            _ => None,
        }
    }

    fn try_eval_func_bool(&self, func: &str, args: &[&Value], env: &WalkEnv) -> Option<bool> {
        let arg_names = args
            .iter()
            .map(|a| self.try_eval_value(a, env).map(|v| v.display()))
            .collect::<Option<Vec<_>>>()?;
        let model = self.current_model(env)?;
        match model.eval_symbols(func, &arg_names)? {
            InterpedValue::Literal(Literal::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    fn try_eval_func_value(
        &self,
        func: &str,
        args: &[&Value],
        env: &WalkEnv,
    ) -> Option<AspectValue> {
        let arg_names = args
            .iter()
            .map(|a| self.try_eval_value(a, env).map(|v| v.display()))
            .collect::<Option<Vec<_>>>()?;
        let model = self.current_model(env)?;
        match model.eval_symbols(func, &arg_names)? {
            InterpedValue::Literal(Literal::Bool(true)) => Some(AspectValue::Symbol("true".into())),
            InterpedValue::Literal(Literal::Bool(false)) => Some(AspectValue::Symbol("false".into())),
            InterpedValue::Literal(lit) => Some(AspectValue::Symbol(lit.to_string())),
            InterpedValue::UnivElem(e) => Some(AspectValue::Symbol(e.name.clone())),
            _ => None,
        }
    }

    fn eval_aspect_expr(
        &self,
        expr: &Value,
        relation: &str,
        env: &WalkEnv,
    ) -> Result<AspectKey, ()> {
        if let Some(sym) = expr.as_symbol() {
            if sym == "asp" {
                let mut k = env.aspect.clone();
                k.relation = relation.to_string();
                return Ok(k);
            }
        }
        let items = expr.to_ref_vec().ok_or(())?;
        let cons = items.first().and_then(|v| v.as_symbol()).ok_or(())?;
        let space = self.spaces_by_name.get(cons).ok_or(())?;
        if items.len() - 1 != space.fields.len() {
            return Err(());
        }
        let mut model_name = String::new();
        let mut fields = Vec::new();
        for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
            match &field.domain {
                FieldDomain::Sort(s) if s == "Model" => {
                    model_name = self.eval_model_name(arg, env).ok_or(())?;
                }
                _ => {
                    let v = self.try_eval_value(arg, env).ok_or(())?;
                    fields.push((field.name.clone(), v));
                }
            }
        }
        Ok(AspectKey {
            relation: relation.to_string(),
            model: model_name,
            fields,
        })
    }

    fn eval_model_name(&self, expr: &Value, env: &WalkEnv) -> Option<String> {
        if let Some(sym) = expr.as_symbol() {
            if sym == "asp" {
                return Some(env.aspect.model.clone());
            }
            return Some(sym.to_string());
        }
        if let Some(items) = expr.to_ref_vec() {
            if items.len() == 2 && items[0].as_symbol() == Some("model") {
                if items[1].as_symbol() == Some("asp") {
                    return Some(env.aspect.model.clone());
                }
            }
        }
        match self.try_eval_value(expr, env)? {
            AspectValue::Model(m) => Some(m),
            _ => None,
        }
    }

    fn current_model(&self, env: &WalkEnv) -> Option<&Model> {
        self.models_by_name
            .get(env.aspect.model.as_str())
            .copied()
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
                let lo = self.try_eval_int(items[1], env).unwrap_or(0).max(0) as usize;
                let hi = self.try_eval_int(items[2], env).unwrap_or(0).max(0) as usize;
                (lo..hi).map(AspectValue::Int).collect()
            }
            "domain" if items.len() == 2 => {
                let inner = items[1].to_ref_vec().unwrap_or_default();
                if inner.len() != 2 {
                    return Vec::new();
                }
                let Some(sort) = inner[0].as_symbol() else {
                    return Vec::new();
                };
                let Some(model_name) = self.eval_model_name(inner[1], env) else {
                    return Vec::new();
                };
                let Some(model) = self.models_by_name.get(model_name.as_str()) else {
                    return Vec::new();
                };
                model
                    .domain_for_sort(sort)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort.to_string(),
                        name,
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

fn values_equal(lhs: &AspectValue, rhs: &AspectValue, model: &Model) -> bool {
    if lhs == rhs {
        return true;
    }
    // Allow Int <-> Univ matching via the model's universe ordering.
    match (lhs, rhs) {
        (AspectValue::Int(idx), AspectValue::Univ { sort, name })
        | (AspectValue::Univ { sort, name }, AspectValue::Int(idx)) => model
            .domain_for_sort(sort)
            .and_then(|d| d.get(*idx).map(|s| s == name))
            .unwrap_or(false),
        _ => false,
    }
}
