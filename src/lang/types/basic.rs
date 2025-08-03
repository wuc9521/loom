use std::collections::HashMap;
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

/// datatype constructor
#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<Field>, // list of fields
}

/// datatype definition
#[derive(Debug, Clone)]
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

/// Class type definition
#[derive(Debug, Clone)]
pub struct ClassType {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, Sort)>,
    pub methods: HashMap<String, super::FunctionType>,
}

#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub name: String,
    pub datatype: String,
    pub field_types: Vec<Sort>,
} 