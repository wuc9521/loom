use crate::lang::types::{SymbolMap, TypeInferer};
use crate::parser::{LSCmd, ast::LSFile};
use log::{debug, info};
use std::collections::HashMap;
use derive_more::Display;
use sygus_parser::{Identifier, Sort, SyGuSCmd};

/// Semantic analysis result containing all symbol information
#[derive(Debug, Clone, Display)]
#[display(fmt = "{}", "type_env.to_string()")]
pub struct SemanticAnalysisResult {
    /// Complete type environment with all symbol information
    pub type_env: SymbolMap,
    /// Mapping from enum values to their concrete representations
    pub enum_value_map: HashMap<String, String>,
    /// Mapping from class names to their templates
    pub class_templates: HashMap<String, ClassTemplate>,
    /// Mapping from datatype names to their templates
    pub datatype_templates: HashMap<String, DatatypeTemplate>,
    /// All uninterpreted sorts that need to be concretized
    pub uninterpreted_sorts: HashMap<String, Sort>,
}

/// Template for class definitions with type parameters
#[derive(Debug, Clone)]
pub struct ClassTemplate {
    pub name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<crate::lang::types::FunctionType>,
}

/// Template for datatype definitions that depend on uninterpreted sorts
#[derive(Debug, Clone)]
pub struct DatatypeTemplate {
    pub name: String,
    pub constructors: sygus_parser::DTDecl,
    pub dependent_sorts: Vec<String>, // Which sorts this template depends on
}

/// Semantic analyzer that performs context-sensitive analysis
pub struct Analyzer {
    type_inferer: TypeInferer,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            type_inferer: TypeInferer::new(),
        }
    }

    /// Perform complete semantic analysis on the AST
    pub fn analyze(&mut self, ls_file: &LSFile) -> Result<SemanticAnalysisResult, String> {
        info!("Starting semantic analysis...");
        
        let mut result = SemanticAnalysisResult {
            type_env: SymbolMap::new(),
            enum_value_map: HashMap::new(),
            class_templates: HashMap::new(),
            datatype_templates: HashMap::new(),
            uninterpreted_sorts: HashMap::new(),
        };

        // First pass: collect all declarations and build symbol table
        self.collect_declarations(ls_file, &mut result)?;
        
        // Second pass: perform type checking and inference
        self.type_check_cmds(ls_file, &mut result)?;
        
        // Third pass: build enum mappings
        self.build_enum_mappings(ls_file, &mut result)?;
        
        // Fourth pass: identify templates
        self.identify_templates(ls_file, &mut result)?;

        info!("Semantic analysis completed successfully");
        Ok(result)
    }

    /// First pass: collect all declarations and build symbol table
    fn collect_declarations(
        &mut self,
        ls_file: &LSFile,
        result: &mut SemanticAnalysisResult,
    ) -> Result<(), String> {
        info!("Collecting declarations...");
        
        for cmd in &ls_file.cmds {
            match cmd {
                LSCmd::As(SyGuSCmd::DeclareSort(sort_name, arity)) => {
                    debug!("Found uninterpreted sort: {} (arity: {})", sort_name, arity);
                    result.uninterpreted_sorts.insert(
                        sort_name.clone(),
                        Sort::Simple(Identifier::Symbol(sort_name.clone())),
                    );
                }
                
                LSCmd::As(SyGuSCmd::DeclareDatatypes(sort_decls, dt_decls)) => {
                    for (i, dt_decl) in dt_decls.iter().enumerate() {
                        let sort_decl = &sort_decls[i];
                        let name = &sort_decl.symbol;
                        
                        // Check if this datatype depends on uninterpreted sorts
                        let mut dependent_sorts = Vec::new();
                        for constructor in dt_decl {
                            for field in &constructor.args {
                                if let Sort::Simple(ident) = &field.sort {
                                    let sort_name = ident.get_name();
                                    if result.uninterpreted_sorts.contains_key(sort_name) {
                                        dependent_sorts.push(sort_name.to_string());
                                    }
                                }
                            }
                        }
                        
                        if !dependent_sorts.is_empty() {
                            // This is a template
                            let template = DatatypeTemplate {
                                name: name.clone(),
                                constructors: dt_decl.clone(),
                                dependent_sorts: dependent_sorts.clone(),
                            };
                            result.datatype_templates.insert(name.clone(), template);
                            debug!("Added datatype template: {} (depends on: {:?})", name, dependent_sorts);
                        } else {
                            // This is a concrete datatype - add to type environment
                            // TODO: Add concrete datatype to type environment
                            debug!("Found concrete datatype: {}", name);
                        }
                    }
                }
                
                LSCmd::DefineSortEnum { name, elems } => {
                    debug!("Found enum sort: {} with elements: {:?}", name, elems);
                    // Enum sorts will be processed in the third pass
                }
                
                LSCmd::DefineClass(class_def) => {
                    debug!("Found class definition: {} with type params: {:?}", 
                           class_def.name, class_def.type_params);
                    
                    let template = ClassTemplate {
                        name: class_def.name.clone(),
                        type_params: class_def.type_params.clone(),
                        methods: class_def.methods.clone(),
                    };
                    result.class_templates.insert(class_def.name.clone(), template);
                }
                
                LSCmd::DefineObj(object_def) => {
                    debug!("Found object definition: {} of class {}", 
                           object_def.name, object_def.class_name);
                    // Object definitions will be processed in the second pass
                }
                
                _ => {
                    // Other commands will be processed in the second pass
                }
            }
        }
        
        Ok(())
    }

    /// Second pass: perform type checking and inference
    fn type_check_cmds(
        &mut self,
        ls_file: &LSFile,
        _result: &mut SemanticAnalysisResult,
    ) -> Result<(), String> {
        info!("Performing type checking...");
        
        for cmd in &ls_file.cmds {
            match cmd {
                LSCmd::As(SyGuSCmd::DefineFun(func_name, _params, _ret_type, _body)) => {
                    debug!("Type checking function: {}", func_name);
                    // TODO: Perform type checking on function definition
                }
                
                LSCmd::As(SyGuSCmd::Constraint(_term)) => {
                    debug!("Type checking constraint");
                    // TODO: Perform type checking on constraint
                }
                
                LSCmd::DefineObj(object_def) => {
                    debug!("Type checking object: {}", object_def.name);
                    // TODO: Perform type checking on object definition
                }
                
                _ => {
                    // Skip other commands for now
                }
            }
        }
        
        Ok(())
    }

    /// Third pass: build enum value mappings
    fn build_enum_mappings(
        &mut self,
        ls_file: &LSFile,
        result: &mut SemanticAnalysisResult,
    ) -> Result<(), String> {
        info!("Building enum value mappings...");
        
        for cmd in &ls_file.cmds {
            if let LSCmd::DefineSortEnum { name, elems } = cmd {
                debug!("Processing enum: {} with {} elements", name, elems.len());
                
                for (_i, elem) in elems.iter().enumerate() {
                    let enum_value = format!("${}", elem);
                    let function_name = format!("{}@{}", name, elem);
                    
                    result.enum_value_map.insert(enum_value.clone(), function_name.clone());
                    debug!("Mapped enum value: {} -> {}", enum_value, function_name);
                }
            }
        }
        
        Ok(())
    }

    /// Fourth pass: identify templates and their dependencies
    fn identify_templates(
        &mut self,
        _ls_file: &LSFile,
        _result: &mut SemanticAnalysisResult,
    ) -> Result<(), String> {
        info!("Identifying templates and dependencies...");
        
        // This pass is already partially done in collect_declarations
        // Here we can add additional template analysis if needed
        
        Ok(())
    }

    /// Get the type environment from the analyzer
    pub fn get_type_env(&self) -> &SymbolMap {
        self.type_inferer.global_scope()
    }

    /// Get a mutable reference to the type environment
    pub fn get_type_env_mut(&mut self) -> &mut SymbolMap {
        self.type_inferer.global_scope_mut()
    }
} 