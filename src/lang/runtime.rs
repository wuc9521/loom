use crate::parser::ast::{ClassDef, ObjectDef, LSTerm};
use derive_more::Display;
use std::collections::HashMap;
use sygus_parser::SortedVar;

#[derive(Debug, Default, Clone)]
pub struct ModelSymbolTable {
    table: std::collections::HashMap<String, SymbolInterp>,
}

#[derive(Debug, Clone)]
pub enum SymbolInterp {
    Univ(crate::lang::model::Univ),
}

impl ModelSymbolTable {
    pub fn new() -> Self {
        Self {
            table: std::collections::HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, interp: SymbolInterp) {
        self.table.insert(name, interp);
    }
    pub fn get(&self, name: &str) -> Option<&SymbolInterp> {
        self.table.get(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolInterp)> {
        self.table.iter()
    }
}

/// Method signature for class methods
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub params: Vec<sygus_parser::SortedVar>,
    pub ret_sort: sygus_parser::Sort,
    pub body: Option<LSTerm>,
    pub is_recursive: bool,
}

/// Class definition value
#[derive(Debug, Clone, Display)]
#[display(fmt = "<class:{}>", name)]
pub struct ClassValue {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, sygus_parser::Sort)>,
    pub methods: HashMap<String, MethodSignature>,
}

/// Object instance value
#[derive(Debug, Clone, Display)]
#[display(fmt = "<object:{}:{}>", name, class_name)]
pub struct ObjectValue {
    pub name: String,
    pub class_name: String,
    pub type_args: Vec<String>,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, Value>,
}

#[derive(Debug, Clone, Display)]
pub enum Value {
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    String(String),
    #[display(fmt = "{}", _0)]
    Function(Function),
    #[display(fmt = "{}", _0)]
    Model(crate::lang::model::Model),
    #[display(fmt = "{}", _0)]
    Term(LSTerm),
    #[display(
        fmt = "({})",
        "_0.iter().map(|v| format!(\"{}\", v)).collect::<Vec<_>>().join(\" \")"
    )]
    List(Vec<Value>),
    #[display(fmt = "{}", _0)]
    Symbol(String),
    #[display(fmt = "<constructor:{}>", name)]
    Constructor {
        name: String,
        datatype: String,
        arity: usize,
    },
    #[display(fmt = "{}", constructor)]
    DataValue {
        constructor: String,
        datatype: String,
        fields: Vec<Value>,
    },
    #[display(fmt = "{}", _0)]
    Class(ClassValue),
    #[display(fmt = "{}", _0)]
    Object(ObjectValue),
    #[display(fmt = "<synthesis_placeholder:{}>", name)]
    SynthesisPlaceholder {
        name: String,
        return_type: sygus_parser::Sort,
    },
}

#[derive(Debug, Clone, Display)]
#[display(fmt = "{}", name)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: LSTerm,
    pub closure: Scope,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    /// Define a class in the current scope
    pub fn define_class(&mut self, class_def: &ClassDef) {
        let mut fields = Vec::new();
        for field in &class_def.fields {
            fields.push((field.name.clone(), field.sort.clone()));
        }
        
        let mut methods = HashMap::new();
        for method in &class_def.methods {
            let mut params = Vec::new();
            let method_params = method.get_params();
            for (i, param) in method_params.iter().enumerate() {
                params.push(SortedVar {
                    name: method.param_names[i].clone(),
                    sort: param.clone(),
                });
            }
            methods.insert(method.name.clone().unwrap(), MethodSignature {
                params,
                ret_sort: method.get_ret(),
                body: method.body.clone(),
                is_recursive: method.is_constructor,
            });
        }
        
        let class_value = ClassValue {
            name: class_def.name.clone(),
            type_params: class_def.type_params.clone(),
            fields,
            methods,
        };
        
        self.define(class_def.name.clone(), Value::Class(class_value));
    }
    
    /// Define an object instance in the current scope
    pub fn define_object(&mut self, object_def: &ObjectDef, _class: &ClassValue) {
        let mut methods = HashMap::new();
        
        // Convert method implementations to functions
        for method_impl in &object_def.methods {
            if let Some(body) = &method_impl.body {
                let function = Function {
                    name: format!("{}.{}", object_def.name, method_impl.name.as_ref().unwrap()),
                    params: method_impl.param_names.clone(),
                    body: body.clone(),
                    closure: Scope::new(), // Object methods don't need closure as they access state via 'self'
                };
                methods.insert(method_impl.name.as_ref().unwrap().clone(), function);
            }
        }
        
        let object_value = ObjectValue {
            name: object_def.name.clone(),
            class_name: object_def.class_name.clone(),
            type_args: object_def.type_args.iter().map(|s| s.to_string()).collect(),    
            fields: HashMap::new(), // Fields can be initialized later
            methods,
        };
        
        self.define(object_def.name.clone(), Value::Object(object_value));
    }
}