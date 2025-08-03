use crate::lang::model::{EnumSortElem, Pattern, PatternElem};
use crate::lang::runtime::{Scope, Value};
use log::debug;

/// Result of pattern matching
#[derive(Debug, Clone)]
pub struct PatternMatchResult {
    pub matched: bool,
    pub context: Scope,
}

impl PatternMatchResult {
    pub fn success(context: Scope) -> Self {
        Self {
            matched: true,
            context,
        }
    }

    pub fn failure() -> Self {
        Self {
            matched: false,
            context: Scope::new(),
        }
    }
}

/// Generic pattern matcher that can be used throughout the project
pub struct PatternMatcher;

impl PatternMatcher {
    pub fn new() -> Self {
        Self {}
    }

    /// Main pattern matching function
    pub fn matches(&self, value: &Value, pattern: &Pattern) -> Result<PatternMatchResult, String> {
        debug!(
            "PatternMatcher::matches: value={:?}, pattern={:?}",
            value, pattern
        );
        let mut context = Scope::new();

        if self.match_pattern(value, pattern, &mut context)? {
            debug!("Pattern matched successfully, context: {:?}", context);
            Ok(PatternMatchResult::success(context))
        } else {
            debug!("Pattern did not match");
            Ok(PatternMatchResult::failure())
        }
    }

    /// Recursive pattern matching implementation
    fn match_pattern(
        &self,
        value: &Value,
        pattern: &Pattern,
        context: &mut Scope,
    ) -> Result<bool, String> {
        // Handle single element patterns
        if pattern.elements.len() == 1 {
            return self.match_single_element(value, &pattern.elements[0], context);
        }

        // Handle constructor patterns (multiple elements)
        if pattern.elements.len() >= 2 {
            return self.match_constructor_pattern(value, pattern, context);
        }

        // Empty pattern should not match anything
        Ok(false)
    }

    /// Match a single pattern element
    fn match_single_element(
        &self,
        value: &Value,
        element: &PatternElem,
        context: &mut Scope,
    ) -> Result<bool, String> {
        match element {
            PatternElem::SortElem(enum_elem) => {
                // Match enum elements
                self.match_enum_element(value, enum_elem)
            }
            PatternElem::Var(var_name) => {
                // For DataValue, check if it's a constructor name match
                if let Value::DataValue { constructor, .. } = value {
                    if constructor == var_name {
                        // this is a constructor name match
                        return Ok(true);
                    } else {
                        // constructor name doesn't match, this pattern fails
                        return Ok(false);
                    }
                }
                // For Symbol, check if it's a constructor name match
                if let Value::Symbol(sym) = value {
                    if sym == var_name {
                        // this is a constructor name match
                        return Ok(true);
                    }
                }
                // For other value types, treat as variable binding
                // (this handles cases where we actually want variable binding)
                context.define(var_name.clone(), value.clone());
                Ok(true)
            }
            PatternElem::Wildcard => {
                // Wildcard - always succeeds
                Ok(true)
            }
        }
    }

    /// Match enum elements
    fn match_enum_element(&self, value: &Value, enum_elem: &EnumSortElem) -> Result<bool, String> {
        match value {
            Value::Symbol(sym) => {
                // Direct symbol comparison
                Ok(sym == &enum_elem.name)
            }
            Value::Int(n) => {
                // try to parse enum element as integer
                if let Ok(elem_num) = enum_elem.name.parse::<i64>() {
                    Ok(*n == elem_num)
                } else {
                    Ok(false)
                }
            }
            Value::String(s) => {
                // string comparison
                Ok(s == &enum_elem.name)
            }
            _ => {
                // for other types, try string comparison
                Ok(value.to_string() == enum_elem.name)
            }
        }
    }

    /// Match constructor patterns (e.g., (cons head tail))
    fn match_constructor_pattern(
        &self,
        value: &Value,
        pattern: &Pattern,
        context: &mut Scope,
    ) -> Result<bool, String> {
        // ensure pattern has enough elements (at least one constructor name)
        if pattern.elements.is_empty() {
            return Ok(false);
        }

        // get constructor name
        let constructor_name = if let PatternElem::Var(name) = &pattern.elements[0] {
            name
        } else {
            return Ok(false);
        };

        // handle Value::DataValue (new data type value)
        if let Value::DataValue {
            constructor,
            datatype: _,
            fields,
        } = value
        {
            if constructor == constructor_name {
                // check if the number of parameters matches
                let expected_params = pattern.elements.len() - 1;
                let actual_params = fields.len();

                if expected_params == actual_params {
                    // match parameters
                    for (i, elem) in pattern.elements.iter().skip(1).enumerate() {
                        if i < actual_params {
                            if !self.match_single_element(&fields[i], elem, context)? {
                                return Ok(false);
                            }
                        }
                    }
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // handle Value::Symbol (simple constructor, like nil, empty, etc.)
        if let Value::Symbol(sym) = value {
            if sym == constructor_name {
                // if pattern has only one element (constructor name), then match successfully
                if pattern.elements.len() == 1 {
                    return Ok(true);
                }
                // otherwise, need parameters but value has no parameters, match failed
                return Ok(false);
            }
            return Ok(false);
        }

        // handle Value::List (constructor with parameters)
        if let Value::List(items) = value {
            // check if the first element is the constructor name
            if let Some(first_item) = items.first() {
                if let Value::Symbol(sym) = first_item {
                    if sym == constructor_name {
                        // check if the number of parameters matches
                        let expected_params = pattern.elements.len() - 1;
                        let actual_params = items.len() - 1;

                        if expected_params == actual_params {
                            // match parameters
                            for (i, elem) in pattern.elements.iter().skip(1).enumerate() {
                                if i < actual_params {
                                    if !self.match_single_element(&items[i + 1], elem, context)? {
                                        return Ok(false);
                                    }
                                }
                            }
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Match against a list of patterns and return the first match
    pub fn match_cases(
        &self,
        value: &Value,
        cases: &[(Pattern, Value)],
    ) -> Result<Option<(Value, Scope)>, String> {
        for (pattern, case_value) in cases {
            let result = self.matches(value, pattern)?;
            if result.matched {
                return Ok(Some((case_value.clone(), result.context)));
            }
        }
        Ok(None)
    }

    /// Extract variables from pattern for type binding
    pub fn extract_variables(&self, pattern: &Pattern) -> Vec<String> {
        let mut vars = Vec::new();
        for element in &pattern.elements {
            if let PatternElem::Var(var_name) = element {
                vars.push(var_name.clone());
            }
        }
        vars
    }

    /// Extract constructor parameters (skip constructor name)
    pub fn extract_constructor_params(&self, pattern: &Pattern) -> Vec<String> {
        let mut params = Vec::new();
        // Skip the first element (constructor name) and extract parameters
        for element in pattern.elements.iter().skip(1) {
            if let PatternElem::Var(var_name) = element {
                params.push(var_name.clone());
            }
        }
        params
    }

    /// Check if pattern contains wildcard
    pub fn has_wildcard(&self, pattern: &Pattern) -> bool {
        pattern
            .elements
            .iter()
            .any(|elem| matches!(elem, PatternElem::Wildcard))
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience trait for pattern matching
pub trait PatternMatchable {
    fn matches_pattern(&self, pattern: &Pattern) -> Result<PatternMatchResult, String>;
}

impl PatternMatchable for Value {
    fn matches_pattern(&self, pattern: &Pattern) -> Result<PatternMatchResult, String> {
        let matcher = PatternMatcher::new();
        matcher.matches(self, pattern)
    }
}

/// Pattern context for type checking during pattern matching
#[derive(Debug, Clone)]
pub struct PatternContext {
    pub type_env: crate::lang::types::TypeEnv,
}

impl PatternContext {
    pub fn new(type_env: crate::lang::types::TypeEnv) -> Self {
        Self { type_env }
    }
}

impl PatternMatcher {
    /// Pattern matching with context (for type checking)
    pub fn matches_with_context(
        &self,
        value: &Value,
        pattern: &Pattern,
        _context: PatternContext,
    ) -> Result<PatternMatchResult, String> {
        // For now, just delegate to the regular matches method
        // TODO: Implement context-aware matching when needed
        self.matches(value, pattern)
    }

    /// Bind pattern variables for type inference
    pub fn bind_pattern_variables_for_type_inference(
        &self,
        pattern: &Pattern,
        type_env: &mut crate::lang::types::TypeEnv,
        term_type: &sygus_parser::Sort,
    ) -> Result<(), String> {
        // Extract variables from pattern and bind them with appropriate types
        let vars = self.extract_variables(pattern);
        for var in vars {
            // For now, bind all pattern variables with the term type
            // TODO: Implement more sophisticated type inference for pattern variables
            type_env.bind_variable(var, term_type.clone());
        }
        Ok(())
    }
}

/// Utility functions for common pattern matching operations
pub mod utils {
    use super::*;

    /// Check if a value matches any pattern in a list
    pub fn matches_any(value: &Value, patterns: &[Pattern]) -> Result<bool, String> {
        let matcher = PatternMatcher::new();
        for pattern in patterns {
            let result = matcher.matches(value, pattern)?;
            if result.matched {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
