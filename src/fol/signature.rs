use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::utils::loader::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signature {
    pub constants: Vec<String>,
    pub predicates: HashMap<String, PredicateSpec>,
    pub functions: HashMap<String, FunctionSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredicateSpec {
    pub arity: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionSpec {
    pub arity: usize,
}

// Custom error type for validation
#[derive(Debug)]
pub enum ValidationError {
    MissingConstant(String),
    MissingPredicate(String),
    MissingFunction(String),
    ArityMismatch { name: String, expected: usize, found: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::MissingConstant(name) => write!(f, "Constant '{}' defined in signature but missing in model", name),
            ValidationError::MissingPredicate(name) => write!(f, "Predicate '{}' defined in signature but missing in model", name),
            ValidationError::MissingFunction(name) => write!(f, "Function '{}' defined in signature but missing in model", name),
            ValidationError::ArityMismatch { name, expected, found } => 
                write!(f, "Arity mismatch for '{}': expected {}, found {}", name, expected, found),
        }
    }
}

impl Error for ValidationError {}

impl Signature {
    /// Validate a configuration against the signature
    pub fn validate_config(&self, config: &Config) -> Result<(), ValidationError> {
        // Validate constants
        for constant_name in &self.constants {
            if !config.constants.contains_key(constant_name) {
                return Err(ValidationError::MissingConstant(constant_name.clone()));
            }
        }

        // Validate predicates
        for (pred_name, pred_spec) in &self.predicates {
            match config.predicates.get(pred_name) {
                None => return Err(ValidationError::MissingPredicate(pred_name.clone())),
                Some(pred_entry) => {
                    if pred_entry.arity != pred_spec.arity {
                        return Err(ValidationError::ArityMismatch {
                            name: pred_name.clone(),
                            expected: pred_spec.arity,
                            found: pred_entry.arity,
                        });
                    }
                }
            }
        }

        // Validate functions
for (func_name, func_spec) in &self.functions {
    if let Some(func_entries) = config.functions.get(func_name) {
        if !func_entries.map.is_empty() {
            let found_arity = func_entries.map[0].key.len();
            if found_arity != func_spec.arity {
                return Err(ValidationError::ArityMismatch {
                    name: func_name.clone(),
                    expected: func_spec.arity,
                    found: found_arity,
                });
            }
        }
        // check the arity by the way of the function
        if func_entries.arity != func_spec.arity {
            return Err(ValidationError::ArityMismatch {
                name: func_name.clone(),
                expected: func_spec.arity,
                found: func_entries.arity,
            });
        }
    } else {
        return Err(ValidationError::MissingFunction(func_name.clone()));
    }
}
        Ok(())
    }
}