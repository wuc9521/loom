use crate::lang::types::{DataType, TypeEnv};
use crate::lang::runtime::Value;
use crate::lang::model::{Pattern, PatternElem};


#[derive(Debug)]
pub struct PatternMatcher {
    pub type_env: TypeEnv,
}

impl PatternMatcher {
    pub fn new(type_env: TypeEnv) -> Self {
        Self { type_env }
    }

    /// check if value matches pattern
    pub fn matches(&self, value: &Value, pattern: &Pattern) -> Result<bool, String> {
        self.matches_pattern(value, pattern)
    }

    /// core pattern matching logic
    fn matches_pattern(&self, value: &Value, pattern: &Pattern) -> Result<bool, String> {
        if pattern.elements.len() == 1 {
            // single element pattern
            return self.matches_single_pattern(value, &pattern.elements[0]);
        }

        // multiple element pattern (for matching constructors)
        if let Value::Symbol(constructor_name) = value {
            // find data type
            if let Some(data_type) = self.find_constructor_datatype(constructor_name) {
                return self.matches_constructor_pattern(value, pattern, data_type);
            }
        }

        // default not matching
        Ok(false)
    }

    /// match single pattern element
    fn matches_single_pattern(&self, value: &Value, pattern_elem: &PatternElem) -> Result<bool, String> {
        match pattern_elem {
            PatternElem::UnivElem(univ_elem) => {
                // match specific value
                if let Value::Symbol(sym) = value {
                    Ok(sym == &univ_elem.name)
                } else {
                    Ok(false)
                }
            }
            PatternElem::Var(_var_name) => {
                // variable pattern, always matching
                Ok(true)
            }
            PatternElem::Wildcard => {
                // wildcard, always matching
                Ok(true)
            }
        }
    }

    /// match constructor pattern
    fn matches_constructor_pattern(&self, _value: &Value, pattern: &Pattern, data_type: &DataType) -> Result<bool, String> {
        // check if pattern starts with constructor
        if pattern.elements.is_empty() {
            return Ok(false);
        }

        let first_elem = &pattern.elements[0];
        if let PatternElem::UnivElem(univ_elem) = first_elem {
            // check if it is a valid constructor
            let constructor_name = &univ_elem.name;
            let constructor = data_type.constructors.iter()
                .find(|c| &c.name == constructor_name)
                .ok_or_else(|| format!("Unknown constructor: {}", constructor_name))?;

        // check parameter number
            let pattern_args = &pattern.elements[1..];
            if pattern_args.len() != constructor.fields.len() {
                return Ok(false);
            }

            // TODO: implement more complex logic to handle constructor parameter matching
            // TODO: return true for now
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// find data type of constructor
    fn find_constructor_datatype(&self, constructor_name: &str) -> Option<&DataType> {
        for data_type in self.type_env.datatypes.values() {
            for constructor in &data_type.constructors {
                if constructor.name == constructor_name {
                    return Some(data_type);
                }
            }
        }
        None
    }

    /// extract variable bindings from pattern
    pub fn extract_bindings(&self, value: &Value, pattern: &Pattern) -> Result<Vec<(String, Value)>, String> {
        let mut bindings = Vec::new();
        self.extract_bindings_recursive(value, pattern, &mut bindings)?;
        Ok(bindings)
    }

    /// recursively extract variable bindings
    fn extract_bindings_recursive(&self, value: &Value, pattern: &Pattern, bindings: &mut Vec<(String, Value)>) -> Result<(), String> {
        for (_i, elem) in pattern.elements.iter().enumerate() {
            match elem {
                PatternElem::Var(var_name) => {
                    // extract variable binding
                    if let Value::Symbol(sym) = value {
                        bindings.push((var_name.clone(), Value::Symbol(sym.clone())));
                    } else {
                        bindings.push((var_name.clone(), value.clone()));
                    }
                }
                PatternElem::UnivElem(univ_elem) => {
                    // check if value matches
                    if let Value::Symbol(sym) = value {
                        if sym != &univ_elem.name {
                            return Err(format!("Pattern mismatch: expected {}, got {}", univ_elem.name, sym));
                        }
                    } else {
                        return Err("Pattern mismatch: expected symbol".to_string());
                    }
                }
                PatternElem::Wildcard => {
                    // wildcard, do not extract binding
                }
            }
        }
        Ok(())
    }
}

/// enhanced match expression evaluator
#[derive(Debug)]
pub struct MatchEvaluator {
    pub pattern_matcher: PatternMatcher,
}

impl MatchEvaluator {
    pub fn new(type_env: TypeEnv) -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(type_env),
        }
    }

    /// evaluate match expression
    pub fn eval_match(&self, value: &Value, cases: &[crate::lang::model::MatchCase]) -> Result<Value, String> {
        for case in cases {
            if let Some(pattern) = &case.pattern {
                if self.pattern_matcher.matches(value, pattern)? {
                    // extract variable bindings
                    let _bindings = self.pattern_matcher.extract_bindings(value, pattern)?;
                    
                    // TODO: implement more complex logic to handle constructor parameter matching
                // TODO: return case.value for now
                    return Ok(Value::Symbol("matched".to_string()));
                }
            } else {
                // default branch
                return Ok(Value::Symbol("default".to_string()));
            }
        }
        Err("No matching pattern found".to_string())
    }
}

/// data type declaration processor
#[derive(Debug)]
pub struct DataTypeProcessor {
    pub type_env: TypeEnv,
}

impl DataTypeProcessor {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnv::new(),
        }
    }

    /// process declare-datatypes command
    pub fn process_declare_datatypes(&mut self, datatypes: Vec<DataType>) -> Result<(), String> {
        for data_type in datatypes {
            self.type_env.register_datatype(data_type.name.clone(), data_type);
        }
        Ok(())
    }

    /// parse data type declaration from SMT-LIB format
    pub fn parse_datatype_declaration(&self, _input: &str) -> Result<Vec<DataType>, String> {
        // TODO: implement SMT-LIB format parsing
        // TODO: return empty vector for now
        Ok(Vec::new())
    }
} 