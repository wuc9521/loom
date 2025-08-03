use std::collections::HashMap;
use sygus_parser::{Identifier, Sort};

use super::basic::{ConstructorInfo, DataType, EnumType, ClassType, Finiteness};
use super::function::FunctionType;

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