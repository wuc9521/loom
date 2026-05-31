use crate::compile::types::{AspectKey, AspectValue, DomainBit, SemanticDomain};
use crate::lang::{eval::eval_symbols, model::Model};
use rustc_hash::FxHashMap as HashMap;
use std::time::Instant;

type Fingerprint = Vec<u8>;

impl AspectKey {
    /// Compute a content fingerprint by evaluating the given functions on the
    /// integer-range scope of this aspect inside `model`. Two aspects with
    /// equal fingerprints are observationally equivalent under the semantic
    /// evaluator and may share a single bit position.
    pub fn fingerprint(&self, model: &Model, func_names: &[String]) -> Fingerprint {
        let mut fp = Vec::new();

        // Encode non-integer fields (modes, enum tags) into the fingerprint.
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

        // Extract the integer scope: a 1-field aspect = single position,
        // a 2-field aspect = [start, end). Anything else: skip the body.
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
                let result = eval_symbols(func, &[pos_name.clone()], model);
                match result {
                    Some(crate::lang::model::InterpedValue::Literal(
                        sygus_parser::Literal::Bool(true),
                    )) => fp.push(b'1'),
                    Some(crate::lang::model::InterpedValue::Literal(
                        sygus_parser::Literal::Bool(false),
                    )) => fp.push(b'0'),
                    Some(crate::lang::model::InterpedValue::UnivElem(e)) => {
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

/// Quotient each domain's bit list by content equivalence. Aspects that
/// observe the same model data through the same scope are merged onto a
/// shared bit position. All original aspects remain in `domain.bits` so
/// downstream lookups by `AspectKey` still succeed; the `bit` field is
/// rewritten to a canonical (and possibly shared) index.
pub fn quotient_domains(
    domains: &mut Vec<SemanticDomain>,
    models: &[&Model],
    func_names: &[String],
) {
    let start = Instant::now();
    let models_by_name: HashMap<&str, &Model> =
        models.iter().map(|m| (m.name.as_str(), *m)).collect();

    let mut total_before = 0usize;
    let mut total_after = 0usize;

    for domain in domains.iter_mut() {
        let before = domain.bits.len();
        total_before += before;
        if before == 0 {
            continue;
        }

        let mut class_map: HashMap<Fingerprint, usize> = HashMap::default();
        let mut next_class = 0usize;
        let mut new_bits: Vec<DomainBit> = Vec::with_capacity(before);

        for bit in &domain.bits {
            let canonical = match models_by_name.get(bit.aspect.model.as_str()) {
                Some(model) => {
                    let mut fp = bit.aspect.fingerprint(model, func_names);
                    if let Some(ref val) = bit.value {
                        fp.push(b'V');
                        match val {
                            AspectValue::Univ { sort, name } => {
                                fp.extend_from_slice(sort.as_bytes());
                                fp.push(b':');
                                fp.extend_from_slice(name.as_bytes());
                            }
                            AspectValue::Symbol(s) => fp.extend_from_slice(s.as_bytes()),
                            _ => {}
                        }
                    }
                    *class_map.entry(fp).or_insert_with(|| {
                        let idx = next_class;
                        next_class += 1;
                        idx
                    })
                }
                None => {
                    let idx = next_class;
                    next_class += 1;
                    idx
                }
            };

            new_bits.push(DomainBit {
                bit: canonical,
                aspect: bit.aspect.clone(),
                value: bit.value.clone(),
            });
        }

        total_after += next_class;
        domain.bits = new_bits;
    }

    log::info!(
        "bisim quotient: {} -> {} bits ({:.1}% reduction) in {:?}",
        total_before,
        total_after,
        if total_before > 0 {
            (1.0 - total_after as f64 / total_before as f64) * 100.0
        } else {
            0.0
        },
        start.elapsed()
    );
}
