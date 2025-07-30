use crate::lang::model::{EnumSortElem, Pattern, PatternElem};
use crate::lang::runtime::{Scope, Value};
use crate::lang::types::TypeEnv;
use log::debug;

/// Context for pattern matching that contains both type and runtime information
#[derive(Debug, Clone)]
pub struct PatternContext {
    pub type_env: TypeEnv,
    pub runtime_scope: Scope,
}

impl PatternContext {
    pub fn new(type_env: TypeEnv) -> Self {
        Self {
            type_env,
            runtime_scope: Scope::new(),
        }
    }

    pub fn with_runtime_scope(mut self, runtime_scope: Scope) -> Self {
        self.runtime_scope = runtime_scope;
        self
    }

    /// Check if a symbol is a constructor in the current type environment
    pub fn is_constructor(&self, name: &str) -> bool {
        self.type_env.is_constructor(name)
    }

    /// Get constructor info if the symbol is a constructor
    pub fn get_constructor_info(&self, name: &str) -> Option<&crate::lang::types::ConstructorInfo> {
        self.type_env.get_constructor(name)
    }

    /// Check if a symbol is an enum element
    pub fn is_enum_element(&self, name: &str) -> Option<&String> {
        for (enum_name, enum_type) in &self.type_env.enums {
            if enum_type.elements.contains(&name.to_string()) {
                return Some(enum_name);
            }
        }
        None
    }

    /// Bind a variable in the runtime scope
    pub fn bind_variable(&mut self, name: String, value: Value) {
        self.runtime_scope.define(name, value);
    }

    /// Get the runtime scope
    pub fn get_runtime_scope(&self) -> &Scope {
        &self.runtime_scope
    }

    /// Get mutable runtime scope
    pub fn get_runtime_scope_mut(&mut self) -> &mut Scope {
        &mut self.runtime_scope
    }
}

/// Result of pattern matching
#[derive(Debug, Clone)]
pub struct PatternMatchResult {
    pub matched: bool,
    pub context: PatternContext,
}

impl PatternMatchResult {
    pub fn success(context: PatternContext) -> Self {
        Self {
            matched: true,
            context,
        }
    }

    pub fn failure(context: PatternContext) -> Self {
        Self {
            matched: false,
            context,
        }
    }
}

/// Core pattern matcher that handles both basic and advanced pattern matching
pub struct PatternMatcher;

impl PatternMatcher {
    pub fn new() -> Self {
        Self {}
    }

    /// Main pattern matching function with type context
    pub fn matches_with_context(
        &self,
        value: &Value,
        pattern: &Pattern,
        context: PatternContext,
    ) -> Result<PatternMatchResult, String> {
        let mut context = context;

        if self.match_pattern(value, pattern, &mut context)? {
            Ok(PatternMatchResult::success(context))
        } else {
            Ok(PatternMatchResult::failure(context))
        }
    }

    /// Pattern matching with type validation
    /// This function combines type checking and runtime matching
    pub fn match_with_type_validation(
        &self,
        value: &Value,
        pattern: &Pattern,
        type_inferer: &crate::lang::types::TypeInferer,
        term_type: &sygus_parser::Sort,
    ) -> Result<PatternMatchResult, String> {
        // Create pattern context with current type environment
        let type_env = type_inferer.global_scope().clone();
        let context = PatternContext::new(type_env);

        // Validate pattern against type system
        self.validate_pattern_against_type(pattern, type_inferer, term_type)?;

        // Perform runtime pattern matching
        self.matches_with_context(value, pattern, context)
    }

    /// Legacy method for backward compatibility
    pub fn matches(&self, value: &Value, pattern: &Pattern) -> Result<PatternMatchResult, String> {
        let context = PatternContext::new(TypeEnv::new());
        self.matches_with_context(value, pattern, context)
    }

    /// Validate pattern against type system
    fn validate_pattern_against_type(
        &self,
        pattern: &Pattern,
        type_inferer: &crate::lang::types::TypeInferer,
        term_type: &sygus_parser::Sort,
    ) -> Result<(), String> {
        // Check if it's a simple constructor pattern
        if pattern.elements.len() == 1 {
            if let crate::lang::model::PatternElem::Var(constructor_name) = &pattern.elements[0] {
                // Verify constructor exists in the corresponding datatype
                if let Some(data_type) = self.find_datatype_by_sort(type_inferer, term_type) {
                    let constructor_exists = data_type
                        .constructors
                        .iter()
                        .any(|c| c.name == *constructor_name);
                    if !constructor_exists {
                        return Err(format!(
                            "Constructor '{}' not found in datatype '{}'",
                            constructor_name, data_type.name
                        ));
                    }
                }
                return Ok(());
            }
        }

        // For constructor patterns, validate constructor and parameter count
        if pattern.elements.len() >= 2 {
            if let crate::lang::model::PatternElem::Var(constructor_name) = &pattern.elements[0] {
                if let Some(data_type) = self.find_datatype_by_sort(type_inferer, term_type) {
                    if let Some(constructor_def) = data_type
                        .constructors
                        .iter()
                        .find(|c| c.name == *constructor_name)
                    {
                        let expected_params = pattern.elements.len() - 1;
                        let actual_params = constructor_def.fields.len();

                        if expected_params != actual_params {
                            return Err(format!(
                                "Constructor '{}' expects {} parameters, but {} were provided in pattern",
                                constructor_name, actual_params, expected_params
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Constructor '{}' not found in datatype '{}'",
                            constructor_name, data_type.name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Find datatype by sort
    fn find_datatype_by_sort<'a>(
        &self,
        type_inferer: &'a crate::lang::types::TypeInferer,
        sort: &sygus_parser::Sort,
    ) -> Option<&'a crate::lang::types::DataType> {
        let sort_name = sort.to_string();
        type_inferer.global_scope().get_datatype(&sort_name)
    }

    /// Bind pattern variables for type inference
    pub fn bind_pattern_variables_for_type_inference(
        &self,
        pattern: &Pattern,
        type_inferer: &mut crate::lang::types::TypeInferer,
        term_type: &sygus_parser::Sort,
    ) -> Result<(), String> {
        debug!("Binding pattern variables for type inference: {}", pattern);

        // Extract constructor parameters (skip constructor name)
        let params = self.extract_constructor_params(pattern);
        debug!("Extracted constructor parameters: {:?}", params);

        // Find data type definition
        if let Some(data_type) = self.find_datatype_by_sort(type_inferer, term_type) {
            debug!(
                "Found data type: {} with {} constructors",
                data_type.name,
                data_type.constructors.len()
            );

            // Collect variables and types to bind
            let mut bindings = Vec::new();

            // Handle variable binding in pattern
            if pattern.elements.len() >= 1 {
                // First element should be constructor name
                if let crate::lang::model::PatternElem::Var(constructor_name) = &pattern.elements[0] {
                    debug!("Looking for constructor: {}", constructor_name);
                    // Find constructor definition
                    if let Some(constructor_def) = data_type
                        .constructors
                        .iter()
                        .find(|c| c.name == *constructor_name)
                    {
                        debug!(
                            "Found constructor: {} with {} fields",
                            constructor_name,
                            constructor_def.fields.len()
                        );

                        // Check if the number of parameters matches
                        if params.len() != constructor_def.fields.len() {
                            return Err(format!(
                                "Constructor '{}' expects {} parameters, but {} were provided in pattern",
                                constructor_name,
                                constructor_def.fields.len(),
                                params.len()
                            ));
                        }

                        // Bind constructor parameters using extracted parameters
                        for (i, param_name) in params.iter().enumerate() {
                            if i < constructor_def.fields.len() {
                                let field_type = constructor_def.fields[i].sort.clone();
                                debug!("{} :: {}", param_name, field_type);
                                bindings.push((param_name.clone(), field_type));
                            }
                        }
                    } else {
                        return Err(format!(
                            "Constructor '{}' not found in datatype '{}'",
                            constructor_name, data_type.name
                        ));
                    }
                }
            }

            // Bind all variables at once
            for (var_name, var_type) in bindings {
                type_inferer.current_scope_mut().bind_variable(var_name, var_type);
            }
        } else {
            debug!("No datatype found for sort: {}", term_type);
        }

        Ok(())
    }

    /// Recursive pattern matching implementation
    fn match_pattern(
        &self,
        value: &Value,
        pattern: &Pattern,
        context: &mut PatternContext,
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

    /// Match a single pattern element with type-aware context
    fn match_single_element(
        &self,
        value: &Value,
        element: &PatternElem,
        context: &mut PatternContext,
    ) -> Result<bool, String> {
        match element {
            PatternElem::SortElem(enum_elem) => {
                // Match enum elements
                self.match_enum_element(value, enum_elem)
            }
            PatternElem::Var(var_name) => {
                // Check if this variable name is actually a constructor
                if context.is_constructor(var_name) {
                    // This is a constructor pattern, not a variable binding
                    self.match_constructor_name(value, var_name)
                } else if context.is_enum_element(var_name).is_some() {
                    // This is an enum element pattern, not a variable binding
                    // Compare the value directly with the enum element name
                    match value {
                        Value::Symbol(sym) => Ok(sym == var_name),
                        _ => Ok(false)
                    }
                } else {
                    // This is a variable binding
                    context.bind_variable(var_name.clone(), value.clone());
                    Ok(true)
                }
            }
            PatternElem::Wildcard => {
                // Wildcard - always succeeds
                Ok(true)
            }
        }
    }

    /// Match constructor name against value
    fn match_constructor_name(&self, value: &Value, constructor_name: &str) -> Result<bool, String> {
        match value {
            Value::DataValue { constructor, .. } => {
                Ok(constructor == constructor_name)
            }
            Value::Symbol(sym) => {
                Ok(sym == constructor_name)
            }
            _ => {
                // For other types, no match in constructor position
                Ok(false)
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
        context: &mut PatternContext,
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

        // Verify constructor exists in type environment
        if !context.is_constructor(constructor_name) {
            return Err(format!("Constructor '{}' not found in type environment", constructor_name));
        }

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
    pub fn match_cases_with_context(
        &self,
        value: &Value,
        cases: &[(Pattern, Value)],
        context: PatternContext,
    ) -> Result<Option<(Value, PatternContext)>, String> {
        for (pattern, case_value) in cases {
            let result = self.matches_with_context(value, pattern, context.clone())?;
            if result.matched {
                return Ok(Some((case_value.clone(), result.context)));
            }
        }
        Ok(None)
    }

    /// Legacy method for backward compatibility
    pub fn match_cases(
        &self,
        value: &Value,
        cases: &[(Pattern, Value)],
    ) -> Result<Option<(Value, Scope)>, String> {
        for (pattern, case_value) in cases {
            let result = self.matches(value, pattern)?;
            if result.matched {
                return Ok(Some((case_value.clone(), result.context.runtime_scope)));
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
    fn matches_pattern_with_context(
        &self,
        pattern: &Pattern,
        context: PatternContext,
    ) -> Result<PatternMatchResult, String>;
    fn match_with_type_validation(
        &self,
        pattern: &Pattern,
        type_inferer: &crate::lang::types::TypeInferer,
        term_type: &sygus_parser::Sort,
    ) -> Result<PatternMatchResult, String>;
}

impl PatternMatchable for Value {
    fn matches_pattern(&self, pattern: &Pattern) -> Result<PatternMatchResult, String> {
        let matcher = PatternMatcher::new();
        matcher.matches(self, pattern)
    }

    fn matches_pattern_with_context(
        &self,
        pattern: &Pattern,
        context: PatternContext,
    ) -> Result<PatternMatchResult, String> {
        let matcher = PatternMatcher::new();
        matcher.matches_with_context(self, pattern, context)
    }

    fn match_with_type_validation(
        &self,
        pattern: &Pattern,
        type_inferer: &crate::lang::types::TypeInferer,
        term_type: &sygus_parser::Sort,
    ) -> Result<PatternMatchResult, String> {
        let matcher = PatternMatcher::new();
        matcher.match_with_type_validation(self, pattern, type_inferer, term_type)
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

    /// Check if a value matches any pattern in a list with context
    pub fn matches_any_with_context(
        value: &Value,
        patterns: &[Pattern],
        context: PatternContext,
    ) -> Result<bool, String> {
        let matcher = PatternMatcher::new();
        for pattern in patterns {
            let result = matcher.matches_with_context(value, pattern, context.clone())?;
            if result.matched {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Create a pattern context from a type environment
    pub fn create_context_from_type_env(type_env: TypeEnv) -> PatternContext {
        PatternContext::new(type_env)
    }
}
