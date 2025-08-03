use crate::parser::ast::LSCmd;
use crate::lang::types::FunctionType;
use crate::lang::model::Pattern;
use crate::translator::core::SyGuSTranslator;
use sygus_parser::{SyGuSCmd, SyGuSTerm, SortedVar, Identifier, Sort};
use log::debug;

/// Bit-vector translation functionality for SyGuSTranslator
impl SyGuSTranslator {
    /// Calculate the cardinality of the semantic state for an object
    /// This function now correctly handles composite datatypes with generic parameters
    pub fn calculate_semantic_state_cardinality(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
        semantic_state_params: &[SortedVar],
    ) -> Result<usize, String> {
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        // For semantic functions, the cardinality is typically determined by the semantic state parameters
        // In most cases, there will be one main semantic state parameter (e.g., Interval)
        if semantic_state_params.is_empty() {
            return Err("No semantic state parameters found".to_string());
        }
        
        // Use the first semantic state parameter to determine cardinality
        let main_semantic_param = &semantic_state_params[0];
        let semantic_sort = &main_semantic_param.sort;
        
        debug!(
            "Calculating cardinality for semantic state parameter '{}' of type {}",
            main_semantic_param.name, semantic_sort
        );
        
        // Step 1: Try to find the datatype definition for this sort
        let datatype_name = if let sygus_parser::Sort::Simple(identifier) = semantic_sort {
            identifier.to_string()
        } else {
            return Err(format!(
                "Semantic state parameter '{}' has complex type {} - expected simple sort",
                main_semantic_param.name, semantic_sort
            ));
        };
        
        let datatype = type_env.get_datatype(&datatype_name)
            .ok_or_else(|| format!("Datatype '{}' not found in type environment", datatype_name))?;
        
        debug!("Found datatype '{}' with {} constructors", datatype_name, datatype.constructors.len());
        
        // Step 2: Get the class definition to understand generic parameters
        let class_def = self.find_class_for_object(object_def)?;
        debug!(
            "Object '{}' belongs to class '{}' with type parameters: {:?}",
            object_def.name, class_def.name, class_def.type_params
        );
        
        // Step 3: Create mapping from generic parameters to concrete types
        let mut type_mapping = std::collections::HashMap::new();
        for (i, type_param) in class_def.type_params.iter().enumerate() {
            if i < object_def.type_args.len() {
                let concrete_type = &object_def.type_args[i];
                type_mapping.insert(type_param.clone(), concrete_type.clone());
                debug!(
                    "Mapped generic parameter '{}' to concrete type '{}'",
                    type_param, concrete_type
                );
            } else {
                return Err(format!(
                    "Object '{}' has fewer type arguments ({}) than class '{}' has type parameters ({})",
                    object_def.name, object_def.type_args.len(), class_def.name, class_def.type_params.len()
                ));
            }
        }
        
        // Step 4: Calculate total cardinality by analyzing datatype constructors
        let mut total_cardinality = 1;
        
        for constructor in &datatype.constructors {
            debug!("Analyzing constructor '{}' with {} fields", constructor.name, constructor.fields.len());
            
            for field in &constructor.fields {
                let field_type = &field.sort;
                debug!("Analyzing field '{}' of type '{}'", field.name, field_type);
                
                // Step 5: Resolve the field type using the type mapping
                let resolved_type = self.resolve_generic_type(field_type, &type_mapping)?;
                debug!("Resolved field type '{}' to '{}'", field_type, resolved_type);
                
                // Step 6: Get the cardinality of the resolved type
                let field_cardinality = self.get_type_cardinality(&resolved_type, type_env)?;
                debug!("Field '{}' has cardinality {}", field.name, field_cardinality);
                
                // Step 7: Multiply into total cardinality
                total_cardinality *= field_cardinality;
            }
        }
        
        debug!(
            "Final cardinality for semantic state parameter '{}' of type '{}': {}",
            main_semantic_param.name, semantic_sort, total_cardinality
        );
        
        Ok(total_cardinality)
    }
    
    /// Resolve a generic type using the type mapping
    fn resolve_generic_type(
        &self,
        generic_type: &Sort,
        type_mapping: &std::collections::HashMap<String, Sort>,
    ) -> Result<Sort, String> {
        match generic_type {
            sygus_parser::Sort::Simple(identifier) => {
                let type_name = identifier.to_string();
                if let Some(concrete_type) = type_mapping.get(&type_name) {
                    Ok(concrete_type.clone())
                } else {
                    // If not in mapping, assume it's already concrete
                    Ok(generic_type.clone())
                }
            }
            sygus_parser::Sort::Parameterized(name, args) => {
                // Recursively resolve parameterized types
                let resolved_args: Result<Vec<Sort>, String> = args
                    .iter()
                    .map(|arg| self.resolve_generic_type(arg, type_mapping))
                    .collect();
                
                Ok(sygus_parser::Sort::Parameterized(
                    name.clone(),
                    resolved_args?,
                ))
            }
        }
    }
    
    /// Get the cardinality of a specific type from the type environment
    fn get_type_cardinality(
        &self,
        sort: &Sort,
        type_env: &crate::lang::types::TypeEnv,
    ) -> Result<usize, String> {
        // First try to get cardinality directly from the type environment
        if let Some(finiteness) = type_env.is_finite(sort) {
            match finiteness {
                crate::lang::types::Finiteness::Finite(cardinality) => {
                    debug!("Type '{}' has finite cardinality {}", sort, cardinality);
                return Ok(cardinality);
                }
                crate::lang::types::Finiteness::Infinite => {
                    return Err(format!("Type '{}' has infinite cardinality", sort));
                }
            }
        }
        
        // If not found, try to get cardinality by name
        if let sygus_parser::Sort::Simple(identifier) = sort {
            let type_name = identifier.to_string();
            if let Some(cardinality) = type_env.get_cardinality(&type_name) {
                debug!("Type '{}' has cardinality {} (by name)", type_name, cardinality);
                return Ok(cardinality);
            }
        }
        
        // For parameterized types, try to calculate from components
        if let sygus_parser::Sort::Parameterized(_, args) = sort {
            let mut total_cardinality = 1;
            for arg in args {
                let arg_cardinality = self.get_type_cardinality(arg, type_env)?;
                total_cardinality *= arg_cardinality;
            }
            debug!("Parameterized type '{}' has calculated cardinality {}", sort, total_cardinality);
            return Ok(total_cardinality);
        }
        
        Err(format!("Cannot determine cardinality for type '{}'", sort))
    }

    /// Find the class definition for an object
    pub fn find_class_for_object(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
    ) -> Result<&crate::parser::ast::ClassDef, String> {
        // Look for the class definition in the original file
        for cmd in &self.original_file.cmds {
            if let LSCmd::ClassDef(class_def) = cmd {
                if class_def.name == object_def.class_name {
                    return Ok(class_def);
                }
            }
        }
        Err(format!("Class definition not found for object {}", object_def.name))
    }

    /// Check if a function is a semantic function and return the scrutinee parameter name
    /// A function is semantic IF AND ONLY IF it contains a match AND is the designated
    /// recursive evaluator (e.g., its name is 'eval').
    pub fn is_semantic_function(
        &self,
        func_type: &FunctionType,
        _class_def: &crate::parser::ast::ClassDef,
    ) -> Result<Option<String>, String> {
        // Check if the function body contains a match expression
        if let Some(body) = func_type.get_body() {
            if !self.contains_match_term(body) {
                return Ok(None);
            }
            
            // Now find which parameter is being matched (the scrutinee)
            let scrutinee_name = self.find_match_scrutinee(func_type, body)?;
            Ok(Some(scrutinee_name))
        } else {
            Ok(None)
        }
    }

    /// Find the scrutinee parameter name by analyzing the match expression
    fn find_match_scrutinee(
        &self,
        func_type: &FunctionType,
        body: &crate::parser::ast::LSTerm,
    ) -> Result<String, String> {
        match body {
            crate::parser::ast::LSTerm::MatchTerm { term, .. } => {
                // The term being matched is the scrutinee
                // We need to find which parameter this term refers to
                self.extract_parameter_name_from_term(term, func_type)
            }
            crate::parser::ast::LSTerm::AppTerm(_, args) => {
                // Check if any of the arguments contain a match
                for arg in args {
                    if self.contains_match_term(arg) {
                        return self.find_match_scrutinee(func_type, arg);
                    }
                }
                Err("Could not find match expression in function body".to_string())
            }
            _ => Err("Function body does not contain a match expression".to_string()),
        }
    }

    /// Extract the parameter name from a term that appears in a match expression
    fn extract_parameter_name_from_term(
        &self,
        term: &crate::parser::ast::LSTerm,
        func_type: &FunctionType,
    ) -> Result<String, String> {
        match term {
            crate::parser::ast::LSTerm::Identifier(ident) => {
                // This is a direct parameter reference
                if let sygus_parser::Identifier::Symbol(param_name) = ident {
                    // Verify this parameter exists in the function
                    let param_names = func_type.get_param_names();
                    if param_names.contains(param_name) {
                        Ok(param_name.clone())
                    } else {
                        Err(format!("Parameter '{}' not found in function parameters", param_name))
                    }
                } else {
                    Err("Identifier in match term is not a symbol".to_string())
                }
            }
            crate::parser::ast::LSTerm::AppTerm(func, args) => {
                // Check if this is a field access on a parameter
                if let sygus_parser::Identifier::Symbol(_field_name) = func {
                    if let Some(first_arg) = args.first() {
                        return self.extract_parameter_name_from_term(first_arg, func_type);
                    }
                }
                Err("Complex term in match expression not supported".to_string())
            }
            _ => Err("Unsupported term type in match expression".to_string()),
        }
    }

    /// Get datatype constructors dynamically from TypeEnv
    pub fn get_datatype_constructors(
        &self,
        datatype_name: &str,
    ) -> Result<Vec<String>, String> {
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        if let Some(datatype) = type_env.get_datatype(datatype_name) {
            Ok(datatype.constructors.iter().map(|c| c.name.clone()).collect())
        } else {
            Err(format!("Datatype {} not found in type environment", datatype_name))
        }
    }

    /// Get the scrutinee (datatype parameter) from a semantic function
    /// This function now takes the scrutinee parameter name as input for accuracy
    pub fn get_semantic_scrutinee(
        &self,
        func_type: &FunctionType,
        scrutinee_name: &str,
    ) -> Result<(String, Sort), String> {
        let param_names = func_type.get_param_names();
        let param_sorts = func_type.get_params();
        
        // Find the scrutinee parameter by name
        for (i, name) in param_names.iter().enumerate() {
            if name == scrutinee_name {
                if i < param_sorts.len() {
                    return Ok((name.clone(), param_sorts[i].clone()));
                } else {
                    return Err(format!("Parameter '{}' found but sort not available", scrutinee_name));
                }
            }
        }
        
        Err(format!("Scrutinee parameter '{}' not found in function parameters", scrutinee_name))
    }

    /// Get semantic state parameters (all parameters except self and the scrutinee)
    /// This function now properly identifies parameters based on their names and roles
    pub fn get_semantic_state_params(
        &self,
        func_type: &FunctionType,
        scrutinee_name: &str,
    ) -> Result<Vec<SortedVar>, String> {
        let param_names = func_type.get_param_names();
        let param_sorts = func_type.get_params();
        
        if param_names.len() != param_sorts.len() {
            return Err("Parameter names and sorts have different lengths".to_string());
        }
        
        let mut semantic_state_params = Vec::new();
        
        for (_i, (name, sort)) in param_names.iter().zip(param_sorts.iter()).enumerate() {
            // Skip 'self' parameter (object instance)
            if name == "self" {
                continue;
            }
            
            // Skip the scrutinee parameter
            if name == scrutinee_name {
                continue;
            }
            
            // All other parameters are semantic state parameters
            semantic_state_params.push(SortedVar {
                name: name.clone(),
                sort: sort.clone(),
            });
        }
        
        Ok(semantic_state_params)
    }

    /// Extract constructor name from a pattern
    pub fn extract_constructor_name_from_pattern(
        &self,
        pattern: &Pattern,
    ) -> Result<String, String> {
        if pattern.elements.is_empty() {
            return Err("Empty pattern".to_string());
        }
        
        match &pattern.elements[0] {
            crate::lang::model::PatternElem::Var(name) => Ok(name.clone()),
            crate::lang::model::PatternElem::SortElem(elem) => Ok(elem.name.clone()),
            crate::lang::model::PatternElem::Wildcard => Err("Wildcard pattern not supported in semantic functions".to_string()),
        }
    }

    /// Translate a semantic function to bit-vector encoding
    pub fn translate_semantic_function(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        func_def: &FunctionType,
        class_def: &crate::parser::ast::ClassDef,
    ) -> Result<Vec<SyGuSCmd>, String> {
        // STEP 1: Get the scrutinee parameter name from the semantic function
        let scrutinee_param_name = self.is_semantic_function(func_def, class_def)?
            .ok_or("Function is not a semantic function")?;
        
        // STEP 2: Get the datatype definition from the TypeEnv (NO HARDCODING)
        let semantic_state_params = self.get_semantic_state_params(func_def, &scrutinee_param_name)?;
        let (_scrutinee_name, scrutinee_sort) = self.get_semantic_scrutinee(func_def, &scrutinee_param_name)?;
        let datatype_name = scrutinee_sort.clone().to_string();
        
        // Get the DataType struct from TypeEnv
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        let datatype = type_env.get_datatype(&datatype_name)
            .ok_or_else(|| format!("Datatype {} not found in type environment", datatype_name))?;
        
        // Extract constructor names from the datatype definition
        let constructors_from_datatype: Vec<String> = datatype.constructors.iter()
            .map(|c| c.name.clone())
            .collect();
        
        // STEP 2: Calculate the bit-vector size N
        let cardinality_n = self.calculate_semantic_state_cardinality(object_def, &semantic_state_params)?;

        let mut commands = Vec::new();
        
        // STEP 3: Find the match expression in the semantic function's body
        let match_term = if let Some(body) = func_def.get_body() {
            if let crate::parser::ast::LSTerm::MatchTerm { cases, .. } = body {
                cases
            } else {
                return Err("Semantic function body is not a MatchTerm".to_string());
            }
        } else {
            return Err("Semantic function has no body".to_string());
        };

        // STEP 4: Iterate over the DEFINED SEMANTICS in the match cases (NO HARDCODED LIST)
        for case in match_term {
            if let Some(pattern) = &case.pattern {
                let constructor_name = self.extract_constructor_name_from_pattern(pattern)?;

                // Generate the bit-vector helper function for this specific constructor
                // Pass the actual semantic expression (case.value) to avoid hardcoding
                let bv_func = self.generate_bv_constructor_function_dynamic(
                    object_def,
                    func_def,
                    &constructor_name,
                    &case.value, // Pass the actual semantic expression
                    cardinality_n,
                )?;
                commands.push(bv_func);
            }
        }

        // STEP 5: Generate the top-level recursive evaluator
        let eval_func = self.generate_object_evaluator_dynamic(
            object_def,
            func_def,
            class_def,
            &constructors_from_datatype, // Pass the dynamically discovered list here!
            cardinality_n,
        )?;
        commands.push(eval_func);

        Ok(commands)
    }

    /// Generate a bit-vector function for a specific constructor (dynamic version)
    /// This function generates the bit-vector helper functions that implement the semantic logic
    /// for each constructor of the scrutinee datatype
    pub fn generate_bv_constructor_function_dynamic(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        _func_def: &FunctionType,
        constructor_name: &str,
        semantic_expr: &crate::parser::ast::LSTerm,
        cardinality_n: usize,
    ) -> Result<SyGuSCmd, String> {
        let func_name = format!("{}_{}", object_def.name, constructor_name);
        let bv_sort = Sort::Simple(Identifier::Symbol(
            format!("(_ BitVec {})", cardinality_n)
        ));
        
        debug!("Generating BV helper function '{}' for constructor '{}'", func_name, constructor_name);
        
        // Get the datatype definition to access constructor information
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        // Find the datatype that contains this constructor
        let mut target_datatype = None;
        for (_, datatype) in &type_env.datatypes {
            if datatype.constructors.iter().any(|c| c.name == *constructor_name) {
                target_datatype = Some(datatype);
                break;
            }
        }
        
        let datatype = target_datatype
            .ok_or_else(|| format!("Constructor {} not found in any datatype", constructor_name))?;
        
        // Find the specific constructor
        let constructor = datatype.constructors.iter()
            .find(|c| c.name == *constructor_name)
            .ok_or_else(|| format!("Constructor {} not found in datatype", constructor_name))?;
        
        // Get the scrutinee type to identify recursive fields
        let class_def = self.find_class_for_object(object_def)?;
        let semantic_func = class_def.methods.iter()
            .find(|m| self.is_semantic_function(m, class_def).unwrap_or(None).is_some())
            .ok_or("Could not find semantic function in class definition")?;
        
        let scrutinee_param_name = self.is_semantic_function(semantic_func, class_def)?
            .ok_or("Function is not a semantic function")?;
        
        let (_, scrutinee_sort) = self.get_semantic_scrutinee(semantic_func, &scrutinee_param_name)?;
        
        // Generate parameters based on constructor arguments
        let mut sorted_vars = Vec::new();
        
        for (i, field) in constructor.fields.iter().enumerate() {
            let param_name = format!("bv_{}", i);
            
            // Check if this field type is recursive (same as scrutinee type)
            let param_sort = if &field.sort == &scrutinee_sort {
                // This is a recursive field - use bit-vector type
                debug!("Field '{}' is recursive (type: {}), using BV type", field.name, field.sort);
                bv_sort.clone()
            } else {
                // This is a non-recursive field - use the original field type
                debug!("Field '{}' is non-recursive (type: {}), using original type", field.name, field.sort);
                field.sort.clone()
            };
            
            sorted_vars.push(SortedVar {
                name: param_name,
                sort: param_sort,
            });
        }
        
        // Clone the constructor fields to avoid borrow checker issues
        let constructor_fields = constructor.fields.clone();
        
        // Generate the body by translating the semantic expression to bit-vector logic
        let body = self.translate_semantic_expr_to_bv_logic(
            object_def,
            semantic_expr,
            &constructor_fields,
            cardinality_n,
        )?;
        
        Ok(SyGuSCmd::DefineFun(
            func_name,
            sorted_vars,
            bv_sort,
            body,
        ))
    }

    /// Translate semantic expression to bit-vector logic
    /// This function converts the semantic logic from the match case into bit-vector operations
    fn translate_semantic_expr_to_bv_logic(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        semantic_expr: &crate::parser::ast::LSTerm,
        constructor_fields: &[crate::lang::types::Field],
        cardinality_n: usize,
    ) -> Result<SyGuSTerm, String> {
        debug!("Translating semantic expression to BV logic: {}", semantic_expr);
        
        // Get the scrutinee type to identify recursive fields
        let class_def = self.find_class_for_object(object_def)?;
        let semantic_func = class_def.methods.iter()
            .find(|m| self.is_semantic_function(m, class_def).unwrap_or(None).is_some())
            .ok_or("Could not find semantic function in class definition")?;
        
        let scrutinee_param_name = self.is_semantic_function(semantic_func, class_def)?
            .ok_or("Function is not a semantic function")?;
        
        let (_, scrutinee_sort) = self.get_semantic_scrutinee(semantic_func, &scrutinee_param_name)?;
        
        // Create a mapping from field names to bit-vector arguments
        // Only recursive fields get bit-vector arguments
        let mut bv_args = Vec::new();
        let mut non_recursive_args = Vec::new();
        
        for (i, field) in constructor_fields.iter().enumerate() {
            if &field.sort == &scrutinee_sort {
                // This is a recursive field - add to bv_args
                bv_args.push(SyGuSTerm::Identifier(Identifier::Symbol(format!("bv_{}", i))));
            } else {
                // This is a non-recursive field - add to non_recursive_args
                non_recursive_args.push((i, field.clone()));
            }
        }
        
        // Initialize eval call counter
        let mut eval_call_counter = 0;
        self.translate_semantic_expr_recursive(object_def, semantic_expr, &bv_args, cardinality_n, &mut eval_call_counter)
    }
    
    /// Recursive translation of semantic expressions to bit-vector logic
    fn translate_semantic_expr_recursive(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        semantic_expr: &crate::parser::ast::LSTerm,
        bv_args: &[SyGuSTerm],
        cardinality_n: usize,
        eval_call_counter: &mut usize,
    ) -> Result<SyGuSTerm, String> {
        debug!("Recursively translating: {}", semantic_expr);
        
        match semantic_expr {
            // Boolean literals
            crate::parser::ast::LSTerm::Literal(sygus_parser::Literal::Bool(b)) => {
                if *b {
                    // true -> all ones bit-vector
                    Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                        format!("(_ bv{} {})", (1 << cardinality_n) - 1, cardinality_n)
                    )))
                } else {
                    // false -> all zeros bit-vector
                    Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                        format!("(_ bv0 {})", cardinality_n)
                    )))
                }
            }
            
            // Integer literals - convert to bit-vector
            crate::parser::ast::LSTerm::Literal(sygus_parser::Literal::Numeral(n)) => {
                Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                    format!("(_ bv{} {})", n, cardinality_n)
                )))
            }
            
            // Boolean operations
            crate::parser::ast::LSTerm::AppTerm(func, args) => {
                if let sygus_parser::Identifier::Symbol(func_name) = func {
                    match func_name.as_str() {
                        "or" => {
                            // (or A B) -> (bvor (translate A) (translate B))
                            if args.len() == 2 {
                                let left = self.translate_semantic_expr_recursive(object_def, &args[0], bv_args, cardinality_n, eval_call_counter)?;
                                let right = self.translate_semantic_expr_recursive(object_def, &args[1], bv_args, cardinality_n, eval_call_counter)?;
                                
                                Ok(SyGuSTerm::Application(
                                    Identifier::Symbol("bvor".to_string()),
                                    vec![left, right],
                                ))
                            } else {
                                Err("OR operator expects exactly 2 arguments".to_string())
                            }
                        }
                        "and" => {
                            // (and A B) -> (bvand (translate A) (translate B))
                            if args.len() == 2 {
                                let left = self.translate_semantic_expr_recursive(object_def, &args[0], bv_args, cardinality_n, eval_call_counter)?;
                                let right = self.translate_semantic_expr_recursive(object_def, &args[1], bv_args, cardinality_n, eval_call_counter)?;
                                
                                Ok(SyGuSTerm::Application(
                                    Identifier::Symbol("bvand".to_string()),
                                    vec![left, right],
                                ))
                            } else {
                                Err("AND operator expects exactly 2 arguments".to_string())
                            }
                        }
                        "not" => {
                            // (not A) -> (bvnot (translate A))
                            if args.len() == 1 {
                                let arg = self.translate_semantic_expr_recursive(object_def, &args[0], bv_args, cardinality_n, eval_call_counter)?;
                                
                                Ok(SyGuSTerm::Application(
                                    Identifier::Symbol("bvnot".to_string()),
                                    vec![arg],
                                ))
                            } else {
                                Err("NOT operator expects exactly 1 argument".to_string())
                            }
                        }
                        "=" => {
                            // Equality comparison - handle based on argument types
                            if args.len() == 2 {
                                self.translate_equality_comparison(object_def, &args[0], &args[1], bv_args, cardinality_n, eval_call_counter)
                            } else {
                                Err("Equality operator expects exactly 2 arguments".to_string())
                            }
                        }
                        "exists" | "forall" => {
                            // Handle quantifiers by unrolling them
                            if args.len() >= 2 {
                                // Parse the quantifier structure
                                // args[0] should be the binding list: ((cut Pos))
                                // args[1] should be the body expression
                                let binding_list = &args[0];
                                let body_expr = &args[1];
                                
                                // Parse the binding: ((cut Pos))
                                let (var_name, var_type) = self.parse_quantifier_binding(binding_list)?;
                                
                                // Get all possible values for the bound variable type
                                let type_env = self.context.type_environment.as_ref()
                                    .ok_or("Type environment not available")?;
                                
                                // Resolve the type using object context if needed
                                let resolved_type = self.resolve_type_with_object_context(var_type, object_def)?;
                                let possible_values = self.get_all_values_for_type(&resolved_type, type_env)?;
                                
                                debug!("Unrolling {} quantifier for variable '{}' of type '{}' with {} possible values", 
                                       func_name, var_name, resolved_type, possible_values.len());
                                
                                // Create a list of expressions by substituting each possible value
                                let mut substituted_expressions = Vec::new();
                                for value in &possible_values {
                                    let substituted_expr = self.substitute_variable_in_term(body_expr, &var_name, value)?;
                                    substituted_expressions.push(substituted_expr);
                                }
                                
                                // Translate each substituted expression
                                let mut translated_expressions = Vec::new();
                                for expr in &substituted_expressions {
                                    let translated = self.translate_semantic_expr_recursive(
                                        object_def, expr, bv_args, cardinality_n, eval_call_counter
                                    )?;
                                    translated_expressions.push(translated);
                                }
                                
                                // Combine the results based on quantifier type
                                if func_name == "exists" {
                                    // exists -> OR of all possibilities
                                    if translated_expressions.is_empty() {
                                        // No values to exist over -> false
                                        Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                                            format!("(_ bv0 {})", cardinality_n)
                                        )))
                                    } else if translated_expressions.len() == 1 {
                                        // Single value -> just return it
                                        Ok(translated_expressions.remove(0))
                                    } else {
                                        // Multiple values -> combine with bvor
                                        let mut result = translated_expressions.remove(0);
                                        for expr in translated_expressions {
                                            result = SyGuSTerm::Application(
                                                Identifier::Symbol("bvor".to_string()),
                                                vec![result, expr],
                                            );
                                        }
                                        Ok(result)
                                    }
                                } else {
                                    // forall -> AND of all possibilities
                                    if translated_expressions.is_empty() {
                                        // No values to forall over -> true
                                        Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                                            format!("(_ bv{} {})", (1 << cardinality_n) - 1, cardinality_n)
                                        )))
                                    } else if translated_expressions.len() == 1 {
                                        // Single value -> just return it
                                        Ok(translated_expressions.remove(0))
                                    } else {
                                        // Multiple values -> combine with bvand
                                        let mut result = translated_expressions.remove(0);
                                        for expr in translated_expressions {
                                            result = SyGuSTerm::Application(
                                                Identifier::Symbol("bvand".to_string()),
                                                vec![result, expr],
                                            );
                                        }
                                        Ok(result)
                                    }
                                }
                            } else {
                                Err("Quantifier must have at least 2 arguments: binding list and body".to_string())
                            }
                        }
                        _ => {
                            // All other function calls are treated as fixed expressions
                            self.translate_fixed_function_call(object_def, func_name, args, bv_args, cardinality_n, eval_call_counter)
                        }
                    }
                } else {
                    Err("Function identifier is not a symbol".to_string())
                }
            }
            
            // Field access - convert to bit-vector parameter reference
            crate::parser::ast::LSTerm::Identifier(ident) => {
                if let sygus_parser::Identifier::Symbol(var_name) = ident {
                    // Check for boolean literals first
                    match var_name.as_str() {
                        "true" => {
                            // true -> all ones bit-vector
                            Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                                format!("(_ bv{} {})", (1 << cardinality_n) - 1, cardinality_n)
                            )))
                        }
                        "false" => {
                            // false -> all zeros bit-vector
                            Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                                format!("(_ bv0 {})", cardinality_n)
                            )))
                        }
                        _ => {
                            // Check if this is a field access that should be converted to a bit-vector parameter
                            // For now, we'll use a simple heuristic based on the variable name
                            // In a more sophisticated implementation, you would need to pass constructor_fields
                            
                            // If not a field, it might be a constant or other variable
                            // For now, return a placeholder
                            Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                                format!("__placeholder_{}__", var_name)
                            )))
                        }
                    }
                } else {
                    Err("Identifier is not a symbol".to_string())
                }
            }
            
            // Match expressions - should not occur in semantic expressions for constructors
            crate::parser::ast::LSTerm::MatchTerm { .. } => {
                Err("Match expressions should not occur in constructor semantic expressions".to_string())
            }
            
            // Other term types - return placeholders for now
            _ => {
                debug!("Unsupported term type in semantic expression: {}", semantic_expr);
                Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                    format!("__unsupported_term_placeholder__")
                )))
            }
        }
    }
    
    /// Translate equality comparison to bit-vector logic
    fn translate_equality_comparison(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        left: &crate::parser::ast::LSTerm,
        right: &crate::parser::ast::LSTerm,
        bv_args: &[SyGuSTerm],
        cardinality_n: usize,
        eval_call_counter: &mut usize,
    ) -> Result<SyGuSTerm, String> {
        // Check if this is a comparison that can be evaluated as a fixed expression
        if self.is_fixed_comparison(left, right) {
            // This is a fixed comparison - evaluate it and return constant bit-vector
            let is_equal = self.evaluate_fixed_comparison(left, right)?;
            if is_equal {
                Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                    format!("(_ bv{} {})", (1 << cardinality_n) - 1, cardinality_n)
                )))
            } else {
                Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                    format!("(_ bv0 {})", cardinality_n)
                )))
            }
        } else {
            // This is a dynamic comparison - translate both sides and compare
            let left_bv = self.translate_semantic_expr_recursive(object_def, left, bv_args, cardinality_n, eval_call_counter)?;
            let right_bv = self.translate_semantic_expr_recursive(object_def, right, bv_args, cardinality_n, eval_call_counter)?;
            
            // Convert to bit-vector: if equal, all ones; if not equal, all zeros
            Ok(SyGuSTerm::Application(
                Identifier::Symbol("ite".to_string()),
                vec![
                    SyGuSTerm::Application(
                        Identifier::Symbol("=".to_string()),
                        vec![left_bv, right_bv],
                    ),
                    SyGuSTerm::Identifier(Identifier::Symbol(
                        format!("(_ bv{} {})", (1 << cardinality_n) - 1, cardinality_n)
                    )),
                    SyGuSTerm::Identifier(Identifier::Symbol(
                        format!("(_ bv0 {})", cardinality_n)
                    )),
                ],
            ))
        }
    }
    
    /// Check if a comparison is fixed (doesn't depend on dynamic values)
    fn is_fixed_comparison(
        &self,
        left: &crate::parser::ast::LSTerm,
        right: &crate::parser::ast::LSTerm,
    ) -> bool {
        // A comparison is fixed if both sides are literals or fixed function calls
        self.is_fixed_expression(left) && self.is_fixed_expression(right)
    }
    
    /// Check if an expression is fixed (doesn't depend on dynamic values)
    fn is_fixed_expression(&self, expr: &crate::parser::ast::LSTerm) -> bool {
        match expr {
            crate::parser::ast::LSTerm::Literal(_) => true,
            crate::parser::ast::LSTerm::AppTerm(func, args) => {
                if let sygus_parser::Identifier::Symbol(func_name) = func {
                    // Check if this is a fixed function call (like iv-len)
                    if self.is_fixed_function(func_name) {
                        args.iter().all(|arg| self.is_fixed_expression(arg))
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Check if a function is fixed (doesn't depend on dynamic values)
    fn is_fixed_function(&self, func_name: &str) -> bool {
        // Functions that can be evaluated statically
        matches!(func_name, "iv-len" | "toInt" | "charAt" | "size")
    }
    
    /// Translate fixed function calls to bit-vector logic
    /// This function evaluates fixed expressions for all semantic states and creates a constant bit-vector
    fn translate_fixed_function_call(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        _func_name: &str,
        args: &[crate::parser::ast::LSTerm],
        _bv_args: &[SyGuSTerm],
        cardinality_n: usize,
        _eval_call_counter: &mut usize,
    ) -> Result<SyGuSTerm, String> {
        // For fixed function calls, we need to evaluate them for all semantic states
        // and create a constant bit-vector
        
        // All function calls that are not basic logic operators are treated as fixed expressions
        self.evaluate_fixed_expression_for_all_states(object_def, args, cardinality_n)
    }
    
    /// Evaluate a fixed expression for all semantic states and create a constant bit-vector
    /// This function now uses the interpreter to evaluate expressions for each semantic state
    fn evaluate_fixed_expression_for_all_states(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        args: &[crate::parser::ast::LSTerm],
        cardinality_n: usize,
    ) -> Result<SyGuSTerm, String> {
        // Get all possible semantic state values
        let state_values = self.generate_all_semantic_state_values(object_def)?;
        
        if state_values.len() != cardinality_n {
            return Err(format!(
                "Generated {} state values but expected {} for cardinality",
                state_values.len(), cardinality_n
            ));
        }
        
        // Pre-create all let expressions to avoid borrow checker issues
        let mut let_expressions = Vec::new();
        for state_value in &state_values {
            // Clone the args to avoid borrow checker issues
            let args_clone = args.to_vec();
            let let_expr = self.create_let_expression_for_state_evaluation(&args_clone, state_value)?;
            let_expressions.push(let_expr);
        }
        
        // Get the interpreter
        let interpreter = self.interpreter.as_mut()
            .ok_or("No interpreter available for fixed expression evaluation")?;
        
        // Build the bit string by evaluating the expression for each state
        let mut bit_string = String::new();
        
        for let_expr in let_expressions {
            // Set the interpreter context for this object
            interpreter.current_object = Some(object_def.name.clone());
            
            // Evaluate the expression using the interpreter
            let result = interpreter.eval(&let_expr)?;
            
            // Convert the result to a bit
            let bit = match result {
                crate::lang::runtime::Value::Bool(true) => '1',
                crate::lang::runtime::Value::Bool(false) => '0',
                _ => {
                    debug!("Non-boolean result from fixed expression evaluation: {:?}", result);
                    '0' // Default to false for non-boolean results
                }
            };
            
            bit_string.push(bit);
        }
        
        debug!("Generated bit string: {}", bit_string);
        
        // Convert the bit string to a SyGuS bit-vector literal
        // Note: SyGuS uses #b prefix for binary literals
        Ok(SyGuSTerm::Identifier(Identifier::Symbol(
            format!("#b{}", bit_string)
        )))
    }
    
    /// Generate all possible semantic state values
    /// This function now takes object_def to correctly resolve generic types
    fn generate_all_semantic_state_values(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
    ) -> Result<Vec<crate::parser::ast::LSTerm>, String> {
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        // Get the class definition to understand generic parameters
        let class_def = self.find_class_for_object(object_def)?;
        
        // Create mapping from generic parameters to concrete types
        let mut type_mapping = std::collections::HashMap::new();
        for (i, type_param) in class_def.type_params.iter().enumerate() {
            if i < object_def.type_args.len() {
                let concrete_type = &object_def.type_args[i];
                type_mapping.insert(type_param.clone(), concrete_type.clone());
                debug!(
                    "Mapped generic parameter '{}' to concrete type '{}'",
                    type_param, concrete_type
                );
            } else {
                return Err(format!(
                    "Object '{}' has fewer type arguments ({}) than class '{}' has type parameters ({})",
                    object_def.name, object_def.type_args.len(), class_def.name, class_def.type_params.len()
                ));
            }
        }
        
        // Find semantic state parameters from the semantic function
        // We need to find the semantic function (eval) to understand what semantic state types are used
        let semantic_func = class_def.methods.iter()
            .find(|m| self.is_semantic_function(m, class_def).unwrap_or(None).is_some())
            .ok_or("Could not find semantic function in class definition")?;
        
        let scrutinee_param_name = self.is_semantic_function(semantic_func, class_def)?
            .ok_or("Function is not a semantic function")?;
        
        let semantic_state_params = self.get_semantic_state_params(semantic_func, &scrutinee_param_name)?;
        
        if semantic_state_params.is_empty() {
            return Err("No semantic state parameters found".to_string());
        }
        
        // Use the first semantic state parameter to determine the datatype
        let main_semantic_param = &semantic_state_params[0];
        let semantic_sort = &main_semantic_param.sort;
        
        debug!(
            "Generating semantic state values for parameter '{}' of type {}",
            main_semantic_param.name, semantic_sort
        );
        
        // Get the datatype name
        let datatype_name = if let sygus_parser::Sort::Simple(identifier) = semantic_sort {
            identifier.to_string()
        } else {
            return Err(format!(
                "Semantic state parameter '{}' has complex type {} - expected simple sort",
                main_semantic_param.name, semantic_sort
            ));
        };
        
        // Find the datatype definition
        let datatype = type_env.get_datatype(&datatype_name)
            .ok_or_else(|| format!("Datatype '{}' not found in type environment", datatype_name))?;
        
        debug!("Found datatype '{}' with {} constructors", datatype_name, datatype.constructors.len());
        
        // Generate all possible values for this datatype
        let mut all_values = Vec::new();
        
        for constructor in &datatype.constructors {
            debug!("Processing constructor '{}' with {} fields", constructor.name, constructor.fields.len());
            
            if constructor.fields.is_empty() {
                // No-argument constructor
                let term = crate::parser::ast::LSTerm::AppTerm(
                    sygus_parser::Identifier::Symbol(constructor.name.clone()),
                    vec![],
                );
                all_values.push(term);
            } else {
                // Constructor with arguments - we need to generate all combinations
                let field_combinations = self.generate_field_combinations(&constructor.fields, &type_mapping, type_env)?;
                
                for combination in field_combinations {
                    let term = crate::parser::ast::LSTerm::AppTerm(
                        sygus_parser::Identifier::Symbol(constructor.name.clone()),
                        combination,
                    );
                    all_values.push(term);
                }
            }
        }
        
        debug!("Generated {} semantic state values", all_values.len());
        Ok(all_values)
    }
    
    /// Generate all possible combinations of field values for a constructor
    fn generate_field_combinations(
        &self,
        fields: &[crate::lang::types::Field],
        type_mapping: &std::collections::HashMap<String, sygus_parser::Sort>,
        type_env: &crate::lang::types::TypeEnv,
    ) -> Result<Vec<Vec<crate::parser::ast::LSTerm>>, String> {
        if fields.is_empty() {
            return Ok(vec![vec![]]);
        }
        
        let field = &fields[0];
        let remaining_fields = &fields[1..];
        
        // Resolve the field type using the type mapping
        let resolved_type = self.resolve_generic_type(&field.sort, type_mapping)?;
        debug!("Resolved field '{}' type '{}' to '{}'", field.name, field.sort, resolved_type);
        
        // Get all possible values for this field type
        let field_values = self.get_all_values_for_type(&resolved_type, type_env)?;
        
        // Recursively generate combinations for remaining fields
        let remaining_combinations = self.generate_field_combinations(remaining_fields, type_mapping, type_env)?;
        
        // Combine current field values with remaining combinations
        let mut all_combinations = Vec::new();
        for field_value in field_values {
            for remaining_combo in &remaining_combinations {
                let mut combo = vec![field_value.clone()];
                combo.extend(remaining_combo.clone());
                all_combinations.push(combo);
            }
        }
        
        Ok(all_combinations)
    }
    
    /// Get all possible values for a given type
    fn get_all_values_for_type(
        &self,
        sort: &sygus_parser::Sort,
        type_env: &crate::lang::types::TypeEnv,
    ) -> Result<Vec<crate::parser::ast::LSTerm>, String> {
        match sort {
            sygus_parser::Sort::Simple(identifier) => {
                let type_name = identifier.to_string();
                
                // Check if it's an enum
                if let Some(enum_def) = type_env.get_enum(&type_name) {
                    let mut values = Vec::new();
                    for element in &enum_def.elements {
                        values.push(crate::parser::ast::LSTerm::Identifier(
                            sygus_parser::Identifier::Symbol(element.clone())
                        ));
                    }
                    Ok(values)
                } else {
                    // For other simple types, we might need to handle them differently
                    // For now, return an empty vector
                    debug!("Type '{}' is not an enum, returning empty value set", type_name);
                    Ok(vec![])
                }
            }
            sygus_parser::Sort::Parameterized(_, _) => {
                // For parameterized types, we would need more complex logic
                // For now, return an empty vector
                debug!("Parameterized type '{}' not fully supported yet", sort);
                Ok(vec![])
            }
        }
    }
    
    /// Create a let expression to bind the semantic state variable for evaluation
    fn create_let_expression_for_state_evaluation(
        &self,
        args: &[crate::parser::ast::LSTerm],
        state_value: &crate::parser::ast::LSTerm,
    ) -> Result<crate::parser::ast::LSTerm, String> {
        // For now, we'll assume the first argument is the semantic state variable
        // In a more sophisticated implementation, we would need to analyze the expression
        // to determine which variable should be bound
        
        if args.is_empty() {
            return Err("No arguments provided for state evaluation".to_string());
        }
        
        // Create a binding for the semantic state variable
        // For now, we'll assume it's the first argument and it's a variable reference
        let state_var = if let crate::parser::ast::LSTerm::Identifier(ident) = &args[0] {
            ident.clone()
        } else {
            return Err("First argument is not a variable identifier".to_string());
        };
        
        // Create the let expression: (let ((state_var state_value)) original_expression)
        let binding = crate::parser::ast::LSVarBinding {
            name: state_var.to_string(),
            term: state_value.clone(),
        };
        
        // For now, we'll create a simple expression that uses the state variable
        // In a full implementation, you would reconstruct the original expression
        let body = crate::parser::ast::LSTerm::Identifier(state_var);
        
        Ok(crate::parser::ast::LSTerm::LetTerm {
            args: vec![binding],
            body: Box::new(body),
        })
    }
    
    /// Evaluate a fixed comparison
    fn evaluate_fixed_comparison(
        &self,
        left: &crate::parser::ast::LSTerm,
        right: &crate::parser::ast::LSTerm,
    ) -> Result<bool, String> {
        // For complex expressions, we need to evaluate them using the interpreter
        if self.is_fixed_expression(left) && self.is_fixed_expression(right) {
            // Both sides are fixed expressions - evaluate them
            let left_value = self.evaluate_fixed_term(left)?;
            let right_value = self.evaluate_fixed_term(right)?;
            
            // Compare the values
            match (left_value, right_value) {
                (crate::lang::runtime::Value::Bool(l), crate::lang::runtime::Value::Bool(r)) => Ok(l == r),
                (crate::lang::runtime::Value::Int(l), crate::lang::runtime::Value::Int(r)) => Ok(l == r),
                (crate::lang::runtime::Value::String(l), crate::lang::runtime::Value::String(r)) => Ok(l == r),
                (crate::lang::runtime::Value::Symbol(l), crate::lang::runtime::Value::Symbol(r)) => Ok(l == r),
                _ => Ok(false), // Different types are not equal
            }
        } else {
            // At least one side is not fixed - return false as a safe default
            Ok(false)
        }
    }
    
    /// Evaluate a fixed term using the interpreter
    fn evaluate_fixed_term(&self, _term: &crate::parser::ast::LSTerm) -> Result<crate::lang::runtime::Value, String> {
        // For now, we'll use a simplified approach since we can't mutate the interpreter
        // In a full implementation, you would need to clone the interpreter or use a different approach
        
        // Return a placeholder value
        Ok(crate::lang::runtime::Value::Bool(false))
    }

    /// Generate the top-level recursive evaluator (dynamic version)
    /// This function generates a SyGuS match expression that translates pattern matching
    /// on the scrutinee datatype into calls to bit-vector helper functions
    pub fn generate_object_evaluator_dynamic(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
        func_def: &FunctionType,
        class_def: &crate::parser::ast::ClassDef,
        _constructor_names: &[String],
        cardinality_n: usize,
    ) -> Result<SyGuSCmd, String> {
        let func_name = format!("{}_eval", object_def.name);
        let bv_sort = Sort::Simple(Identifier::Symbol(
            format!("(_ BitVec {})", cardinality_n)
        ));
        
        // Get the scrutinee parameter - we need to find the actual scrutinee name first
        let scrutinee_param_name = self.is_semantic_function(func_def, class_def)?
            .ok_or("Function is not a semantic function")?;
        let (scrutinee_name, scrutinee_sort) = self.get_semantic_scrutinee(func_def, &scrutinee_param_name)?;
        
        // Parameters: (phi RE) - where phi is the scrutinee
        let sorted_vars = vec![SortedVar {
            name: scrutinee_name.clone(),
            sort: scrutinee_sort.clone(),
        }];
        
        // Get the datatype definition to access constructor information
        let type_env = self.context.type_environment.as_ref()
            .ok_or("Type environment not available")?;
        
        let datatype_name = scrutinee_sort.clone().to_string();
        let datatype = type_env.get_datatype(&datatype_name)
            .ok_or_else(|| format!("Datatype {} not found in type environment", datatype_name))?;
        
        debug!("Generating evaluator for datatype '{}' with {} constructors", datatype_name, datatype.constructors.len());
        
        // Generate the match expression body
        let match_body = self.generate_sygus_match_expression(
            &scrutinee_name,
            &datatype,
            object_def,
            &func_name,
            &scrutinee_sort,
        )?;
        
        Ok(SyGuSCmd::DefineFun(
            func_name,
            sorted_vars,
            bv_sort,
            match_body,
        ))
    }
    
    /// Generate a SyGuS match expression for pattern matching on a datatype
    fn generate_sygus_match_expression(
        &self,
        scrutinee_var: &str,
        datatype: &crate::lang::types::DataType,
        object_def: &crate::parser::ast::ObjectDef,
        eval_func_name: &str,
        scrutinee_sort: &Sort,
    ) -> Result<SyGuSTerm, String> {
        // For SyGuS, we need to use nested ite expressions since match is not directly supported
        // We'll build a nested if-then-else structure
        
        if datatype.constructors.is_empty() {
            return Err("Datatype has no constructors".to_string());
        }
        
        // Start with the last constructor as the base case
        let mut current_body = self.generate_constructor_case(
            &datatype.constructors[datatype.constructors.len() - 1],
            scrutinee_var,
            object_def,
            eval_func_name,
            &scrutinee_sort, // Pass the scrutinee sort
        )?;
        
        // Build nested ite expressions from the second-to-last constructor backwards
        for i in (0..datatype.constructors.len() - 1).rev() {
            let constructor = &datatype.constructors[i];
            let then_body = self.generate_constructor_case(
                constructor,
                scrutinee_var,
                object_def,
                eval_func_name,
                &scrutinee_sort, // Pass the scrutinee sort
            )?;
            
            // Create the test condition: ((_ is constructor_name) scrutinee_var)
            let test_condition = SyGuSTerm::Application(
                Identifier::Symbol(format!("(_ is {})", constructor.name)),
                vec![SyGuSTerm::Identifier(Identifier::Symbol(scrutinee_var.to_string()))],
            );
            
            current_body = SyGuSTerm::Application(
                Identifier::Symbol("ite".to_string()),
                vec![test_condition, then_body, current_body],
            );
        }
        
        Ok(current_body)
    }
    
    /// Generate a case for a specific constructor in the match expression
    fn generate_constructor_case(
        &self,
        constructor: &crate::lang::types::Constructor,
        scrutinee_var: &str,
        object_def: &crate::parser::ast::ObjectDef,
        eval_func_name: &str,
        scrutinee_sort: &Sort, // Add scrutinee sort parameter
    ) -> Result<SyGuSTerm, String> {
        let bv_helper_name = format!("{}_{}", object_def.name, constructor.name);
        
        if constructor.fields.is_empty() {
            // No-argument constructor (e.g., re.empty, re.eps)
            Ok(SyGuSTerm::Application(
                Identifier::Symbol(bv_helper_name),
                    vec![],
            ))
            } else {
            // Constructor with arguments (e.g., re.union, re.++, re.*, re.sym)
            let mut bv_args = Vec::new();
            
            for (_i, field) in constructor.fields.iter().enumerate() {
                // Generate field access: (constructor_name.field_name scrutinee_var)
                let field_access = SyGuSTerm::Application(
                    Identifier::Symbol(format!("{}.{}", constructor.name, field.name)),
                    vec![SyGuSTerm::Identifier(Identifier::Symbol(scrutinee_var.to_string()))],
                );
                
                // Check if this field type is the same as the scrutinee type (recursive case)
                if &field.sort == scrutinee_sort {
                    // This is a recursive field - apply recursive eval call
                    debug!("Field '{}' is recursive (type: {}), applying eval", field.name, field.sort);
                    let recursive_call = SyGuSTerm::Application(
                        Identifier::Symbol(eval_func_name.to_string()),
                        vec![field_access],
                    );
                    bv_args.push(recursive_call);
        } else {
                    // This is a non-recursive field - pass the field access directly
                    debug!("Field '{}' is non-recursive (type: {}), passing directly", field.name, field.sort);
                    bv_args.push(field_access);
                }
            }
            
            Ok(SyGuSTerm::Application(
                Identifier::Symbol(bv_helper_name),
                bv_args,
            ))
        }
    }

    /// Translate a simple object method (non-semantic)
    pub fn translate_simple_object_method(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
        method_impl: &FunctionType,
    ) -> Result<SyGuSCmd, String> {
        let unknown_str = "unknown".to_string();
        let method_name = method_impl.get_name().unwrap_or(&unknown_str);
        let flattened_func_name = format!("{}_{}", object_def.name, method_name);
        
        // Get function type information from the method implementation
        let param_names = method_impl.get_param_names();
        let param_sorts = method_impl.get_params();
        let ret_sort = method_impl.get_ret();
        
        // For object methods, we need to remove the 'self' parameter
        let (instance_param_names, instance_param_sorts) = 
            if !param_names.is_empty() && param_names[0] == "self" {
                (param_names[1..].to_vec(), param_sorts[1..].to_vec())
            } else {
                (param_names.to_vec(), param_sorts.to_vec())
            };
        
        let sorted_vars: Vec<SortedVar> = instance_param_names
            .iter()
            .zip(instance_param_sorts.iter())
            .map(|(name, sort)| SortedVar {
                name: name.clone(),
                sort: sort.clone(),
            })
            .collect();
        
        // Translate the function body
        let body = if let Some(body_term) = method_impl.get_body() {
            // Replace self references with object-specific field access
            self.translate_term_with_self_replacement(body_term, &object_def.name)?
        } else {
            // No body - this might be a stub or abstract method
            SyGuSTerm::Identifier(Identifier::Symbol("__match_placeholder__".to_string()))
        };
        
        Ok(SyGuSCmd::DefineFun(
            flattened_func_name,
            sorted_vars,
            ret_sort.clone(),
            body,
        ))
    }

    /// Get function type from class method definition
    pub fn get_function_type_from_class_method<'a>(
        &self,
        method_def: &'a FunctionType,
    ) -> Option<&'a FunctionType> {
        Some(method_def)
    }

    /// Generate the top-level models function for an object
    pub fn generate_models_function(
        &self,
        object_def: &crate::parser::ast::ObjectDef,
        _cardinality_n: usize,
    ) -> Result<SyGuSCmd, String> {
        let func_name = format!("{}_models", object_def.name);
        
        // Parameters: (phi RE)
        let sorted_vars = vec![SortedVar {
            name: "phi".to_string(),
            sort: Sort::Simple(Identifier::Symbol("RE".to_string())),
        }];
        
        // Body: extract the bit corresponding to the full interval
        // For now, extract the first bit (index 0)
        let body = SyGuSTerm::Application(
            Identifier::Symbol("extract".to_string()),
            vec![
                SyGuSTerm::Identifier(
                    Identifier::Symbol("0".to_string())
                ),
                SyGuSTerm::Identifier(
                    Identifier::Symbol("0".to_string())
                ),
                SyGuSTerm::Application(
                    Identifier::Symbol(format!("{}_eval", object_def.name)),
                    vec![SyGuSTerm::Identifier(
                        Identifier::Symbol("phi".to_string())
                    )],
                ),
            ],
        );
        
        Ok(SyGuSCmd::DefineFun(
            func_name,
            sorted_vars,
            Sort::Simple(Identifier::Symbol("Bool".to_string())),
            body,
        ))
    }

    /// Parse a quantifier binding like ((cut Pos))
    fn parse_quantifier_binding(
        &self,
        binding_list: &crate::parser::ast::LSTerm,
    ) -> Result<(String, sygus_parser::Sort), String> {
        // Parse ((cut Pos)) structure
        if let crate::parser::ast::LSTerm::AppTerm(_, bindings) = binding_list {
            if bindings.len() == 1 {
                let binding = &bindings[0];
                if let crate::parser::ast::LSTerm::AppTerm(_, binding_parts) = binding {
                    if binding_parts.len() == 2 {
                        // First part should be the variable name
                        let var_name = if let crate::parser::ast::LSTerm::Identifier(ident) = &binding_parts[0] {
                            if let sygus_parser::Identifier::Symbol(name) = ident {
                                name.clone()
                            } else {
                                return Err("Variable name in quantifier binding must be a symbol".to_string());
                            }
                        } else {
                            return Err("First part of quantifier binding must be a variable name".to_string());
                        };
                        
                        // Second part should be the type
                        let var_type = if let crate::parser::ast::LSTerm::Identifier(ident) = &binding_parts[1] {
                            if let sygus_parser::Identifier::Symbol(_) = ident {
                                sygus_parser::Sort::Simple(ident.clone())
                            } else {
                                return Err("Type in quantifier binding must be a symbol".to_string());
                            }
                        } else {
                            return Err("Second part of quantifier binding must be a type".to_string());
                        };
                        
                        Ok((var_name, var_type))
                    } else {
                        Err("Quantifier binding must have exactly 2 parts: variable name and type".to_string())
                    }
                } else {
                    Err("Quantifier binding must be an application term".to_string())
                }
            } else {
                Err("Quantifier binding list must have exactly 1 binding".to_string())
            }
        } else {
            Err("Quantifier binding list must be an application term".to_string())
        }
    }
    
    /// Resolve a type using object context (for generic type parameters)
    fn resolve_type_with_object_context(
        &self,
        var_type: sygus_parser::Sort,
        object_def: &crate::parser::ast::ObjectDef,
    ) -> Result<sygus_parser::Sort, String> {
        // Get the class definition to understand generic parameters
        let class_def = self.find_class_for_object(object_def)?;
        
        // Create mapping from generic parameters to concrete types
        let mut type_mapping = std::collections::HashMap::new();
        for (i, type_param) in class_def.type_params.iter().enumerate() {
            if i < object_def.type_args.len() {
                let concrete_type = &object_def.type_args[i];
                type_mapping.insert(type_param.clone(), concrete_type.clone());
            }
        }
        
        // Resolve the type using the mapping
        self.resolve_generic_type(&var_type, &type_mapping)
    }
    
    /// Substitute a variable with a value in a term
    fn substitute_variable_in_term(
        &self,
        term: &crate::parser::ast::LSTerm,
        var_name: &str,
        value: &crate::parser::ast::LSTerm,
    ) -> Result<crate::parser::ast::LSTerm, String> {
        match term {
            crate::parser::ast::LSTerm::Identifier(ident) => {
                if let sygus_parser::Identifier::Symbol(name) = ident {
                    if name == var_name {
                        Ok(value.clone())
                    } else {
                        Ok(term.clone())
                    }
                } else {
                    Ok(term.clone())
                }
            }
            crate::parser::ast::LSTerm::AppTerm(func, args) => {
                let substituted_args: Result<Vec<crate::parser::ast::LSTerm>, String> = args
                    .iter()
                    .map(|arg| self.substitute_variable_in_term(arg, var_name, value))
                    .collect();
                
                Ok(crate::parser::ast::LSTerm::AppTerm(
                    func.clone(),
                    substituted_args?,
                ))
            }
            crate::parser::ast::LSTerm::MatchTerm { term: match_term, cases } => {
                let substituted_match_term = self.substitute_variable_in_term(match_term, var_name, value)?;
                let substituted_cases: Result<Vec<crate::lang::model::MatchCase>, String> = cases
                    .iter()
                    .map(|case| {
                        let substituted_value = self.substitute_variable_in_term(&case.value, var_name, value)?;
                        Ok(crate::lang::model::MatchCase {
                            pattern: case.pattern.clone(),
                            value: substituted_value,
                        })
                    })
                    .collect();
                
                Ok(crate::parser::ast::LSTerm::MatchTerm {
                    term: Box::new(substituted_match_term),
                    cases: substituted_cases?,
                })
            }
            crate::parser::ast::LSTerm::LetTerm { args, body } => {
                let substituted_args: Result<Vec<crate::parser::ast::LSVarBinding>, String> = args
                    .iter()
                    .map(|binding| {
                        let substituted_term = self.substitute_variable_in_term(&binding.term, var_name, value)?;
                        Ok(crate::parser::ast::LSVarBinding {
                            name: binding.name.clone(),
                            term: substituted_term,
                        })
                    })
                    .collect();
                
                let substituted_body = self.substitute_variable_in_term(body, var_name, value)?;
                
                Ok(crate::parser::ast::LSTerm::LetTerm {
                    args: substituted_args?,
                    body: Box::new(substituted_body),
                })
            }
            crate::parser::ast::LSTerm::Literal(_) => Ok(term.clone()),
            _ => Ok(term.clone()), // For other term types, just clone
        }
    }
} 