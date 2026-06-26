use sygus_parser::SExpr as Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AspectKey {
    pub relation: String,
    pub model: String,
    pub fields: Vec<(String, AspectValue)>,
}

impl AspectKey {
    pub fn display_fields(&self) -> String {
        self.fields
            .iter()
            .map(|(name, value)| format!("{name}={}", value.display()))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitMapping {
    pub aspect: AspectKey,
    pub bit: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Encoding {
    pub aspect_spaces: Vec<String>,
    pub semantic_relations: Vec<String>,
    pub semantic_domains: Vec<SemanticDomain>,
    pub bit_mappings: Vec<BitMapping>,
    pub constraint_bits: Vec<ConstraintBit>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintBit {
    pub relation: String,
    pub model: String,
    pub expected: bool,
    pub bit: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticProgram {
    pub aspect_spaces: Vec<AspectSpace>,
    pub semantics: Vec<SemanticSpec>,
    pub domains: Vec<SemanticDomain>,
    pub constraints: Vec<SemanticConstraint>,
}

#[derive(Debug, Clone)]
pub struct SemanticSpec {
    pub relation: String,
    pub term_sort: String,
    /// The user-chosen aspect binder name from the header, e.g. `asp` in
    /// `(asp ExAsp)`. This is an arbitrary local name (the paper's state
    /// binder σ), so it must be threaded through rather than hardcoded.
    pub aspect_var: String,
    pub aspect_space: String,
    pub return_kind: SemanticKind,
    pub body: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticKind {
    Bool,
    FiniteValue { sort: String },
}

#[derive(Debug, Clone)]
pub struct SemanticDomain {
    pub relation: String,
    pub term_sort: String,
    pub aspect_space: String,
    pub kind: SemanticKind,
    pub bits: Vec<DomainBit>,
}

impl SemanticDomain {
    /// Number of distinct BV positions occupied by this domain. Under the
    /// bisim quotient, several `DomainBit` entries may map to the same
    /// `bit` index, so this is `max(bit) + 1` rather than `bits.len()`.
    pub fn width(&self) -> usize {
        self.bits.iter().map(|b| b.bit + 1).max().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct DomainBit {
    pub bit: usize,
    pub aspect: AspectKey,
    pub value: Option<AspectValue>,
}

#[allow(dead_code)]
impl Encoding {
    pub fn to_comment_text(&self, target: &str) -> String {
        let mut lines = Vec::new();
        lines.push(format!("; {target} compilation summary"));
        lines.push(format!(
            "; aspect-spaces: {}",
            self.aspect_spaces.join(", ")
        ));
        lines.push(format!(
            "; semantic-relations: {}",
            self.semantic_relations.join(", ")
        ));
        for domain in &self.semantic_domains {
            lines.push(format!(
                "; semantic-domain {} term-sort={} aspect-space={} kind={:?} bits={}",
                domain.relation,
                domain.term_sort,
                domain.aspect_space,
                domain.kind,
                domain.bits.len()
            ));
        }
        lines.push(format!("; bv-len: {}", self.bit_mappings.len()));
        for mapping in &self.bit_mappings {
            lines.push(format!(
                "; bit {} <-> {}({}; {})",
                mapping.bit,
                mapping.aspect.relation,
                mapping.aspect.model,
                mapping.aspect.display_fields()
            ));
        }
        for constraint in &self.constraint_bits {
            lines.push(format!(
                "; constraint {}{} -> bit {} == {}",
                if constraint.expected { "+" } else { "-" },
                format!("{}:{}", constraint.relation, constraint.model),
                constraint.bit,
                if constraint.expected { 1 } else { 0 }
            ));
        }
        if self.bit_mappings.is_empty() {
            lines.push("; no semantic aspects were enumerated; add semantic constraints to create BV state bits".to_string());
        }
        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub domain: FieldDomain,
}

#[derive(Debug, Clone)]
pub enum FieldDomain {
    Sort(String),
    Enum(Vec<String>),
}

impl FieldDomain {
    pub fn as_sort(&self) -> Option<&str> {
        match self {
            FieldDomain::Sort(sort) => Some(sort),
            FieldDomain::Enum(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AspectSpace {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub where_expr: Option<Value>,
}

/// `Univ` uses `Arc<str>` for its two fields because this is the only
/// variant cloned in the bisim hot loop — every `env.values.get(symbol)
/// .cloned()` plus every quantifier-binding push. With `String`, that's
/// two heap allocations per clone; with `Arc<str>`, it's two atomic
/// refcount bumps. Symbol/Model stay `String` since they're constructed
/// in cold paths and consumed in compile/output code that already expects
/// `String`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AspectValue {
    Model(String),
    Int(usize),
    Univ {
        sort: std::sync::Arc<str>,
        name: std::sync::Arc<str>,
    },
    Symbol(String),
}

impl AspectValue {
    pub fn display(&self) -> String {
        match self {
            AspectValue::Model(value) | AspectValue::Symbol(value) => value.clone(),
            AspectValue::Univ { name, .. } => name.to_string(),
            AspectValue::Int(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticConstraint {
    pub relation: String,
    pub model: String,
    pub fields: Vec<(String, AspectValue)>,
    pub expected: bool,
}
