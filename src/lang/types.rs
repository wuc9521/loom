use crate::lang::model::MatchCase;
use crate::parser::ast::{LSTerm, LSVarBinding};
use crate::parser::{LSParseError, Rule};
use crate::ANY_TYPE;
use derive_more::Display;
use log::{debug, error};
use pest::iterators::Pair;
use std::collections::HashMap;
use sygus_parser::{Identifier, Sort, SortedVar};

/// finiteness
#[derive(Debug, Clone, PartialEq)]
pub enum Finiteness {
    Finite(usize), // finite, and give the cardinality
    Infinite,      // infinite
}

/// datatype field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub sort: Sort,
}

/// datatype constructor
#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<Field>, // list of fields
}

/// datatype definition
#[derive(Debug, Clone)]
pub struct DataType {
    pub name: String,
    pub constructors: Vec<Constructor>,
}

/// enum type definition
#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub elements: Vec<String>,
}

/// Class type definition
#[derive(Debug, Clone)]
pub struct ClassType {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, Sort)>,
    pub methods: HashMap<String, FunctionType>,
}

/// type environment
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub datatypes: HashMap<String, DataType>,
    pub enums: HashMap<String, EnumType>,
    pub variables: HashMap<String, Sort>,
    pub function_types: HashMap<String, FunctionType>,
    pub constructors: HashMap<String, ConstructorInfo>,
    pub classes: HashMap<String, ClassType>,
    pub objects: HashMap<String, (String, Vec<String>)>, // object name -> (class name, type args)
}

#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub name: String,
    pub datatype: String,
    pub field_types: Vec<Sort>,
}

#[derive(Debug, Clone, Display)]
#[display(
    fmt = "{} :: {} -> {}",
    "self.get_name().unwrap_or(&\"<#unknown>\".to_string())",
    "self.params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" -> \")",
    "self.ret.to_string()"
)]
pub struct FunctionType {
    pub name: Option<String>,
    pub param_names: Vec<String>,
    pub params: Vec<Sort>,
    pub ret: Sort,
    pub body: Option<LSTerm>,
    pub is_constructor: bool,
}

impl FunctionType {
    /// get function name
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// get parameter types
    pub fn get_params(&self) -> &[Sort] {
        &self.params
    }

    /// get parameter names
    pub fn get_param_names(&self) -> &[String] {
        &self.param_names
    }

    /// get return type
    pub fn get_ret(&self) -> &Sort {
        &self.ret
    }

    /// get function body
    pub fn get_body(&self) -> Option<&LSTerm> {
        self.body.as_ref()
    }

    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        // println!("pair: {:?}", pair.as_rule());
        let mut inner = pair.clone().into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let mut args = vec![];
        let mut ret_sort = None;

        // parse sorted variables
        while let Some(var_pair) = inner.next() {
            if var_pair.as_rule() == Rule::SortedVar {
                let sorted_var = sygus_parser::SortedVar::from_str(var_pair.as_str()).unwrap();
                args.push(sorted_var);
            } else if var_pair.as_rule() == Rule::Sort {
                let sort_str = var_pair.as_str().to_string();
                ret_sort = Some(sygus_parser::Sort::from_str(&sort_str).unwrap());
                break;
            } else {
                eprintln!("Unexpected token: {}", var_pair.as_str());
                break;
            }
        }
        debug!("inner: {:?}", inner.as_str());
        let body_pair = inner.peek().ok_or_else(|| {
            LSParseError::InvalidSyntax(format!(
                "Expected function body, got {} at {}",
                inner.as_str(),
                name
            ))
        })?;
        let body = LSTerm::parse(body_pair)?;

        Ok(FunctionType::new_definition(
            name,
            args.iter().map(|sv| sv.name.clone()).collect(),
            args.iter().map(|sv| sv.sort.clone()).collect(),
            ret_sort.unwrap(),
            body,
        ))
    }

    /// create function signature (only type information, no function body)
    pub fn new_signature(name: String, params: Vec<Sort>, ret: Sort) -> Self {
        Self {
            name: Some(name),
            param_names: vec![],
            params,
            ret,
            body: None,
            is_constructor: false,
        }
    }

    pub fn new_constructor(name: String, params: Vec<Sort>, ret: Sort) -> Self {
        Self {
            name: Some(name),
            param_names: vec![],
            params,
            ret,
            body: None,
            is_constructor: true,
        }
    }

    /// create complete function definition (with function body)
    pub fn new_definition(
        name: String,
        param_names: Vec<String>,
        params: Vec<Sort>,
        ret: Sort,
        body: LSTerm,
    ) -> Self {
        Self {
            name: Some(name),
            param_names,
            params,
            ret,
            body: Some(body),
            is_constructor: false,
        }
    }
}

/// adapter: convert sygus-parser's datatype to our type system
pub mod adapter {
    use super::*;
    use sygus_parser::SyGuSCmd;

    /// create DataType from sygus-parser's DTDecl
    pub fn from_dtdecl(name: &str, dtdecl: &sygus_parser::DTDecl) -> DataType {
        let constructors = dtdecl
            .iter()
            .map(|cons_decl| Constructor {
                name: cons_decl.name.clone(),
                fields: cons_decl
                    .args
                    .iter()
                    .map(|sv| Field {
                        name: sv.name.clone(),
                        sort: sv.sort.clone(),
                    })
                    .collect(),
            })
            .collect();

        DataType {
            name: name.to_string(),
            constructors,
        }
    }

    /// create DataType from sygus-parser's SyGuSCmd
    pub fn from_sygus_cmd(cmd: &SyGuSCmd) -> Option<(String, DataType)> {
        match cmd {
            SyGuSCmd::DeclareDatatype(name, dtdecl) => {
                Some((name.clone(), from_dtdecl(name, dtdecl)))
            }
            _ => None,
        }
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            datatypes: HashMap::new(),
            enums: HashMap::new(),
            variables: HashMap::new(),
            function_types: HashMap::new(),
            constructors: HashMap::new(),
            classes: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    /// check if the symbol already exists
    pub fn has_symbol(&self, name: &str) -> bool {
        self.datatypes.contains_key(name)
            || self.enums.contains_key(name)
            || self.variables.contains_key(name)
            || self.function_types.contains_key(name)
            || self.classes.contains_key(name)
            || self.objects.contains_key(name)
    }

    /// check if the datatype already exists
    pub fn has_datatype(&self, name: &str) -> bool {
        self.datatypes.contains_key(name)
    }

    /// check if the enum already exists
    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }

    /// check if the variable already exists
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// check if the function already exists
    pub fn has_function(&self, name: &str) -> bool {
        self.function_types.contains_key(name)
    }

    /// get the type of the symbol (if exists)
    pub fn get_symbol_type(&self, name: &str) -> Option<&str> {
        if self.has_datatype(name) {
            Some("datatype")
        } else if self.has_enum(name) {
            Some("enum")
        } else if self.has_variable(name) {
            Some("variable")
        } else if self.has_function(name) {
            Some("function")
        } else if self.is_class(name) {
            Some("class")
        } else if self.objects.contains_key(name) {
            Some("object")
        } else {
            None
        }
    }

    /// register datatype
    pub fn register_datatype(&mut self, name: String, data_type: DataType) {
        self.datatypes.insert(name.clone(), data_type.clone());

        // register constructor info
        for constructor in &data_type.constructors {
            let constructor_info = ConstructorInfo {
                name: constructor.name.clone(),
                datatype: name.clone(),
                field_types: constructor.fields.iter().map(|f| f.sort.clone()).collect(),
            };
            self.constructors
                .insert(constructor.name.clone(), constructor_info);

            // register constructor type information, but mark as constructor
            let param_sorts: Vec<Sort> =
                constructor.fields.iter().map(|f| f.sort.clone()).collect();
            let constructor_func = FunctionType::new_constructor(
                constructor.name.clone(),
                param_sorts,
                Sort::from_name(name.as_str()),
            );
            self.register_function_type(constructor.name.clone(), constructor_func);

            // register field accessor functions
            for field in &constructor.fields {
                let field_accessor = FunctionType::new_signature(
                    field.name.clone(),
                    vec![Sort::from_name(name.as_str())], // parameter type is the datatype itself
                    field.sort.clone(),                   // return type is the field type
                );
                self.register_function_type(field.name.clone(), field_accessor);
            }
        }
    }

    /// get constructor info
    pub fn get_constructor(&self, name: &str) -> Option<&ConstructorInfo> {
        self.constructors.get(name)
    }

    /// check if it is a constructor
    pub fn is_constructor(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }

    /// get data type by name
    pub fn get_datatype(&self, name: &str) -> Option<&DataType> {
        self.datatypes.get(name)
    }

    /// bind variable to sort
    pub fn bind_variable(&mut self, name: String, sort: Sort) {
        self.variables.insert(name, sort);
    }

    /// get variable type by name
    pub fn get_variable_type(&self, name: &str) -> Option<&Sort> {
        self.variables.get(name)
    }

    /// register a unified function type
    pub fn register_function_type(&mut self, name: String, func_type: FunctionType) {
        self.function_types.insert(name, func_type);
    }

    /// get unified function type by name
    pub fn get_function_type(&self, name: &str) -> Option<&FunctionType> {
        self.function_types.get(name)
    }

    /// register enum
    pub fn register_enum(&mut self, name: String, elements: Vec<String>) {
        let name_clone = name.clone();
        self.enums.insert(
            name.clone(),
            EnumType {
                name: name_clone,
                elements,
            },
        );

        // also bind the sort name to the enum type
        self.bind_variable(name.clone(), Sort::from_name(name.as_str()));
    }

    /// get enum type by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumType> {
        self.enums.get(name)
    }

    /// Register a class type
    pub fn register_class_type(
        &mut self,
        name: String,
        type_params: Vec<String>,
        fields: Vec<(String, Sort)>,
    ) {
        let class_type = ClassType {
            name: name.clone(),
            type_params,
            fields,
            methods: HashMap::new(),
        };
        self.classes.insert(name, class_type);
    }

    /// Add method to class
    pub fn add_method_to_class(
        &mut self,
        class_name: &str,
        method_name: String,
        method_type: FunctionType,
    ) -> Result<(), String> {
        if let Some(class) = self.classes.get_mut(class_name) {
            class.methods.insert(method_name, method_type);
            Ok(())
        } else {
            Err(format!("Class '{}' not found", class_name))
        }
    }

    /// Register an object instance
    pub fn register_object(&mut self, name: String, class_name: String, type_args: Vec<String>) {
        self.objects.insert(name, (class_name, type_args));
    }

    /// Get class type
    pub fn get_class(&self, name: &str) -> Option<&ClassType> {
        self.classes.get(name)
    }

    /// Check if a name is a class
    pub fn is_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// Get object info
    pub fn get_object_info(&self, name: &str) -> Option<&(String, Vec<String>)> {
        self.objects.get(name)
    }

    /// check if the type is finite
    pub fn is_finite(&self, sort: &Sort) -> Option<Finiteness> {
        match sort {
            // Bool is finite
            sort if sort == &Sort::bool() => Some(Finiteness::Finite(2)),

            // enum type is finite
            Sort::Simple(Identifier::Symbol(name)) => {
                if let Some(enum_type) = self.get_enum(name) {
                    Some(Finiteness::Finite(enum_type.elements.len()))
                } else {
                    None // other types are unknown
                }
            }

            _ => None,
        }
    }

    /// check if the type is finite by name
    pub fn is_finite_by_name(&self, name: &str) -> Option<bool> {
        // check if it is an enum type
        if let Some(_) = self.get_enum(name) {
            return Some(true); // enum type is always finite
        }

        // check if it is a datatype
        if let Some(datatype) = self.get_datatype(name) {
            return Some(!self.is_datatype_recursive(datatype)); // non-recursive datatype is finite
        }

        // check if it is a known finite type
        if name == "Bool" {
            return Some(true);
        }

        // check if it is a known infinite type
        if name == "Int" || name == "Real" || name == "String" {
            return Some(false);
        }

        // other types are infinite by default
        Some(false)
    }

    /// get the cardinality of the type (if finite)
    pub fn get_cardinality(&self, name: &str) -> Option<usize> {
        // check if it is an enum type
        if let Some(enum_type) = self.get_enum(name) {
            return Some(enum_type.elements.len());
        }

        // check if it is a known finite type
        if name == "Bool" {
            return Some(2);
        }

        // check if it is a datatype
        if let Some(datatype) = self.get_datatype(name) {
            if !self.is_datatype_recursive(datatype) {
                return self.calculate_datatype_cardinality(datatype);
            }
        }

        None // infinite or unknown type
    }

    /// calculate the cardinality of a non-recursive datatype
    fn calculate_datatype_cardinality(&self, datatype: &DataType) -> Option<usize> {
        let mut total = 0;
        for constructor in &datatype.constructors {
            let mut constructor_cardinality = 1;
            for field in &constructor.fields {
                if let Some(card) = self.get_cardinality(&field.sort.to_string()) {
                    constructor_cardinality *= card;
                } else {
                    return None; // field type is infinite
                }
            }
            total += constructor_cardinality;
        }
        Some(total)
    }

    /// check if the datatype is recursive
    fn is_datatype_recursive(&self, datatype: &DataType) -> bool {
        for constructor in &datatype.constructors {
            for field in &constructor.fields {
                if field.sort == Sort::Simple(Identifier::Symbol(datatype.name.clone())) {
                    return true; // found recursive reference
                }
            }
        }
        false
    }
}

/// type inferer
#[derive(Debug)]
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
        let current_scope = self._current_scope();
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

    fn register_builtins(&mut self) {
        debug!("register built-in types and functions");
        // register built-in constants
        self.global_scope_mut()
            .bind_variable("true".to_string(), Sort::bool());
        self.global_scope_mut()
            .bind_variable("false".to_string(), Sort::bool());

        // register built-in functions
        self.global_scope_mut().register_function_type(
            "and".to_string(),
            FunctionType::new_signature(
                "and".to_string(),
                vec![Sort::bool(), Sort::bool()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "or".to_string(),
            FunctionType::new_signature(
                "or".to_string(),
                vec![Sort::bool(), Sort::bool()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "not".to_string(),
            FunctionType::new_signature("not".to_string(), vec![Sort::bool()], Sort::bool()),
        );

        self.global_scope_mut().register_function_type(
            "=".to_string(),
            FunctionType::new_signature(
                "=".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "<".to_string(),
            FunctionType::new_signature(
                "<".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            ">".to_string(),
            FunctionType::new_signature(
                ">".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "<=".to_string(),
            FunctionType::new_signature(
                "<=".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            ">=".to_string(),
            FunctionType::new_signature(
                ">=".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::bool(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "+".to_string(),
            FunctionType::new_signature(
                "+".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::int(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "-".to_string(),
            FunctionType::new_signature(
                "-".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::int(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "*".to_string(),
            FunctionType::new_signature(
                "*".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::int(),
            ),
        );

        self.global_scope_mut().register_function_type(
            "/".to_string(),
            FunctionType::new_signature(
                "/".to_string(),
                vec![Sort::int(), Sort::int()],
                Sort::int(),
            ),
        );

        let ite_func = FunctionType::new_signature(
            "ite".to_string(),
            vec![Sort::bool(), Sort::int(), Sort::int()], // condition, then-branch, else-branch
            Sort::int(),
        );
        self.global_scope_mut()
            .register_function_type("ite".to_string(), ite_func);

        // register finiteness function
        let finite_func = FunctionType::new_signature(
            "finite".to_string(),
            vec![Sort::Simple(Identifier::Symbol(ANY_TYPE.to_string()))], // parameter can be any type
            Sort::bool(),
        );
        self.global_scope_mut()
            .register_function_type("finite".to_string(), finite_func);

        // register cardinality function
        let cardinality_func = FunctionType::new_signature(
            "card".to_string(),
            vec![Sort::Simple(Identifier::Symbol(ANY_TYPE.to_string()))], // parameter can be any type
            Sort::int(),
        );
        self.global_scope_mut()
            .register_function_type("card".to_string(), cardinality_func);

        debug!(
            "built-in functions registered, total {} functions",
            self.global_scope().function_types.len()
        );
    }

    /// get current scope
    fn _current_scope(&self) -> &TypeEnv {
        &self.stack[self.stack.len() - 1]
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
        self.stack.push(TypeEnv::new());
    }

    pub fn exit_scope(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn infer(&mut self, expr: &LSTerm) -> Result<Sort, String> {
        let result = match expr {
            LSTerm::Literal(lit) => {
                match lit {
                    sygus_parser::Literal::Numeral(_) => Ok(Sort::int()),
                    sygus_parser::Literal::Decimal(_) => Ok(Sort::real()),
                    sygus_parser::Literal::Bool(_) => Ok(Sort::bool()),
                    _ => Ok(Sort::string()), // default string type
                }
            }
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();
                if let Some(sort) = self.find_variable_type(&sym) {
                    Ok(sort.clone())
                } else {
                    // check if it is a constructor (nullary constructor)
                    for scope in &self.stack {
                        for (_, datatype) in &scope.datatypes {
                            for constructor in &datatype.constructors {
                                if constructor.name == sym && constructor.fields.is_empty() {
                                    debug!(
                                        "Found constructor: {} for datatype {}",
                                        sym, datatype.name
                                    );
                                    return Ok(Sort::Simple(Identifier::Symbol(
                                        datatype.name.clone(),
                                    )));
                                }
                            }
                        }
                    }

                    // check if it is an enum value
                    for scope in &self.stack {
                        for (_, enum_type) in &scope.enums {
                            if enum_type.elements.contains(&sym) {
                                debug!("Found enum element: {} for enum {}", sym, enum_type.name);
                                return Ok(Sort::Simple(Identifier::Symbol(
                                    enum_type.name.clone(),
                                )));
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
                param,
                ret_sort,
            } => self.infer_modal_expr(name, args, ret_sort, body, param),
            LSTerm::NuTerm {
                name,
                args,
                body,
                param,
                ret_sort,
            } => self.infer_modal_expr(name, args, ret_sort, body, param),
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
            debug!("Found function type: {:?}", func_type);
            // check argument number
            if func_type.params.len() != arg_types.len() {
                return Err(format!(
                    "Function {} expects {} arguments, but {} were provided",
                    func,
                    func_type.params.len(),
                    arg_types.len()
                ));
            }

            // check argument types
            for (i, (expected, actual)) in func_type.params.iter().zip(arg_types.iter()).enumerate()
            {
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

            Ok(func_type.ret.clone())
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
                            return Ok(Sort::Simple(Identifier::Symbol(datatype.name.clone())));
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
                            return Ok(Sort::Simple(Identifier::Symbol(enum_type.name.clone())));
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

    fn infer_builtin_function(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Sort, String> {
        match func_name {
            "and" | "or" => {
                if args.len() < 2 {
                    return Err("and/or: expected at least 2 arguments".to_string());
                }
                for arg in args {
                    let arg_type = self.infer(arg)?;
                    if arg_type != Sort::bool() {
                        return Err("and/or: expected boolean arguments".to_string());
                    }
                }
                Ok(Sort::bool())
            }
            "not" => {
                if args.len() != 1 {
                    return Err("not: expected exactly 1 argument".to_string());
                }
                let arg_type = self.infer(&args[0])?;
                if arg_type != Sort::bool() {
                    return Err("not: expected boolean argument".to_string());
                }
                Ok(Sort::bool())
            }
            "=" | "<" | ">" | "<=" | ">=" => {
                if args.len() != 2 {
                    return Err(format!("{}: expected exactly 2 arguments", func_name));
                }
                let arg1_type = self.infer(&args[0])?;
                let arg2_type = self.infer(&args[1])?;
                if arg1_type != Sort::int() && arg1_type != Sort::real() {
                    return Err(format!("{}: expected numeric arguments", func_name));
                }
                if arg2_type != Sort::int() && arg2_type != Sort::real() {
                    return Err(format!("{}: expected numeric arguments", func_name));
                }
                Ok(Sort::bool())
            }
            "+" | "-" | "*" | "/" => {
                if args.len() != 2 {
                    return Err(format!("{}: expected exactly 2 arguments", func_name));
                }
                let arg1_type = self.infer(&args[0])?;
                let arg2_type = self.infer(&args[1])?;
                if arg1_type != Sort::int() && arg1_type != Sort::real() {
                    return Err(format!("{}: expected numeric arguments", func_name));
                }
                if arg2_type != Sort::int() && arg2_type != Sort::real() {
                    return Err(format!("{}: expected numeric arguments", func_name));
                }
                Ok(Sort::int()) // simplified handling, should be determined by parameter types
            }
            "ite" => {
                if args.len() != 3 {
                    return Err("ite: expected exactly 3 arguments".to_string());
                }
                let cond_type = self.infer(&args[0])?;
                if cond_type != Sort::bool() {
                    return Err("ite: expected boolean condition".to_string());
                }
                let then_type = self.infer(&args[1])?;
                let else_type = self.infer(&args[2])?;
                if then_type != else_type {
                    return Err("ite: then and else branches must have the same type".to_string());
                }
                Ok(then_type) // return the type of the then branch
            }
            "finite" => {
                if args.len() != 1 {
                    return Err("finite: expected exactly 1 argument".to_string());
                }
                let arg_type = self.infer(&args[0])?;
                // accept any type identifier
                if let Sort::Simple(Identifier::Symbol(_)) = arg_type {
                    Ok(Sort::bool())
                } else {
                    Err("finite: expected type identifier".to_string())
                }
            }
            "card" => {
                if args.len() != 1 {
                    return Err("cardinality: expected exactly 1 argument".to_string());
                }
                let arg_type = self.infer(&args[0])?;
                // accept any type identifier
                if let Sort::Simple(Identifier::Symbol(_)) = arg_type {
                    Ok(Sort::int())
                } else {
                    Err("cardinality: expected type identifier".to_string())
                }
            }
            _ => {
                error!("builtin function not found: {}", func_name);
                Err(format!("Unknown builtin function: {}", func_name))
            }
        }
    }

    fn infer_match_expr(
        &mut self,
        term: &Box<LSTerm>,
        cases: &[MatchCase],
    ) -> Result<Sort, String> {
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
        matcher.bind_pattern_variables_for_type_inference(pattern, self, term_type)
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

        // construct function type
        let param_types: Vec<Sort> = args.iter().map(|sv| sv.sort.clone()).collect();
        let mut func_type = body_type.to_string();
        for param_type in param_types.iter().rev() {
            func_type = format!("(-> {} {})", param_type.to_string(), func_type);
        }
        Ok(Sort::Simple(Identifier::Symbol(func_type)))
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
        // lookup in scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(sort) = scope.get_variable_type(name) {
                return Some(sort);
            }
        }
        None
    }

    fn find_function_type(&self, name: &str) -> Option<&FunctionType> {
        for scope in &self.stack {
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
        _param: &Box<LSTerm>,
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

        // check return type
        if body_type != *ret_sort {
            return Err(format!(
                "Mu expression body type {} does not match declared return type {}",
                body_type, ret_sort
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

                    // Check argument count and types
                    if method_type.params.len() != arg_types.len() {
                        return Err(format!(
                            "Method {} expects {} arguments, but {} were provided",
                            method,
                            method_type.params.len(),
                            arg_types.len()
                        ));
                    }

                    // Instantiate type parameters in method signature
                    let instantiated_ret =
                        self.instantiate_type(&method_type.ret, &class.type_params, &type_args);
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

                // Check argument count
                if method_type.params.len() != arg_types.len() {
                    return Err(format!(
                        "Method {} expects {} arguments, but {} were provided",
                        method,
                        method_type.params.len(),
                        arg_types.len()
                    ));
                }

                // Check argument types
                for (i, (expected, actual)) in
                    method_type.params.iter().zip(arg_types.iter()).enumerate()
                {
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

                return Ok(method_type.ret.clone());
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
    fn instantiate_type(&self, sort: &Sort, type_params: &[String], type_args: &[String]) -> Sort {
        match sort {
            Sort::Simple(Identifier::Symbol(name)) => {
                // Check if it's a type parameter
                if let Some(index) = type_params.iter().position(|p| p == name) {
                    if index < type_args.len() {
                        return Sort::Simple(Identifier::Symbol(type_args[index].clone()));
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

    /// Handle arrow sorts in type checking
    pub fn parse_arrow_sort(&self, sorts: Vec<Sort>) -> Sort {
        if sorts.len() < 2 {
            panic!("Arrow sort must have at least 2 sorts");
        }

        let mut result = sorts.last().unwrap().clone();
        for i in (0..sorts.len() - 1).rev() {
            result = Sort::Parameterized(
                Identifier::Symbol("->".to_string()),
                vec![sorts[i].clone(), result],
            );
        }
        result
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
