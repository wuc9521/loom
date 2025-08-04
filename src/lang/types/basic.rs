use std::collections::HashMap;
use derive_more::Display;
use sygus_parser::Sort;

/// finiteness
#[derive(Debug, Clone, PartialEq)]
pub enum Finiteness {
    Finite(usize), // finite, and give the cardinality
    Infinite,      // infinite
}

/// datatype field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub sort: Sort,
}

/// Unified constructor structure
#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub datatype: String,
    pub fields: Vec<Field>,
}

impl Constructor {
    /// Get field types as a vector of Sort
    pub fn field_types(&self) -> Vec<Sort> {
        self.fields.iter().map(|f| f.sort.clone()).collect()
    }
    
    /// Create a new constructor
    pub fn new(name: String, datatype: String, fields: Vec<Field>) -> Self {
        Self { name, datatype, fields }
    }
}

/// datatype definition
#[derive(Debug, Clone, Display)]
#[display(fmt = "({} ({}))", name, "constructors.iter().map(|c| format!(\"({})\", c.name)).collect::<Vec<String>>().join(\", \")")]
pub struct DataType {
    pub name: String,
    pub constructors: Vec<Constructor>,
}

/// enum type definition
#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub elements: Vec<String>,
}

/// Class type definition (for type checking phase)
#[derive(Debug, Clone)]
pub struct ClassType {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, Sort)>,
    pub methods: HashMap<String, super::FunctionType>,
    pub datatypes: Vec<String>,
} 