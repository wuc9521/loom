use crate::lang::types::TypeEnv;
use crate::parser::ast::{LSCmd, LSFile, LSTerm, ObjectDef};
use crate::translator::utils::log2_ceil;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use sygus_parser::api::Cmd;
use sygus_parser::api::Term;
use sygus_parser::{
    DTConsDecl, DTDecl, Identifier, Sort, SortDecl, SortedVar, SyGuSCmd, SyGuSTerm, VarBinding,
};

/// Desugaring pass that transforms high-level LS constructs into simple SyGuS constructs
pub struct Desugarer {
    /// Original AST
    pub original_file: LSFile,
    /// Type environment from interpreter
    pub type_env: Option<TypeEnv>,
    /// Runtime context from interpreter
    pub runtime_context: Option<HashMap<String, crate::lang::runtime::Value>>,
    /// Desugaring context with all mapping relationships
    pub context: DesugarContext,
    /// Desugared commands
    pub desugared_commands: Vec<SyGuSCmd>,
}

/// Template for datatype definitions that depend on uninterpreted sorts
#[derive(Debug, Clone)]
pub struct DatatypeTemplate {
    pub name: String,
    pub constructors: DTDecl,
    pub dependent_sorts: HashSet<String>, // Which sorts this template depends on
}

/// Template for class definitions
#[derive(Debug, Clone)]
pub struct ClassTemplate {
    pub name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<crate::lang::types::FunctionType>,
}

/// Context for the desugaring process, storing all mapping relationships
#[derive(Debug, Clone)]
pub struct DesugarContext {
    /// Sort mappings: old_type_name -> new_sort_definition
    pub sort_map: HashMap<String, sygus_parser::Sort>,
    /// Enum value mappings: enum_value -> new_function_name
    pub enum_val_map: HashMap<String, String>,
    /// Object field mappings: (object_name, field_name) -> flattened_function_name
    pub object_fields: HashMap<(String, String), String>,
    /// Object method mappings: (object_name, method_name) -> flattened_function_name
    pub object_methods: HashMap<(String, String), String>,
    /// Class method mappings: (class_name, method_name) -> template_function_name
    pub class_methods: HashMap<(String, String), String>,
    /// Used symbols to avoid conflicts
    pub used_symbols: HashSet<String>,
    /// Templates for datatype definitions that depend on uninterpreted sorts
    pub datatype_templates: HashMap<String, DatatypeTemplate>,
    /// Templates for class definitions
    pub class_templates: HashMap<String, ClassTemplate>,
    /// Uninterpreted sorts that haven't been concretized yet
    pub uninterpreted_sorts: HashSet<String>,
}

impl DesugarContext {
    pub fn new() -> Self {
        Self {
            sort_map: HashMap::new(),
            enum_val_map: HashMap::new(),
            object_fields: HashMap::new(),
            object_methods: HashMap::new(),
            class_methods: HashMap::new(),
            used_symbols: HashSet::new(),
            datatype_templates: HashMap::new(),
            class_templates: HashMap::new(),
            uninterpreted_sorts: HashSet::new(),
        }
    }

    /// Map an old type name to a new sort definition
    pub fn map_sort(&mut self, old_type: &str, new_sort: sygus_parser::Sort) {
        let sort_clone = new_sort.clone();
        self.sort_map.insert(old_type.to_string(), new_sort);
        debug!("Mapped sort: {} -> {:?}", old_type, sort_clone);
    }

    pub fn map_enum_value(&mut self, enum_value: &str, new_function: &str) {
        self.enum_val_map
            .insert(enum_value.to_string(), new_function.to_string());
        debug!("Mapped enum value: {} -> {}", enum_value, new_function);
    }

    /// Add a datatype template for lazy instantiation
    pub fn add_datatype_template(&mut self, template: DatatypeTemplate) {
        debug!(
            "Added datatype template: {} (depends on: {:?})",
            template.name, template.dependent_sorts
        );
        self.datatype_templates
            .insert(template.name.clone(), template);
    }

    /// Add a class template
    pub fn add_class_template(&mut self, template: ClassTemplate) {
        debug!(
            "Added class template: {} with type params: {:?}",
            template.name, template.type_params
        );
        self.class_templates.insert(template.name.clone(), template);
    }

    /// Mark a sort as uninterpreted
    pub fn add_uninterpreted_sort(&mut self, sort_name: &str) {
        self.uninterpreted_sorts.insert(sort_name.to_string());
        debug!("Added uninterpreted sort: {}", sort_name);
    }

    /// Check if a sort is uninterpreted
    pub fn is_uninterpreted_sort(&self, sort_name: &str) -> bool {
        self.uninterpreted_sorts.contains(sort_name)
    }

    /// Map an object method to a flattened function name (using underscores)
    pub fn map_object_method(&mut self, object_name: &str, method_name: &str) -> String {
        let key = (object_name.to_string(), method_name.to_string());
        if let Some(mapped) = self.object_methods.get(&key) {
            return mapped.clone();
        }

        let flattened_name = format!("{}@{}", object_name, method_name);
        self.object_methods.insert(key, flattened_name.clone());
        self.used_symbols.insert(flattened_name.clone());

        debug!(
            "Mapped object method: {}.{} -> {}",
            object_name, method_name, flattened_name
        );
        flattened_name
    }

    /// Map a class method to a template function name (using underscores)
    pub fn map_class_method(&mut self, class_name: &str, method_name: &str) -> String {
        let key = (class_name.to_string(), method_name.to_string());
        if let Some(mapped) = self.class_methods.get(&key) {
            return mapped.clone();
        }

        let template_name = format!("{}_template_{}", class_name, method_name);
        self.class_methods.insert(key, template_name.clone());
        self.used_symbols.insert(template_name.clone());

        debug!(
            "Mapped class method: {}.{} -> {}",
            class_name, method_name, template_name
        );
        template_name
    }
}

impl Desugarer {
    pub fn new(
        original_file: LSFile,
        type_env: Option<TypeEnv>,
        runtime_context: Option<HashMap<String, crate::lang::runtime::Value>>,
    ) -> Self {
        Self {
            original_file,
            type_env,
            runtime_context,
            context: DesugarContext::new(),
            desugared_commands: Vec::new(),
        }
    }

    /// Main desugaring method that transforms the entire AST
    pub fn desugar_ast(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        info!("Starting unified desugaring pass...");
        self.build_mapping_ctx()?;

        let commands_to_process = self.original_file.cmds.clone();
        for cmd in &commands_to_process {
            let desugared_cmds = self.desugar_cmd_rec(cmd)?;
            self.desugared_commands.extend(desugared_cmds);
        }

        info!(
            "Unified desugaring pass completed. Generated {} commands.",
            self.desugared_commands.len()
        );
        Ok(self.desugared_commands.clone())
    }

    fn build_mapping_ctx(&mut self) -> Result<(), String> {
        info!("Building complete mapping context...");

        let cmds = &self.original_file.cmds;

        // First pass: identify uninterpreted sorts and templates
        for cmd in cmds {
            match cmd {
                LSCmd::As(SyGuSCmd::DeclareSort(sort_name, _)) => {
                    self.context.add_uninterpreted_sort(sort_name);
                }
                LSCmd::As(SyGuSCmd::DeclareDatatypes(sort_decls, dt_decls)) => {
                    for (i, dt_decl) in dt_decls.iter().enumerate() {
                        let sort_decl = &sort_decls[i];
                        let name = &sort_decl.symbol;

                        // Check if this datatype depends on uninterpreted sorts
                        let mut dependent_sorts = HashSet::new();
                        for constructor in dt_decl {
                            for field in &constructor.args {
                                if let Sort::Simple(ident) = &field.sort {
                                    let sort_name = ident.get_name();
                                    if self.context.is_uninterpreted_sort(sort_name) {
                                        dependent_sorts.insert(sort_name.to_string());
                                    }
                                }
                            }
                        }

                        if !dependent_sorts.is_empty() {
                            // This is a template - don't generate immediately
                            let template = DatatypeTemplate {
                                name: name.clone(),
                                constructors: dt_decl.clone(),
                                dependent_sorts,
                            };
                            self.context.add_datatype_template(template);
                        }
                    }
                }
                _ => {}
            }
        }

        // Second pass: build enum mappings and class/object templates
        for cmd in cmds {
            match cmd {
                LSCmd::DefineSortEnum { name, elems } => {
                    let bit_width = log2_ceil(elems.len());
                    self.context.map_sort(name, Sort::bitvec(bit_width));

                    for (i, elem) in elems.iter().enumerate() {
                        let func_name = format!("{}@{}", name, elem);
                        self.context.map_enum_value(elem, &func_name);

                        debug!(
                            "Mapped enum value: {} -> {} (bit-vector: {})",
                            elem, func_name, i
                        );
                    }
                }

                LSCmd::ClassDef(class_def) => {
                    // Store class as template
                    let template = ClassTemplate {
                        name: class_def.name.clone(),
                        type_params: class_def.type_params.clone(),
                        methods: class_def.methods.clone(),
                    };
                    self.context.add_class_template(template);

                    for method in &class_def.methods {
                        if let Some(method_name) = method.get_name() {
                            self.context.map_class_method(&class_def.name, method_name);
                            debug!("Mapped class method: {}.{}", class_def.name, method_name);
                        }
                    }
                }

                LSCmd::ObjectDef(object_def) => {
                    for method in &object_def.methods {
                        if let Some(method_name) = method.get_name() {
                            self.context
                                .map_object_method(&object_def.name, method_name);
                            debug!("Mapped object method: {}.{}", object_def.name, method_name);
                        }
                    }
                }

                _ => {}
            }
        }

        info!("Mapping context built successfully");
        Ok(())
    }

    fn desugar_cmd_rec(&mut self, cmd: &LSCmd) -> Result<Vec<SyGuSCmd>, String> {
        match cmd {
            // Step 1: Filter out declare-sort commands (templates)
            LSCmd::As(SyGuSCmd::DeclareSort(_, _)) => {
                // This is a template - do not add to output
                debug!("Filtering out declare-sort template: {:?}", cmd);
                Ok(vec![])
            }

            // Step 1: Filter out declare-datatypes that depend on uninterpreted sorts (templates)
            LSCmd::As(SyGuSCmd::DeclareDatatypes(sort_decls, dt_decls)) => {
                let mut commands = Vec::new();

                for (i, dt_decl) in dt_decls.iter().enumerate() {
                    let sort_decl = &sort_decls[i];
                    let name = &sort_decl.symbol;

                    // Check if this datatype depends on uninterpreted sorts
                    let mut has_dependencies = false;
                    for constructor in dt_decl {
                        for field in &constructor.args {
                            if let Sort::Simple(ident) = &field.sort {
                                let sort_name = ident.get_name();
                                if self.context.is_uninterpreted_sort(sort_name) {
                                    has_dependencies = true;
                                    break;
                                }
                            }
                        }
                        if has_dependencies {
                            break;
                        }
                    }

                    if has_dependencies {
                        // This is a template - do not add to output
                        debug!("Filtering out declare-datatype template: {}", name);
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

            // Handle class definitions - create method templates
            LSCmd::ClassDef(class_def) => {
                let commands = Vec::new();

                // For now, we just register the methods in the context
                // The actual method instantiation happens when processing object definitions
                for method in &class_def.methods {
                    if let Some(method_name) = method.get_name() {
                        debug!(
                            "Registered class method template: {}.{}",
                            class_def.name, method_name
                        );
                    }
                }

                Ok(commands)
            }

            // Handle object definitions - create flattened functions with isolated context
            LSCmd::ObjectDef(object_def) => {
                let mut commands = Vec::new();

                // Create an isolated context for this object
                let mut object_context = self.context.clone();

                // Step 2: Extract concrete type from object definition
                let concrete_types = self.type_params_of_object(object_def)?;
                for (i, concrete_type) in concrete_types.iter().enumerate() {
                    if let Some(mapped_sort) = self.context.sort_map.get(concrete_type) {
                        let class_type_param = self
                            .context
                            .class_templates
                            .get(&object_def.class_name)
                            .unwrap()
                            .type_params[i]
                            .clone();
                        object_context
                            .sort_map
                            .insert(class_type_param, mapped_sort.clone());
                    }
                    let instantiated_datatypes =
                        self.inst_dt_templ(&object_def.name, &concrete_type, &mut object_context)?;
                    commands.extend(instantiated_datatypes);
                }

                // Generate object methods with the isolated context
                for method in &object_def.methods {
                    if let Some(_) = method.get_name() {
                        let flattened_func = self.create_flattened_method_function(
                            object_def,
                            method,
                            &mut object_context,
                        )?;
                        commands.push(flattened_func);
                    }
                }

                Ok(commands)
            }

            // Handle standard SyGuS commands with recursive term transformation
            LSCmd::As(sygus_cmd) => {
                let desugared_cmd = self.desugar_sygus_cmd_rec(sygus_cmd)?;
                Ok(vec![desugared_cmd])
            }

            // Handle other LS commands
            _ => {
                debug!("Skipping unsupported command: {:?}", cmd);
                Ok(vec![])
            }
        }
    }

    /// Recursively desugar a SyGuS command with full context awareness
    fn desugar_sygus_cmd_rec(&mut self, cmd: &SyGuSCmd) -> Result<SyGuSCmd, String> {
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
                debug!("Desugaring constraint term: {:?}", term);
                let desugared_term = self.desugar_sygus_term_rec(term)?;
                debug!("Desugared constraint term: {:?}", desugared_term);
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
                if let Some(mapped_sort) = self.context.sort_map.get(ident.get_name()) {
                    debug!("Mapped sort: {} -> {:?}", ident.get_name(), mapped_sort);
                    Ok(mapped_sort.clone())
                } else {
                    debug!(
                        "No mapping found for sort: {}, keeping original",
                        ident.get_name()
                    );
                    Ok(sort.clone())
                }
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
    fn desugar_sygus_term_rec(&mut self, term: &SyGuSTerm) -> Result<SyGuSTerm, String> {
        match term {
            SyGuSTerm::Identifier(ident) => {
                // Check if this is a mapped enum value
                if let Some(new_function) = self.context.enum_val_map.get(ident.get_name()) {
                    debug!(
                        "Mapped enum value: {} -> {}",
                        ident.get_name(),
                        new_function
                    );
                    // Replace enum value with function call
                    Ok(Term::var(new_function.as_str()))
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
                debug!("Desugaring application: {}", func);
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

                Ok(SyGuSTerm::Exists(desugared_vars?, Box::new(desugared_body)))
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

                Ok(SyGuSTerm::Forall(desugared_vars?, Box::new(desugared_body)))
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

    /// Instantiate datatype templates for a specific object
    fn inst_dt_templ(
        &self,
        object_name: &str,
        concrete_type: &str,
        object_context: &mut DesugarContext,
    ) -> Result<Vec<SyGuSCmd>, String> {
        let mut commands = Vec::new();

        for (template_name, template) in &self.context.datatype_templates {
            // Check if this template depends on the type being concretized
            if template.dependent_sorts.contains("Pos") {
                // Assuming Pos is the generic type
                // Create a concrete instance of this datatype
                let concrete_datatype_name = format!("{}@{}", template_name, object_name);

                // Update the object context to map the template name to the concrete name
                object_context.sort_map.insert(
                    template_name.clone(),
                    Sort::Simple(Identifier::Symbol(concrete_datatype_name.clone())),
                );

                // Create the concrete datatype definition
                let mut concrete_constructors = Vec::new();
                for constructor in &template.constructors {
                    let mut concrete_fields = Vec::new();
                    for field in &constructor.args {
                        let concrete_sort = if let Sort::Simple(ident) = &field.sort {
                            if ident.get_name() == "Pos" {
                                // Replace with concrete type
                                if let Some(mapped_sort) =
                                    object_context.sort_map.get(concrete_type)
                                {
                                    mapped_sort.clone()
                                } else {
                                    Sort::Simple(Identifier::Symbol(concrete_type.to_string()))
                                }
                            } else {
                                field.sort.clone()
                            }
                        } else {
                            field.sort.clone()
                        };

                        concrete_fields.push(sygus_parser::SortedVar {
                            name: field.name.clone(),
                            sort: concrete_sort,
                        });
                    }

                    concrete_constructors.push(DTConsDecl {
                        name: constructor.name.clone(),
                        args: concrete_fields,
                    });
                }

                // Create the declare-datatypes command
                let sort_decl = SortDecl {
                    symbol: concrete_datatype_name.clone(),
                    arity: 0,
                };
                let datatype_cmd =
                    SyGuSCmd::DeclareDatatypes(vec![sort_decl], vec![concrete_constructors]);
                commands.push(datatype_cmd);

                debug!(
                    "Instantiated datatype template {} as {} for object {}",
                    template_name, template_name, object_name
                );
            }
        }

        Ok(commands)
    }

    /// Create a flattened method function with self eliminated and specific context
    fn create_flattened_method_function(
        &mut self,
        object_def: &crate::parser::ast::ObjectDef,
        method: &crate::lang::types::FunctionType,
        object_context: &mut DesugarContext,
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
                let resolved_sort = self.desugar_sort(sort)?;
                Ok(SortedVar {
                    name,
                    sort: resolved_sort,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Create function body with self references replaced
        if let Some(body) = method.get_body() {
            let body_term =
                self.eliminate_self_references(body, &object_def.name, object_context)?;

            // Desugar the function definition completely with object context
            let desugared_func =
                self.desugar_fun_def(&func_name, &sorted_vars, &method.get_ret(), &body_term)?;

            Ok(desugared_func)
        } else {
            Err(format!("Method {} has no body", method_name))
        }
    }

    /// Eliminate self references and desugar terms with specific context
    fn eliminate_self_references(
        &mut self,
        term: &LSTerm,
        object_name: &str,
        object_context: &DesugarContext,
    ) -> Result<SyGuSTerm, String> {
        self.desugar_term(term, object_name, object_context)
    }

    /// Desugar a term with context-aware transformations
    fn desugar_term(
        &mut self,
        term: &LSTerm,
        object_name: &str,
        context: &DesugarContext,
    ) -> Result<SyGuSTerm, String> {
        match term {
            LSTerm::Identifier(identifier) => {
                let name = identifier.get_name();

                // Check if this is a self reference
                if name == "self" {
                    return Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                        object_name.to_string(),
                    )));
                }

                // Check if this is an enum value that needs to be converted to function call
                if let Some(new_function) = context.enum_val_map.get(name) {
                    // Convert enum value to function call (with no arguments)
                    return Ok(Term::var(new_function));
                }

                // Check if this is a variable that maps to an enum constant function
                // For variables like 'a' and 'b' that reference enum values
                if let Some(runtime_ctx) = &self.runtime_context {
                    if let Some(_value) = runtime_ctx.get(name) {
                        // If the value is an enum value, convert it to function call
                        if let Some(enum_func) = context.enum_val_map.get(&format!("${}", name)) {
                            return Ok(Term::var(enum_func));
                        }
                    }
                }

                // Keep other identifiers as-is
                Ok(SyGuSTerm::Identifier(identifier.clone()))
            }
            LSTerm::AppTerm(func, args) => {
                // Recursively process arguments with context
                let processed_args: Result<Vec<SyGuSTerm>, String> = args
                    .iter()
                    .map(|arg| self.desugar_term(arg, object_name, context))
                    .collect();

                Ok(SyGuSTerm::Application(func.clone(), processed_args?))
            }
            LSTerm::MatchTerm {
                term: scrutinee,
                cases,
            } => {
                // Desugar match expression into nested ite expressions with context
                self.desugar_match_expr(scrutinee, cases, object_name, context)
            }
            LSTerm::Literal(literal) => {
                // Convert literal to SyGuSTerm
                Ok(SyGuSTerm::Literal(literal.clone()))
            }
            _ => {
                // For other terms, convert to SyGuSTerm (simplified)
                Ok(SyGuSTerm::Identifier(Identifier::Symbol(
                    "placeholder".to_string(),
                )))
            }
        }
    }

    /// Desugar match expression into nested ite expressions with context
    fn desugar_match_expr(
        &mut self,
        scrutinee: &LSTerm,
        cases: &[crate::lang::model::MatchCase],
        object_name: &str,
        context: &DesugarContext,
    ) -> Result<SyGuSTerm, String> {
        if cases.is_empty() {
            return Err("Match expression has no cases".to_string());
        }

        // Desugar the scrutinee with context
        let scrutinee_term = self.desugar_term(scrutinee, object_name, context)?;

        // Build nested ite expressions
        let mut result =
            self.desugar_match_case(&cases[0], &scrutinee_term, object_name, context)?;

        for case in &cases[1..] {
            let case_term = self.desugar_match_case(case, &scrutinee_term, object_name, context)?;

            // Create ite expression: (ite condition then_branch else_branch)
            result = Term::ite(
                self.create_equality_test(&scrutinee_term, &case.pattern)?,
                case_term,
                result,
            );
        }

        Ok(result)
    }

    /// Desugar a single match case with context
    fn desugar_match_case(
        &mut self,
        case: &crate::lang::model::MatchCase,
        _: &SyGuSTerm,
        object_name: &str,
        context: &DesugarContext,
    ) -> Result<SyGuSTerm, String> {
        // Desugar the case value with context
        self.desugar_term(&case.value, object_name, context)
    }

    /// Desugar a function definition completely
    fn desugar_fun_def(
        &mut self,
        func_name: &str,
        params: &[SortedVar],
        ret_type: &sygus_parser::Sort,
        body: &SyGuSTerm,
    ) -> Result<SyGuSCmd, String> {
        // Desugar parameter types
        let desugared_params: Vec<SortedVar> = params
            .iter()
            .map(|param| {
                let desugared_sort = self.desugar_sort(&param.sort)?;
                Ok(SortedVar {
                    name: param.name.clone(),
                    sort: desugared_sort,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Desugar return type
        let desugared_ret_type = self.desugar_sort(ret_type)?;

        // Desugar function body (recursively replace enum values)
        let desugared_body = self.desugar_sygus_term(body)?;

        Ok(SyGuSCmd::DefineFun(
            func_name.to_string(),
            desugared_params,
            desugared_ret_type,
            desugared_body,
        ))
    }

    /// Desugar a sort, replacing old type names with new sort definitions
    fn desugar_sort(&self, sort: &sygus_parser::Sort) -> Result<sygus_parser::Sort, String> {
        match sort {
            sygus_parser::Sort::Simple(identifier) => {
                if let Identifier::Symbol(name) = identifier {
                    if let Some(new_sort) = self.context.sort_map.get(name) {
                        Ok(new_sort.clone())
                    } else {
                        Ok(sort.clone())
                    }
                } else {
                    Ok(sort.clone())
                }
            }
            _ => Ok(sort.clone()),
        }
    }

    /// Desugar a SyGuSTerm, replacing enum values with function calls
    fn desugar_sygus_term(&self, term: &SyGuSTerm) -> Result<SyGuSTerm, String> {
        match term {
            SyGuSTerm::Identifier(identifier) => {
                if let Identifier::Symbol(name) = identifier {
                    if let Some(new_function) = self.context.enum_val_map.get(name) {
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
                    .map(|arg| self.desugar_sygus_term(arg))
                    .collect();

                Ok(SyGuSTerm::Application(func.clone(), desugared_args?))
            }
            _ => Ok(term.clone()),
        }
    }

    /// Extract concrete type from object definition
    /// For objects like "w1: Model<Posw1>", extract "Posw1" from type_args
    fn type_params_of_object(&self, object_def: &ObjectDef) -> Result<Vec<String>, String> {
        if !object_def.type_args.is_empty() {
            for arg in &object_def.type_args {
                if let Sort::Simple(ident) = arg {
                    let concrete_type = ident.get_name().to_string();
                    return Ok(vec![concrete_type]);
                }
            }
        }
        debug!(
            "Could not extract concrete type from object {} (no type args or complex type)",
            object_def.name
        );
        Ok(vec![])
    }

    /// Create equality test for match pattern
    fn create_equality_test(
        &self,
        scrutinee: &SyGuSTerm,
        pattern: &Option<crate::lang::model::Pattern>,
    ) -> Result<SyGuSTerm, String> {
        match pattern {
            Some(pattern) => {
                // Handle different pattern types based on the Pattern structure
                if pattern.elements.is_empty() {
                    // Empty pattern, always match
                    Ok(SyGuSTerm::Literal(sygus_parser::Literal::Bool(true)))
                } else if pattern.elements.len() == 1 {
                    // Single element pattern
                    match &pattern.elements[0] {
                        crate::lang::model::PatternElem::SortElem(elem) => {
                            // Enum element pattern, check if it's an enum value we can map
                            let enum_value = format!("${}", elem.name);
                            if let Some(new_function) = self.context.enum_val_map.get(&enum_value) {
                                // Create function call to the new enum function
                                let func_call = SyGuSTerm::Application(
                                    Identifier::Symbol(new_function.clone()),
                                    vec![],
                                );
                                Ok(SyGuSTerm::Application(
                                    Identifier::Symbol("=".to_string()),
                                    vec![scrutinee.clone(), func_call],
                                ))
                            } else {
                                // Fallback to identifier comparison
                                let pattern_term =
                                    SyGuSTerm::Identifier(Identifier::Symbol(elem.name.clone()));
                                Ok(SyGuSTerm::Application(
                                    Identifier::Symbol("=".to_string()),
                                    vec![scrutinee.clone(), pattern_term],
                                ))
                            }
                        }
                        crate::lang::model::PatternElem::Var(_) => {
                            // Variable pattern, always match (wildcard)
                            Ok(SyGuSTerm::Literal(sygus_parser::Literal::Bool(true)))
                        }
                        crate::lang::model::PatternElem::Wildcard => {
                            // Wildcard pattern, always match
                            Ok(SyGuSTerm::Literal(sygus_parser::Literal::Bool(true)))
                        }
                    }
                } else {
                    // Multiple element pattern (constructor pattern)
                    // For now, use the first element as the constructor name
                    if let Some(crate::lang::model::PatternElem::SortElem(elem)) =
                        pattern.elements.first()
                    {
                        let pattern_term =
                            SyGuSTerm::Identifier(Identifier::Symbol(elem.name.clone()));
                        Ok(SyGuSTerm::Application(
                            Identifier::Symbol("=".to_string()),
                            vec![scrutinee.clone(), pattern_term],
                        ))
                    } else {
                        // Fallback for complex patterns
                        Ok(SyGuSTerm::Literal(sygus_parser::Literal::Bool(true)))
                    }
                }
            }
            None => {
                // No pattern (default case), always match
                Ok(SyGuSTerm::Literal(sygus_parser::Literal::Bool(true)))
            }
        }
    }
}
