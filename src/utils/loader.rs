use crate::fol::{
    func::Function, model::Model, pred::Predicate, r#const::Constant, signature::Signature,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use std::error::Error;
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub domain: Vec<i32>,
    pub constants: HashMap<String, i32>,
    pub predicates: HashMap<String, PredicateEntry>,
    pub functions: HashMap<String, FunctionWithMap>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionWithMap {
    pub arity: usize,
    pub map: Vec<FunctionEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub key: Vec<i32>,
    pub value: i32,
}

impl FunctionWithMap {
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get_map(&self) -> &Vec<FunctionEntry> {
        &self.map
    }

    pub fn arity(&self) -> usize {
        self.arity
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredicateEntry {
    pub arity: usize,
    pub value: Vec<Vec<i32>>,
}

pub fn load_signature(filename: &PathBuf) -> Result<Signature, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let signature: Signature = serde_json::from_reader(reader)?;
    Ok(signature)
}

/// load a model fron the config file and validate with its signature.
pub fn load_from_file_with_signature(
    positivity: bool,
    model_path: &PathBuf,
    signature: &Signature,
) -> Result<Model, Box<dyn Error>> {
    let file = File::open(model_path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    signature.validate_config(&config)?; // Validate the config against the signature specification
    let mut model = Model::new(positivity);
    // Get domain size and create normalized domain
    let domain_size = config.domain.len();
    model.set_domain_size(domain_size);

    // Create sorted version of original domain
    let mut sorted_domain = config.domain.clone();
    sorted_domain.sort();
    sorted_domain.dedup();

    // Create mapping from original values to normalized values (0 to n-1)
    let domain_mapping: HashMap<i32, i32> = sorted_domain
        .iter()
        .enumerate()
        .map(|(i, &val)| (val, i as i32))
        .collect();

    // Process constants with normalized values
    for constant_name in &signature.constants {
        if let Some(&value) = config.constants.get(constant_name) {
            let normalized_value = *domain_mapping.get(&value).unwrap_or(&0);
            model
                .constants
                .push(Constant::new(constant_name, normalized_value));
        }
    }

    // Process predicates with normalized values
    for (name, pred_spec) in &signature.predicates {
        if let Some(entry) = config.predicates.get(name) {
            let mut predicate = Predicate::new(name, pred_spec.arity);

            for tuple in &entry.value {
                let normalized_tuple: Vec<i32> = tuple
                    .iter()
                    .map(|&val| *domain_mapping.get(&val).unwrap_or(&0))
                    .collect();
                predicate.extend(normalized_tuple);
            }
            model.predicates.push(predicate);
        }
    }

    // Process functions with normalized values
    for (name, function_spec) in &config.functions {
        if !function_spec.map.is_empty() {
            let mut function = Function::new(name, function_spec.arity);

            for entry in &function_spec.map {
                let normalized_key: Vec<i32> = entry
                    .key
                    .iter()
                    .map(|&val| *domain_mapping.get(&val).unwrap_or(&0))
                    .collect();
                let normalized_value = *domain_mapping.get(&entry.value).unwrap_or(&0);
                function.extend(normalized_key, normalized_value);
            }
            model.functions.push(function);
        }
    }

    Ok(model)
}