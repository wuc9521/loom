use crate::parser::{LSCmd, ast::LSFile, LSTerm, ObjectDef, MatchCase, Pattern, PatternElem};
use crate::translator::semantic::SemanticAnalysisResult;
use crate::translator::utils::log2_ceil;
use log::{debug, info};
use std::collections::HashMap;
use sygus_parser::{Identifier, Sort, SortedVar, SyGuSCmd, SyGuSTerm, VarBinding};
use sygus_parser::api::{Cmd, Term};


/// Context-aware desugarer that transforms high-level LS constructs into simple SyGuS constructs
pub struct Desugarer {
    /// Original AST
    pub original_file: LSFile,
    /// Semantic analysis result containing all context information
    pub semantic_result: SemanticAnalysisResult,
    /// Desugared commands
    pub desugared_commands: Vec<SyGuSCmd>,
}

impl Desugarer {
    pub fn new(original_file: LSFile, semantic_result: SemanticAnalysisResult) -> Self {
        println!("Semantic result: {:?}", semantic_result);
        Self {
            original_file,
            semantic_result,
            desugared_commands: Vec::new(),
        }
    }

    /// Main desugaring method that transforms the entire AST with full context awareness
    pub fn desugar_ast(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        info!("Starting context-aware desugaring pass...");

        let commands_to_process = self.original_file.cmds.clone();
        for cmd in &commands_to_process {
            let desugared_cmds = self.desugar_cmd_with_context(cmd)?;
            self.desugared_commands.extend(desugared_cmds);
        }

        info!(
            "Context-aware desugaring pass completed. Generated {} commands.",
            self.desugared_commands.len()
        );
        Ok(self.desugared_commands.clone())
    }

    /// Desugar a command with full context awareness
    fn desugar_cmd_with_context(&mut self, cmd: &LSCmd) -> Result<Vec<SyGuSCmd>, String> {
        match cmd {
            // Handle uninterpreted sort declarations (templates)
            LSCmd::As(SyGuSCmd::DeclareSort(_, _)) => {
                debug!("Filtering out declare-sort template: {:?}", cmd);
                Ok(vec![])
            }

            // Handle datatype declarations with context awareness
            LSCmd::As(SyGuSCmd::DeclareDatatypes(sort_decls, dt_decls)) => {
                let mut commands = Vec::new();

                for (i, dt_decl) in dt_decls.iter().enumerate() {
                    let sort_decl = &sort_decls[i];
                    let name = &sort_decl.symbol;

                    // Check if this is a template that depends on uninterpreted sorts
                    if self.semantic_result.datatype_templates.contains_key(name) {
                        debug!("Filtering out datatype template: {}", name);
                    } else {
                        // This is a concrete datatype - add to output
                        let concrete_sort_decl = vec![sort_decl.clone()];
                        let concrete_dt_decl = vec![dt_decl.clone()];
                        commands.push(SyGuSCmd::DeclareDatatypes(
                            concrete_sort_decl,
                            concrete_dt_decl,
                        ));
                    }
                }

                Ok(commands)
            }

            // Handle enum definitions with context awareness
            LSCmd::DefineSortEnum { name, elems } => {
                let mut commands = Vec::new();
                let bit_width = log2_ceil(elems.len());

                // Create the bit-vector sort definition
                let define_sort = Cmd::define_sort(name.as_str(), Sort::bitvec(bit_width)).build();
                commands.push(define_sort);

                // Create function definitions for each enum value
                for (i, elem) in elems.iter().enumerate() {
                    let func_name = format!("{}@{}", name, elem);
                    let bv_literal = Term::bconst(i as u64, bit_width);

                    let define_fun = SyGuSCmd::DefineFun(
                        func_name,
                        vec![], // No parameters
                        Sort::bitvec(bit_width),
                        bv_literal,
                    );
                    commands.push(define_fun);
                }

                Ok(commands)
            }

            // Handle class definitions - store as templates (no output)
            LSCmd::DefineClass(class_def) => {
                debug!("Processing class definition: {}", class_def.name);
                // Classes are templates - no immediate output
                Ok(vec![])
            }

            // Handle object definitions with isolated context and template instantiation
            LSCmd::DefineObj(object_def) => {
                let mut commands = Vec::new();

                // Create an isolated context for this object
                let mut object_context = self.create_object_context(object_def)?;

                // Instantiate datatype templates for this object
                let instantiated_datatypes = self.instantiate_datatype_templates(
                    object_def,
                    &mut object_context,
                )?;
                commands.extend(instantiated_datatypes);

                // Generate object methods with the isolated context
                for method in &object_def.methods {
                    if let Some(_) = method.get_name() {
                        let flattened_func = self.create_flattened_method_function(
                            object_def,
                            method,
                            &object_context,
                        )?;
                        commands.push(flattened_func);
                    }
                }

                Ok(commands)
            }

            // Handle standard SyGuS commands with recursive term transformation
            LSCmd::As(sygus_cmd) => {
                let desugared_cmd = self.desugar_sygus_cmd_with_context(sygus_cmd)?;
                Ok(vec![desugared_cmd])
            }

            // Handle other LS commands
            _ => {
                debug!("Skipping unsupported command: {:?}", cmd);
                Ok(vec![])
            }
        }
    }

    /// Create an isolated context for object processing
    fn create_object_context(&self, object_def: &ObjectDef) -> Result<ObjectContext, String> {
        let mut context = ObjectContext::new();

        // Extract concrete types from object definition
        let concrete_types = self.extract_concrete_types(object_def)?;
        
        // Get class template
        let class_template = self
            .semantic_result
            .class_templates
            .get(&object_def.class_name)
            .ok_or_else(|| {
                format!("Class template not found for class: {}", object_def.class_name)
            })?;

        // Bind type parameters to concrete types
        for (i, concrete_type) in concrete_types.iter().enumerate() {
            if i < class_template.type_params.len() {
                let type_param = &class_template.type_params[i];
                let concrete_sort = self.resolve_concrete_type(concrete_type)?;
                context.type_bindings.insert(type_param.clone(), concrete_sort);
            }
        }

        // Copy enum value mappings
        context.enum_value_map = self.semantic_result.enum_value_map.clone();

        Ok(context)
    }

    /// Extract concrete types from object definition
    fn extract_concrete_types(&self, object_def: &ObjectDef) -> Result<Vec<String>, String> {
        let mut concrete_types = Vec::new();
        
        for arg in &object_def.type_args {
            if let Sort::Simple(ident) = arg {
                concrete_types.push(ident.get_name().to_string());
            }
        }
        
        Ok(concrete_types)
    }

    /// Resolve a concrete type name to its actual sort
    fn resolve_concrete_type(&self, type_name: &str) -> Result<Sort, String> {
        // Check if it's an enum type
        if let Some(enum_func) = self.semantic_result.enum_value_map.get(&format!("${}", type_name)) {
            // Extract the enum name from the function name (e.g., "Color@red" -> "Color")
            if let Some(enum_name) = enum_func.split('@').next() {
                // Find the enum definition to get the bit width
                for cmd in &self.original_file.cmds {
                    if let LSCmd::DefineSortEnum { name, elems } = cmd {
                        if name == enum_name {
                            let bit_width = log2_ceil(elems.len());
                            return Ok(Sort::bitvec(bit_width));
                        }
                    }
                }
            }
        }
        
        // For now, assume it's a bit vector of width 1 (Pos type)
        Ok(Sort::bitvec(1))
    }

    /// Instantiate datatype templates for a specific object
    fn instantiate_datatype_templates(
        &self,
        object_def: &ObjectDef,
        object_context: &ObjectContext,
    ) -> Result<Vec<SyGuSCmd>, String> {
        let mut commands = Vec::new();

        for (template_name, template) in &self.semantic_result.datatype_templates {
            // Check if this template depends on types that are bound in the object context
            let mut should_instantiate = false;
            for dependent_sort in &template.dependent_sorts {
                if object_context.type_bindings.contains_key(dependent_sort) {
                    should_instantiate = true;
                    break;
                }
            }

            if should_instantiate {
                let instantiated_datatype = self.instantiate_single_datatype_template(
                    template_name,
                    template,
                    object_def,
                    object_context,
                )?;
                commands.push(instantiated_datatype);
            }
        }

        Ok(commands)
    }

    /// Instantiate a single datatype template
    fn instantiate_single_datatype_template(
        &self,
        template_name: &str,
        template: &crate::translator::semantic::DatatypeTemplate,
        object_def: &ObjectDef,
        _object_context: &ObjectContext,
    ) -> Result<SyGuSCmd, String> {
        // Create a new datatype name for this instance
        let instantiated_name = format!("{}@{}", template_name, object_def.name);
        
        // For now, return a simple datatype declaration
        // TODO: Implement proper template instantiation with type substitution
        let sort_decl = sygus_parser::SortDecl {
            symbol: instantiated_name,
            arity: 0, // TODO: Calculate proper arity
        };
        
        Ok(SyGuSCmd::DeclareDatatypes(
            vec![sort_decl],
            vec![template.constructors.clone()],
        ))
    }

    /// Create a flattened method function with self eliminated and specific context
    fn create_flattened_method_function(
        &self,
        object_def: &ObjectDef,
        method: &crate::lang::types::FunctionType,
        object_context: &ObjectContext,
    ) -> Result<SyGuSCmd, String> {
        let method_name = method.get_name().map(|s| s.as_str()).unwrap_or("unknown");
        let func_name = format!("{}@{}", object_def.name, method_name);

        // Remove self parameter and adjust other parameters
        let mut params = method.get_params().clone();
        let param_names = method.get_param_names();

        // Check if first parameter is "self" and remove it
        if !param_names.is_empty() && param_names[0] == "self" && !params.is_empty() {
            params.remove(0);
        }

        // Convert params to SortedVar format with context-aware sort resolution
        let sorted_vars: Vec<SortedVar> = params
            .iter()
            .enumerate()
            .map(|(i, sort)| {
                let name = if i < param_names.len() {
                    param_names[i].clone()
                } else {
                    format!("param{}", i)
                };
                let resolved_sort = self.resolve_sort_with_context(sort, object_context)?;
                Ok(SortedVar {
                    name,
                    sort: resolved_sort,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Create function body with self references replaced
        if let Some(body) = method.get_body() {
            let body_term = self.eliminate_self_references(body, &object_def.name, object_context)?;

            // Desugar the function definition completely with object context
            let desugared_func = self.desugar_fun_def_with_context(
                &func_name,
                &sorted_vars,
                &method.get_ret(),
                &body_term,
                object_context,
            )?;

            Ok(desugared_func)
        } else {
            Err(format!("Method {} has no body", method_name))
        }
    }

    /// Resolve a sort with context awareness
    fn resolve_sort_with_context(&self, sort: &Sort, context: &ObjectContext) -> Result<Sort, String> {
        match sort {
            Sort::Simple(ident) => {
                let name = ident.get_name();
                
                // Check if this is a type parameter that should be bound
                if let Some(bound_sort) = context.type_bindings.get(name) {
                    Ok(bound_sort.clone())
                } else {
                    // Check if this is an enum type
                    if let Some(enum_func) = self.semantic_result.enum_value_map.get(&format!("${}", name)) {
                        // Extract the enum name and get its bit width
                        if let Some(enum_name) = enum_func.split('@').next() {
                            for cmd in &self.original_file.cmds {
                                if let LSCmd::DefineSortEnum { name: enum_def_name, elems } = cmd {
                                    if enum_def_name == enum_name {
                                        let bit_width = log2_ceil(elems.len());
                                        return Ok(Sort::bitvec(bit_width));
                                    }
                                }
                            }
                        }
                    }
                    
                    // Default to bit vector of width 1
                    Ok(Sort::bitvec(1))
                }
            }
            Sort::Parameterized(ident, param_sorts) => {
                // Recursively resolve parameter sorts
                let resolved_params: Result<Vec<Sort>, String> = param_sorts
                    .iter()
                    .map(|s| self.resolve_sort_with_context(s, context))
                    .collect();

                Ok(Sort::Parameterized(ident.clone(), resolved_params?))
            }
        }
    }

    /// Eliminate self references and desugar terms with specific context
    fn eliminate_self_references(
        &self,
        term: &LSTerm,
        object_name: &str,
        object_context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        self.desugar_term_with_context(term, object_name, object_context)
    }

    /// Desugar a term with context-aware transformations
    fn desugar_term_with_context(
        &self,
        term: &LSTerm,
        object_name: &str,
        context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        match term {
            LSTerm::Identifier(identifier) => {
                let name = identifier.get_name();

                // Check if this is a self reference
                if name == "self" {
                    return Ok(Term::var(object_name));
                }

                // Check if this is an enum value that needs to be converted to function call
                if let Some(new_function) = context.enum_value_map.get(name) {
                    return Ok(Term::var(new_function));
                }

                // Check if this is a variable that maps to an enum constant function
                if let Some(enum_func) = context.enum_value_map.get(&format!("${}", name)) {
                    return Ok(Term::var(enum_func));
                }

                // Keep other identifiers as-is
                Ok(Term::var(name))
            }
            LSTerm::AppTerm(func, args) => {
                // Recursively process arguments with context
                let processed_args: Result<Vec<SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.desugar_term_with_context(arg, object_name, context))
                    .collect();

                Ok(SyGuSTerm::Application(func.clone(), processed_args?))
            }
            LSTerm::MatchTerm {
                term: scrutinee,
                cases,
            } => {
                // Desugar match expression into nested ite expressions with context
                self.desugar_match_expr_with_context(scrutinee, cases, object_name, context)
            }
            LSTerm::Literal(literal) => {
                // Convert literal to SyGuSTerm
                Ok(Term::lit(literal.clone()))
            }
            _ => {
                // For other terms, convert to SyGuSTerm (simplified)
                Ok(Term::var("__#placeholder__"))
            }
        }
    }

    /// Desugar match expression into nested ite expressions with context
    fn desugar_match_expr_with_context(
        &self,
        scrutinee: &LSTerm,
        cases: &[MatchCase],
        object_name: &str,
        context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        if cases.is_empty() {
            return Err("Match expression has no cases".to_string());
        }

        // Desugar the scrutinee with context
        let scrutinee_term = self.desugar_term_with_context(scrutinee, object_name, context)?;

        // Build nested ite expressions
        let mut result = self.desugar_match_case_with_context(
            &cases[0],
            &scrutinee_term,
            object_name,
            context,
        )?;

        for case in &cases[1..] {
            let case_term = self.desugar_match_case_with_context(
                case,
                &scrutinee_term,
                object_name,
                context,
            )?;

            // Create ite expression: (ite condition then_branch else_branch)
            result = Term::ite(
                self.desugar_eq_test_with_context(&scrutinee_term, &case.pattern, context)?,
                case_term,
                result,
            );
        }

        Ok(result)
    }

    /// Desugar a single match case with context
    fn desugar_match_case_with_context(
        &self,
        case: &MatchCase,
        _: &SyGuSTerm,
        object_name: &str,
        context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        // Desugar the case value with context
        self.desugar_term_with_context(&case.value, object_name, context)
    }

    /// Desugar a function definition completely with context
    fn desugar_fun_def_with_context(
        &self,
        func_name: &str,
        params: &[SortedVar],
        ret_type: &Sort,
        body: &SyGuSTerm,
        context: &ObjectContext,
    ) -> Result<SyGuSCmd, String> {
        // Desugar parameter types with context
        let desugared_params: Vec<SortedVar> = params
            .iter()
            .map(|param| {
                let desugared_sort = self.resolve_sort_with_context(&param.sort, context)?;
                Ok(SortedVar {
                    name: param.name.clone(),
                    sort: desugared_sort,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Desugar return type with context
        let desugared_ret_type = self.resolve_sort_with_context(ret_type, context)?;

        // Desugar function body (recursively replace enum values)
        let desugared_body = self.desugar_sygus_term_with_context(body, context)?;

        Ok(SyGuSCmd::DefineFun(
            func_name.to_string(),
            desugared_params,
            desugared_ret_type,
            desugared_body,
        ))
    }

    /// Desugar a SyGuSTerm with context awareness
    fn desugar_sygus_term_with_context(
        &self,
        term: &SyGuSTerm,
        context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        match term {
            SyGuSTerm::Identifier(identifier) => {
                if let Identifier::Symbol(name) = identifier {
                    if let Some(new_function) = context.enum_value_map.get(name) {
                        // Replace enum value with function call
                        Ok(Term::var(new_function.as_str()))
                    } else {
                        Ok(term.clone())
                    }
                } else {
                    Ok(term.clone())
                }
            }
            SyGuSTerm::Application(func, args) => {
                // Recursively desugar arguments
                let desugared_args: Result<Vec<SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.desugar_sygus_term_with_context(arg, context))
                    .collect();

                Ok(SyGuSTerm::Application(func.clone(), desugared_args?))
            }

            SyGuSTerm::Let(bindings, body) => {
                // Recursively transform bindings and body
                let desugared_bindings: Result<Vec<VarBinding>, String> = bindings
                    .iter()
                    .map(|binding| {
                        let desugared_term = self.desugar_sygus_term_rec(&binding.term)?;
                        Ok(VarBinding {
                            name: binding.name.clone(),
                            term: Box::new(desugared_term),
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(SyGuSTerm::Let(
                    desugared_bindings?,
                    Box::new(desugared_body),
                ))
            }

            SyGuSTerm::Exists(vars, body) => {
                // Recursively transform quantified variables and body
                let desugared_vars: Result<Vec<SortedVar>, String> = vars
                    .iter()
                    .map(|var| {
                        let desugared_sort = self.desugar_sort_rec(&var.sort)?;
                        Ok(SortedVar {
                            name: var.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(sygus_parser::api::Term::exists(desugared_vars?, desugared_body))
            }

            SyGuSTerm::Forall(vars, body) => {
                // Recursively transform quantified variables and body
                let desugared_vars: Result<Vec<SortedVar>, String> = vars
                    .iter()
                    .map(|var| {
                        let desugared_sort = self.desugar_sort_rec(&var.sort)?;
                        Ok(SortedVar {
                            name: var.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(sygus_parser::api::Term::forall(desugared_vars?, desugared_body))
            }

            SyGuSTerm::Annotated(term, attrs) => {
                // Recursively transform the annotated term
                let desugared_term = self.desugar_sygus_term_rec(term)?;
                Ok(SyGuSTerm::Annotated(
                    Box::new(desugared_term),
                    attrs.clone(),
                ))
            }

            _ => Ok(term.clone()),
        }
    }

    /// Recursively desugar a SyGuS command with full context awareness
    fn desugar_sygus_cmd_with_context(&self, cmd: &SyGuSCmd) -> Result<SyGuSCmd, String> {
        match cmd {
            SyGuSCmd::DefineFun(func_name, params, ret_type, body) => {
                // Recursively transform parameters, return type, and body
                let desugared_params: Result<Vec<SortedVar>, String> = params
                    .iter()
                    .map(|param| {
                        let desugared_sort = self.desugar_sort_rec(&param.sort)?;
                        Ok(SortedVar {
                            name: param.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_ret_type = self.desugar_sort_rec(ret_type)?;
                let desugared_body = self.desugar_sygus_term_rec(body)?;
                Ok(SyGuSCmd::DefineFun(
                    func_name.clone(),
                    desugared_params?,
                    desugared_ret_type,
                    desugared_body,
                ))
            }

            SyGuSCmd::Constraint(term) => {
                debug!("Desugaring constraint term: {}", term);
                let desugared_term = self.desugar_sygus_term_rec(term)?;
                debug!("Desugared constraint term: {}", desugared_term);
                Ok(SyGuSCmd::Constraint(desugared_term))
            }

            SyGuSCmd::SynthFun(func_name, params, ret_type, grammar) => {
                // Recursively transform parameters and return type
                let desugared_params: Result<Vec<SortedVar>, String> = params
                    .iter()
                    .map(|param| {
                        let desugared_sort = self.desugar_sort_rec(&param.sort)?;
                        Ok(SortedVar {
                            name: param.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_ret_type = self.desugar_sort_rec(ret_type)?;

                // For now, keep grammar as-is (could be enhanced later)
                Ok(SyGuSCmd::SynthFun(
                    func_name.clone(),
                    desugared_params?,
                    desugared_ret_type,
                    grammar.clone(),
                ))
            }

            // Pass through other commands as-is
            _ => Ok(cmd.clone()),
        }
    }

    /// Recursively desugar a sort with full context awareness
    fn desugar_sort_rec(&self, sort: &Sort) -> Result<Sort, String> {
        match sort {
            Sort::Simple(ident) => {
                // Check if this is a mapped enum sort
                if let Some(enum_func) = self.semantic_result.enum_value_map.get(&format!("${}", ident.get_name())) {
                    // Extract the enum name and get its bit width
                    if let Some(enum_name) = enum_func.split('@').next() {
                        for cmd in &self.original_file.cmds {
                            if let LSCmd::DefineSortEnum { name, elems } = cmd {
                                if name == enum_name {
                                    let bit_width = log2_ceil(elems.len());
                                    return Ok(Sort::bitvec(bit_width));
                                }
                            }
                        }
                    }
                }
                
                debug!(
                    "No mapping found for sort: {}, keeping original",
                    ident.get_name()
                );
                Ok(sort.clone())
            }
            Sort::Parameterized(ident, param_sorts) => {
                // Recursively transform parameter sorts
                let desugared_params: Result<Vec<Sort>, String> = param_sorts
                    .iter()
                    .map(|s| self.desugar_sort_rec(s))
                    .collect();

                Ok(Sort::Parameterized(ident.clone(), desugared_params?))
            }
        }
    }

    /// Recursively desugar a SyGuS term with full context awareness
    fn desugar_sygus_term_rec(&self, term: &SyGuSTerm) -> Result<SyGuSTerm, String> {
        match term {
            SyGuSTerm::Identifier(ident) => {
                // Check if this is a mapped enum value
                if let Some(new_function) = self.semantic_result.enum_value_map.get(ident.get_name()) {
                    debug!(
                        "Mapped enum value: {} -> {}",
                        ident.get_name(),
                        new_function
                    );
                    // Replace enum value with function call
                    Ok(sygus_parser::api::Term::var(new_function.as_str()))
                } else {
                    debug!(
                        "No mapping found for identifier: {}, keeping original",
                        ident.get_name()
                    );
                    Ok(term.clone())
                }
            }

            SyGuSTerm::Application(func, args) => {
                // Recursively transform arguments
                debug!(
                    "Desugaring application: {} with args: {}",
                    func,
                    args.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let desugared_args: Result<Vec<SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.desugar_sygus_term_rec(arg))
                    .collect();

                Ok(SyGuSTerm::Application(func.clone(), desugared_args?))
            }

            SyGuSTerm::Let(bindings, body) => {
                // Recursively transform bindings and body
                let desugared_bindings: Result<Vec<VarBinding>, String> = bindings
                    .iter()
                    .map(|binding| {
                        let desugared_term = self.desugar_sygus_term_rec(&binding.term)?;
                        Ok(VarBinding {
                            name: binding.name.clone(),
                            term: Box::new(desugared_term),
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(SyGuSTerm::Let(
                    desugared_bindings?,
                    Box::new(desugared_body),
                ))
            }

            SyGuSTerm::Exists(vars, body) => {
                // Recursively transform quantified variables and body
                let desugared_vars: Result<Vec<SortedVar>, String> = vars
                    .iter()
                    .map(|var| {
                        let desugared_sort = self.desugar_sort_rec(&var.sort)?;
                        Ok(SortedVar {
                            name: var.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(sygus_parser::api::Term::exists(desugared_vars?, desugared_body))
            }

            SyGuSTerm::Forall(vars, body) => {
                // Recursively transform quantified variables and body
                let desugared_vars: Result<Vec<SortedVar>, String> = vars
                    .iter()
                    .map(|var| {
                        let desugared_sort = self.desugar_sort_rec(&var.sort)?;
                        Ok(SortedVar {
                            name: var.name.clone(),
                            sort: desugared_sort,
                        })
                    })
                    .collect();

                let desugared_body = self.desugar_sygus_term_rec(body)?;

                Ok(sygus_parser::api::Term::forall(desugared_vars?, desugared_body))
            }

            SyGuSTerm::Annotated(term, attrs) => {
                // Recursively transform the annotated term
                let desugared_term = self.desugar_sygus_term_rec(term)?;
                Ok(SyGuSTerm::Annotated(
                    Box::new(desugared_term),
                    attrs.clone(),
                ))
            }

            _ => Ok(term.clone()),
        }
    }

    /// Desugar a match pattern into an equality test with context
    fn desugar_eq_test_with_context(
        &self,
        scrutinee: &SyGuSTerm,
        pattern: &Option<Pattern>,
        _context: &ObjectContext,
    ) -> Result<SyGuSTerm, String> {
        match pattern {
            Some(pattern) => {
                if pattern.elements.len() > 1 {
                    unimplemented!("Pattern with multiple elements: {}", pattern)
                } else {
                    let elem = pattern.elements[0].clone();
                    match elem {
                        PatternElem::SortElem(sort_elem) => Ok(Term::eq(
                            scrutinee.clone(),
                            Term::var(sort_elem.name.as_str()),
                        )),
                        PatternElem::Var(var) => {
                            Ok(Term::eq(scrutinee.clone(), Term::var(var.as_str())))
                        }
                        PatternElem::Wildcard => {
                            // If it's a wildcard, we don't need an equality test
                            Ok(Term::bool(true))
                        }
                    }
                }
            }
            None => {
                unimplemented!("No pattern (default case), always match")
            }
        }
    }
}

/// Isolated context for object processing
#[derive(Debug, Clone)]
struct ObjectContext {
    /// Type parameter bindings for this object
    pub type_bindings: HashMap<String, Sort>,
    /// Enum value mappings
    pub enum_value_map: HashMap<String, String>,
}

impl ObjectContext {
    fn new() -> Self {
        Self {
            type_bindings: HashMap::new(),
            enum_value_map: HashMap::new(),
        }
    }
}
