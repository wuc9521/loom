use crate::lang::model::MatchCase;
use crate::parser::ast::{LSTerm, LSVarBinding};
use crate::ANY_TYPE;
use log::{debug, error};
use sygus_parser::{Identifier, Literal, Sort, SortedVar};

use super::basic::EnumType;
use super::environment::TypeEnv;
use super::function::FunctionType;

/// type inferer
#[derive(Debug, Clone)]
pub struct TypeInferer {
    pub stack: Vec<TypeEnv>, // unified scope stack, manage all type environments
}

impl TypeInferer {
    pub fn new() -> Self {
        let mut inferer = Self { stack: Vec::new() };
        inferer.stack.push(TypeEnv::new());
        inferer.register_builtins();
        inferer
    }

    /// check if there is a symbol conflict in the global scope
    pub fn check_global_conflict(&self, name: &str, symbol_type: &str) -> Result<(), String> {
        let global_scope = self.global_scope();
        if global_scope.has_symbol(name) {
            let existing_type = global_scope.get_symbol_type(name).unwrap_or("unknown");
            Err(format!(
                "Symbol '{}' is already defined as {} in global scope, cannot redefine as {}",
                name, existing_type, symbol_type
            ))
        } else {
            Ok(())
        }
    }

    /// check if there is a symbol conflict in the current scope
    pub fn check_current_conflict(&self, name: &str, symbol_type: &str) -> Result<(), String> {
        let current_scope = self.current_scope();
        if current_scope.has_symbol(name) {
            let existing_type = current_scope.get_symbol_type(name).unwrap_or("unknown");
            Err(format!(
                "Symbol '{}' is already defined as {} in current scope, cannot redefine as {}",
                name, existing_type, symbol_type
            ))
        } else {
            Ok(())
        }
    }

    /// check if there is a symbol conflict in all scopes
    pub fn check_all_conflicts(&self, name: &str, symbol_type: &str) -> Result<(), String> {
        // first check global conflict
        self.check_global_conflict(name, symbol_type)?;
        // then check current scope conflict
        self.check_current_conflict(name, symbol_type)?;
        Ok(())
    }

    pub fn current_scope(&self) -> &TypeEnv {
        &self.stack[self.stack.len() - 1]
    }

    /// get current scope mutable reference
    pub fn current_scope_mut(&mut self) -> &mut TypeEnv {
        let len = self.stack.len();
        &mut self.stack[len - 1]
    }

    /// get global scope
    pub fn global_scope(&self) -> &TypeEnv {
        &self.stack[0]
    }

    /// get global scope mutable reference
    pub fn global_scope_mut(&mut self) -> &mut TypeEnv {
        &mut self.stack[0]
    }

    pub fn enter_scope(&mut self) {
        debug!("Entering scope, stack size: {}", self.stack.len());
        self.stack.push(TypeEnv::new());
        debug!("Entered scope, new stack size: {}", self.stack.len());
    }

    pub fn exit_scope(&mut self) {
        debug!("Exiting scope, stack size: {}", self.stack.len());
        if self.stack.len() > 1 {
            self.stack.pop();
            debug!("Exited scope, new stack size: {}", self.stack.len());
        } else {
            debug!("Cannot exit scope, stack size is 1");
        }
    }

    pub fn infer(&mut self, expr: &LSTerm) -> Result<Sort, String> {
        let result = match expr {
            LSTerm::Literal(lit) => {
                match lit {
                    Literal::Numeral(_) => Ok(Sort::int()),
                    Literal::Decimal(_) => Ok(Sort::real()),
                    Literal::Bool(_) => Ok(Sort::bool()),
                    _ => Ok(Sort::string()), // default string type
                }
            }
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                debug!("Looking up identifier: '{}'", sym);
                if let Some(sort) = self.find_variable_type(&sym) {
                    debug!("Found variable '{}' with type {}", sym, sort);
                    Ok(sort.clone())
                } else {
                    debug!(
                        "Variable '{}' not found, checking functions, constructors and enum elements",
                        sym
                    );

                    // check if it is a function (including nullary functions/constants)
                    if let Some(func_type) = self.find_function_type(&sym) {
                        debug!("Found function: {} with type {}", sym, func_type);
                        return Ok(func_type.get_ret());
                    }

                    // check if it is a constructor (nullary constructor)
                    for scope in &self.stack {
                        for (_, datatype) in &scope.datatypes {
                            for constructor in &datatype.constructors {
                                if constructor.name == sym && constructor.fields.is_empty() {
                                    debug!(
                                        "Found constructor: {} for datatype {}",
                                        sym, datatype.name
                                    );
                                    return Ok(Sort::from_name(datatype.name.as_str()));
                                }
                            }
                        }
                    }

                    // check if it is an enum value
                    for scope in &self.stack {
                        for (_, enum_type) in &scope.enums {
                            if enum_type.elements.contains(&sym) {
                                debug!("Found enum element: {} for enum {}", sym, enum_type.name);
                                return Ok(Sort::from_name(enum_type.name.as_str()));
                            }
                        }
                    }

                    error!("symbol type not found: {}", sym);
                    Err(format!("Undefined variable: {}", sym))
                }
            }
            LSTerm::AppTerm(func, args) => self.infer_function_call(&func.to_string(), args),
            LSTerm::LetTerm {
                args: bindings,
                body,
            } => self.infer_let_expr(bindings, body),
            LSTerm::LambdaTerm { args, body } => self.infer_lam_expr(args, body),
            LSTerm::MuTerm {
                name,
                args,
                body,
                params,
                ret_sort,
            } => self.infer_modal_expr(name, args, ret_sort, body, params),
            LSTerm::NuTerm {
                name,
                args,
                body,
                params,
                ret_sort,
            } => self.infer_modal_expr(name, args, ret_sort, body, params),
            LSTerm::MatchTerm { term, cases } => self.infer_match_expr(term, cases),
            LSTerm::ForallTerm { args, body } => self.infer_quantifier_expr(args, body),
            LSTerm::ExistsTerm { args, body } => self.infer_quantifier_expr(args, body),
            _ => {
                Ok(Sort::string()) // default type
            }
        };
        if let Ok(sort) = &result {
            debug!("{} :: {}", expr, sort);
        } else {
            error!("type inference failed: {:?}", result.as_ref().unwrap_err());
        }
        result
    }

    fn infer_function_call(&mut self, func: &str, args: &[LSTerm]) -> Result<Sort, String> {
        // infer argument types
        let mut arg_types = Vec::new();
        for arg in args.iter() {
            let arg_type = self.infer(arg)?;
            arg_types.push(arg_type);
        }

        // find function type
        if let Some(func_type) = self.find_function_type(func) {
            debug!("Found function type: {}", func_type);
            let params = func_type.get_params();
            let ret = func_type.get_ret();

            // check argument number
            if params.len() != arg_types.len() {
                return Err(format!(
                    "Function {} expects {} arguments, but {} were provided",
                    func,
                    params.len(),
                    arg_types.len()
                ));
            }

            // check argument types
            for (i, (expected, actual)) in params.iter().zip(arg_types.iter()).enumerate() {
                if !self.checked(expected, actual) {
                    return Err(format!(
                        "Argument {} of function {} has type {}, expected {}",
                        i + 1,
                        func,
                        actual,
                        expected
                    ));
                }
            }

            Ok(ret)
        } else {
            // check if it is a constructor
            for scope in &self.stack {
                for (_, datatype) in &scope.datatypes {
                    for constructor in &datatype.constructors {
                        if constructor.name == func {
                            debug!("Found constructor: {} for datatype {}", func, datatype.name);

                            // check if the number of parameters matches
                            if args.len() != constructor.fields.len() {
                                return Err(format!(
                                    "Constructor {} expects {} arguments, got {}",
                                    func,
                                    constructor.fields.len(),
                                    args.len()
                                ));
                            }

                            // check if the parameter types match
                            for (i, (field, arg_type)) in
                                constructor.fields.iter().zip(arg_types.iter()).enumerate()
                            {
                                if !self.checked(&field.sort, arg_type) {
                                    return Err(format!(
                                        "Argument {} of constructor {} has type {}, expected {}",
                                        i + 1,
                                        func,
                                        arg_type,
                                        field.sort
                                    ));
                                }
                            }

                            // return the datatype
                            return Ok(Sort::from_name(datatype.name.as_str()));
                        }
                    }
                }
            }

            // check if it is an enum element
            for scope in &self.stack {
                for (_, enum_type) in &scope.enums {
                    if enum_type.elements.contains(&func.to_string()) {
                        debug!("Found enum element: {} for enum {}", func, enum_type.name);
                        // Enum elements are constants, not functions
                        if args.is_empty() {
                            return Ok(Sort::from_name(enum_type.name.as_str()));
                        } else {
                            return Err(format!("Enum element '{}' is not a function and cannot be called with arguments", func));
                        }
                    }
                }
            }

            // check if it is a builtin function
            self.infer_builtin_function(func, args)
        }
    }

    fn infer_match_expr(
        &mut self,
        term: &Box<LSTerm>,
        cases: &[MatchCase],
    ) -> Result<Sort, String> {
        debug!("Inferring match term: {}", term);
        debug!("Current scope stack size: {}", self.stack.len());
        // infer term type
        let term_type = self.infer(term)?;
        debug!("Match term type: {}", term_type);

        // infer all case types
        let mut case_types = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            debug!("Processing case {}: {}", i, case);

            // enter new scope to handle each case
            self.enter_scope();

            // handle variable binding in pattern matching
            if let Some(pattern) = &case.pattern {
                debug!("Binding pattern variables for: {}", pattern);
                debug!("Pattern elements: {:?}", pattern.elements);
                self.bind_pattern_variables(pattern, &term_type)?;
            }

            // infer the type of the case value
            debug!("Inferring type for case value: {}", case.value);
            let case_type = self.infer(&case.value)?;
            case_types.push(case_type);

            // exit scope
            self.exit_scope();
        }
        // check if all case types are the same
        if case_types.is_empty() {
            return Err("Match expression must have at least one case".to_string());
        }

        let first_type = &case_types[0];
        for (i, case_type) in case_types.iter().enumerate() {
            if case_type != first_type {
                return Err(format!(
                    "Match case {} has type {}, expected {}",
                    i + 1,
                    case_type,
                    first_type
                ));
            }
        }

        Ok(first_type.clone())
    }

    /// bind variables in pattern matching - fixed version
    pub fn bind_pattern_variables(
        &mut self,
        pattern: &crate::lang::model::Pattern,
        term_type: &Sort,
    ) -> Result<(), String> {
        debug!("Binding pattern variables: {}", pattern);

        // Use the unified pattern matcher for variable binding
        let matcher = crate::lang::pattern::PatternMatcher::new();
        matcher.bind_pattern_variables_for_type_inference(
            pattern,
            self.current_scope_mut(),
            term_type,
        )
    }

    fn infer_let_expr(
        &mut self,
        bindings: &[LSVarBinding],
        body: &Box<LSTerm>,
    ) -> Result<Sort, String> {
        // enter new scope
        self.enter_scope();

        // bind variable types
        for binding in bindings {
            let var_type = self.infer(&binding.term)?;
            self.current_scope_mut()
                .bind_variable(binding.name.clone(), var_type);
        }
        let body_type = self.infer(body)?;
        self.exit_scope();
        Ok(body_type)
    }

    fn infer_lam_expr(&mut self, args: &[SortedVar], body: &Box<LSTerm>) -> Result<Sort, String> {
        // enter new scope
        self.enter_scope();
        for arg in args {
            self.current_scope_mut()
                .bind_variable(arg.name.clone(), arg.sort.clone());
        }

        // infer body type
        let body_type = self.infer(body)?;

        // exit scope
        self.exit_scope();

        // construct function type using proper Parameterized type
        let param_types: Vec<Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
        let mut result_type = body_type;
        for param_type in param_types.iter().rev() {
            result_type = Sort::arrow(param_type.clone(), result_type);
        }
        Ok(result_type)
    }

    fn checked(&self, t1: &Sort, t2: &Sort) -> bool {
        // if types are the same, return true
        if t1 == t2 {
            return true;
        }
        if let Sort::Simple(Identifier::Symbol(name)) = t1 {
            if name == ANY_TYPE {
                return true;
            }
        }
        if let Sort::Simple(Identifier::Symbol(name)) = t2 {
            if name == ANY_TYPE {
                return true;
            }
        }

        // if two are type identifiers, they are compatible
        if let (Sort::Simple(Identifier::Symbol(_)), Sort::Simple(Identifier::Symbol(_))) = (t1, t2)
        {
            return true;
        }

        // otherwise, check type alias
        let t1_resolved = self.resolve_type_alias(t1);
        let t2_resolved = self.resolve_type_alias(t2);
        t1_resolved == t2_resolved
    }

    pub fn find_variable_type(&self, name: &str) -> Option<&Sort> {
        debug!(
            "Looking up variable type for '{}' in {} scopes",
            name,
            self.stack.len()
        );
        // lookup in scope stack (from top to bottom, i.e. from recent to distant)
        for (i, scope) in self.stack.iter().enumerate().rev() {
            debug!("Checking scope {} for variable '{}'", i, name);
            if let Some(sort) = scope.get_variable_type(name) {
                debug!(
                    "Found variable '{}' in scope {} with type {}",
                    name, i, sort
                );
                return Some(sort);
            }
        }
        debug!("Variable '{}' not found in any scope", name);
        None
    }

    fn find_function_type(&self, name: &str) -> Option<&FunctionType> {
        debug!(
            "Searching for function type '{}' in {} scopes",
            name,
            self.stack.len()
        );
        for (i, scope) in self.stack.iter().enumerate() {
            debug!(
                "Checking scope {}: function_types = {:?}",
                i,
                scope.function_types.keys().collect::<Vec<_>>()
            );
            if let Some(func_type) = scope.function_types.get(name) {
                debug!("Found function type for '{}': {}", name, func_type);
                return Some(func_type);
            }
        }
        debug!("No function type found for '{}'", name);
        None
    }

    pub fn find_enum_type(&self, name: &str) -> Option<&EnumType> {
        // lookup in scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(enum_type) = scope.get_enum(name) {
                return Some(enum_type);
            }
        }
        None
    }

    fn resolve_type_alias(&self, sort: &Sort) -> Sort {
        self.resolve_type_alias_with_visited(sort, &mut std::collections::HashSet::new())
    }

    fn resolve_type_alias_with_visited(
        &self,
        sort: &Sort,
        visited: &mut std::collections::HashSet<String>,
    ) -> Sort {
        match sort {
            Sort::Simple(Identifier::Symbol(name)) => {
                // check if the type has been visited, avoid circular reference
                if visited.contains(name) {
                    debug!("Circular type alias detected for: {}", name);
                    return sort.clone();
                }

                // check type alias
                if let Some(aliased_sort) = self.find_variable_type(name) {
                    debug!("{} := {}", name, aliased_sort);
                    visited.insert(name.clone());
                    let result = self.resolve_type_alias_with_visited(aliased_sort, visited);
                    visited.remove(name);
                    result
                } else {
                    sort.clone()
                }
            }
            _ => sort.clone(),
        }
    }

    fn infer_modal_expr(
        &mut self,
        name: &str,
        args: &[SortedVar],
        ret_sort: &Sort,
        body: &Box<LSTerm>,
        params: &[LSTerm],
    ) -> Result<Sort, String> {
        // enter new scope
        self.enter_scope();

        // bind parameter types
        for arg in args {
            self.current_scope_mut()
                .bind_variable(arg.name.clone(), arg.sort.clone());
        }

        // register recursive function to current environment
        let param_types: Vec<Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
        self.current_scope_mut().register_function_type(
            name.to_string(),
            FunctionType::new_signature(name.to_string(), param_types, ret_sort.clone()),
        );

        // infer body type
        let body_type = self.infer(body)?;

        // For mu expressions, the body should be a function type
        // and the param should match the function's parameter types
        debug!("Mu expression body type: {:?}", body_type);
        if let Sort::Parameterized(identifier, _) = &body_type {
            debug!(
                "Mu expression body is Parameterized with identifier: {:?}",
                identifier
            );
            if let Identifier::Symbol(sym) = identifier {
                debug!("Mu expression body identifier symbol: {}", sym);
                if sym == "->" {
                    // Decompose the function type to get parameter types and return type
                    let (func_params, func_ret) = FunctionType::decompose_arrow_sort(&body_type);

                    // Check that function return type matches declared return type
                    if func_ret != *ret_sort {
                        return Err(format!(
                            "Mu expression function return type {} does not match declared return type {}",
                            func_ret, ret_sort
                        ));
                    }

                    // Check that param type matches function parameter types
                    let param_type = self.infer(&params[0])?;
                    // TODO:
                    if !func_params.is_empty() && param_type != func_params[0] {
                        return Err(format!(
                            "Mu expression param type {} does not match function parameter type {}",
                            param_type, func_params[0]
                        ));
                    }
                } else {
                    return Err(format!(
                        "Mu expression body type {} is not a function type",
                        body_type
                    ));
                }
            }
        } else {
            return Err(format!(
                "Mu expression body type {} is not a function type",
                body_type
            ));
        }

        // exit scope
        self.exit_scope();

        debug!("mu expression type inference completed: {}", ret_sort);
        Ok(ret_sort.clone())
    }

    fn infer_quantifier_expr(
        &mut self,
        args: &[SortedVar],
        body: &Box<LSTerm>,
    ) -> Result<Sort, String> {
        self.enter_scope();
        for arg in args {
            self.current_scope_mut()
                .bind_variable(arg.name.clone(), arg.sort.clone());
        }
        let body_type = self.infer(body)?;
        self.exit_scope();

        // check body type must be Bool
        if body_type != Sort::bool() {
            return Err(format!(
                "Quantifier body type {} must be Bool, but got {}",
                body_type, body_type
            ));
        }

        // quantifier expression returns Bool type
        Ok(Sort::bool())
    }

    /// Infer type of method call
    pub fn infer_method_call(
        &mut self,
        receiver: &str,
        method: &str,
        args: &[LSTerm],
    ) -> Result<Sort, String> {
        debug!("Inferring method call: {}.{}", receiver, method);

        // Handle "self" specially
        if receiver == "self" {
            // Look for self binding in current scope
            let self_type = self.find_variable_type("self").cloned();
            if let Some(self_type) = self_type {
                return self.infer_method_on_type(&self_type, method, args);
            }
            return Err("'self' used outside of class/object context".to_string());
        }

        // Look up receiver type
        let receiver_type = self.find_variable_type(receiver).cloned();
        if let Some(receiver_type) = receiver_type {
            return self.infer_method_on_type(&receiver_type, method, args);
        }

        // Check if receiver is an object
        let object_info = self
            .stack
            .iter()
            .rev()
            .find_map(|scope| scope.get_object_info(receiver))
            .cloned();

        if let Some((class_name, type_args)) = object_info {
            // Get class type
            let class = self
                .stack
                .iter()
                .rev()
                .find_map(|scope| scope.get_class(&class_name))
                .cloned();

            if let Some(class) = class {
                if let Some(method_type) = class.methods.get(method) {
                    // Type check arguments
                    let mut arg_types = Vec::new();
                    for arg in args {
                        arg_types.push(self.infer(arg)?);
                    }

                    let params = method_type.get_params();
                    let ret = method_type.get_ret();

                    // Check argument count and types
                    if params.len() != arg_types.len() {
                        return Err(format!(
                            "Method {} expects {} arguments, but {} were provided",
                            method,
                            params.len(),
                            arg_types.len()
                        ));
                    }

                    // Instantiate type parameters in method signature
                    let instantiated_ret =
                        self.instantiate_type(&ret, &class.type_params, &type_args);
                    return Ok(instantiated_ret);
                }
            }
        }

        Err(format!("Cannot find type for receiver '{}'", receiver))
    }

    /// Infer method call on a known type
    fn infer_method_on_type(
        &mut self,
        receiver_type: &Sort,
        method: &str,
        args: &[LSTerm],
    ) -> Result<Sort, String> {
        // Extract class name from receiver type
        let class_name = match receiver_type {
            Sort::Simple(Identifier::Symbol(name)) => name,
            _ => {
                return Err(format!(
                    "Cannot call method on non-class type: {}",
                    receiver_type
                ))
            }
        };

        // Find class definition
        let class = self
            .stack
            .iter()
            .rev()
            .find_map(|scope| scope.get_class(class_name))
            .cloned();

        if let Some(class) = class {
            if let Some(method_type) = class.methods.get(method) {
                // Type check arguments
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.infer(arg)?);
                }

                let params = method_type.get_params();
                let ret = method_type.get_ret();

                // Check argument count
                if params.len() != arg_types.len() {
                    return Err(format!(
                        "Method {} expects {} arguments, but {} were provided",
                        method,
                        params.len(),
                        arg_types.len()
                    ));
                }

                // Check argument types
                for (i, (expected, actual)) in params.iter().zip(arg_types.iter()).enumerate() {
                    if !self.checked(expected, actual) {
                        return Err(format!(
                            "Argument {} of method {} has type {}, expected {}",
                            i + 1,
                            method,
                            actual,
                            expected
                        ));
                    }
                }

                return Ok(ret);
            }

            // Check if it's a field accessor
            for (field_name, field_type) in &class.fields {
                if field_name == method {
                    // Field access is like a zero-argument method
                    if !args.is_empty() {
                        return Err(format!("Field {} access takes no arguments", field_name));
                    }
                    return Ok(field_type.clone());
                }
            }
        }

        Err(format!(
            "Method '{}' not found in type '{}'",
            method, class_name
        ))
    }

    /// Instantiate type parameters in a sort
    pub fn instantiate_type(
        &self,
        sort: &Sort,
        type_params: &[String],
        type_args: &[String],
    ) -> Sort {
        match sort {
            Sort::Simple(Identifier::Symbol(name)) => {
                // Check if it's a type parameter
                if let Some(index) = type_params.iter().position(|p| p == name) {
                    if index < type_args.len() {
                        return Sort::from_name(type_args[index].as_str());
                    }
                }
                sort.clone()
            }
            Sort::Parameterized(func, args) => {
                let instantiated_args: Vec<Sort> = args
                    .iter()
                    .map(|arg| self.instantiate_type(arg, type_params, type_args))
                    .collect();
                Sort::Parameterized(func.clone(), instantiated_args)
            }
            _ => sort.clone(),
        }
    }

    /// Check if a sort is an arrow type
    pub fn is_arrow_sort(&self, sort: &Sort) -> bool {
        match sort {
            Sort::Parameterized(Identifier::Symbol(name), _) => name == "->",
            _ => false,
        }
    }

    /// Decompose arrow sort into components
    pub fn decompose_arrow_sort(&self, sort: &Sort) -> Option<(Vec<Sort>, Sort)> {
        match sort {
            Sort::Parameterized(Identifier::Symbol(name), args)
                if name == "->" && args.len() == 2 =>
            {
                let arg_sort = args[0].clone();
                let ret_sort = args[1].clone();

                // Recursively decompose if return type is also an arrow
                if let Some((mut more_args, final_ret)) = self.decompose_arrow_sort(&ret_sort) {
                    more_args.insert(0, arg_sort);
                    Some((more_args, final_ret))
                } else {
                    Some((vec![arg_sort], ret_sort))
                }
            }
            _ => None,
        }
    }
}
