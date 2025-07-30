use crate::lang::interpreter::Interpreter;
use crate::lang::runtime::{Function, Scope, Value};
use crate::lang::types::FunctionType;
use crate::parser::ast::{ClassDef, ObjectDef};
use log::debug;

impl Interpreter {
    /// Define a class
    pub fn define_class(&mut self, class_def: ClassDef) -> Result<(), String> {
        debug!("Defining class: {}", class_def.name);

        // Check for conflicts
        self.typing.check_all_conflicts(&class_def.name, "class")?;

        // Type check the class definition
        self.type_check_class(&class_def)?;

        // Store class in runtime
        self.current_scope_mut().define_class(&class_def);

        // Register class type in type environment
        self.typing.current_scope_mut().register_class_type(
            class_def.name.clone(),
            class_def.type_params.clone(),
            class_def
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.sort.clone()))
                .collect(),
        );

        // Register class methods as global functions for constraint processing
        for method in &class_def.methods {
            if let Some(body) = &method.body {
                debug!(
                    "Registering class method '{}' as global function",
                    method.name.as_ref().unwrap()
                );

                // Use the original parameter names (which already include 'self')
                let method_params = method.param_names.clone();

                let function = Function {
                    name: method.name.clone().unwrap(),
                    params: method_params,
                    body: body.clone(),
                    closure: self.current_scope().clone(),
                };

                // Register to global scope so it can be found in constraints
                self.global_scope_mut()
                    .define(method.name.clone().unwrap(), Value::Function(function));
            }
        }

        // Also register class fields to global scope so class methods can access them
        for field in &class_def.fields {
            debug!("Registering class field '{}' to global scope", field.name);
            let field_function_type = FunctionType::from_sort(field.name.clone(), &field.sort);
            self.typing
                .global_scope_mut()
                .register_function_type(field.name.clone(), field_function_type);
        }

        Ok(())
    }

    /// Define an object (instance of a class)
    pub fn define_object(&mut self, object_def: ObjectDef) -> Result<(), String> {
        debug!(
            "Defining object: {} of class {}",
            object_def.name, object_def.class_name
        );

        // Check for conflicts
        self.typing
            .check_all_conflicts(&object_def.name, "object")?;

        // Look up the class
        let class_value = match self.lookup_symbol_value(&object_def.class_name) {
            Some(Value::Class(class)) => class,
            _ => return Err(format!("Class '{}' not found", object_def.class_name)),
        };

        // Type check the object definition
        self.type_check_object(&object_def, &class_value)?;

        // Store object in runtime
        self.current_scope_mut()
            .define_object(&object_def, &class_value);

        // Register object methods as global functions for constraint processing
        for method_impl in &object_def.methods {
            if let Some(body) = &method_impl.body {
                debug!(
                    "Registering object method '{}' as global function",
                    method_impl.name.as_ref().unwrap()
                );

                // Use the original parameter names (which already include 'self')
                let method_params = method_impl.param_names.clone();

                let function = Function {
                    name: method_impl.name.clone().unwrap(),
                    params: method_params,
                    body: body.clone(),
                    closure: Scope::new(), // Object methods don't need closure as they access state via 'self'
                };

                // Register to global scope so it can be found in constraints
                self.global_scope_mut()
                    .define(method_impl.name.clone().unwrap(), Value::Function(function));
            }
        }

        // Also register object fields to global scope so class methods can access them
        for field in &class_value.fields {
            debug!("Registering object field '{}' to global scope", field.0);
            let field_function_type = FunctionType::from_sort(field.0.clone(), &field.1);
            self.typing
                .global_scope_mut()
                .register_function_type(field.0.clone(), field_function_type);
        }

        Ok(())
    }

    /// Type check a class definition
    fn type_check_class(&mut self, class_def: &ClassDef) -> Result<(), String> {
        debug!("Type checking class: {}", class_def.name);

        // Enter class scope
        self.typing.enter_scope();

        // Bind type parameters
        for type_param in &class_def.type_params {
            self.typing.current_scope_mut().bind_variable(
                type_param.clone(),
                sygus_parser::Sort::Simple(sygus_parser::Identifier::Symbol(type_param.clone())),
            );
        }

        // Bind "self" to class type in method signatures
        let class_sort =
            sygus_parser::Sort::Simple(sygus_parser::Identifier::Symbol(class_def.name.clone()));

        // First, register all class fields as function types in the class scope
        // This allows methods to call field accessors during type checking
        // Fields are accessed directly without self parameter in class context
        for field in &class_def.fields {
            // Register field with its original type (no self parameter)
            self.typing.current_scope_mut().register_function_type(
                field.name.clone(),
                FunctionType::from_sort(field.name.clone(), &field.sort),
            );
        }

        // Then, register all class methods as function types in the class scope
        // This allows methods to call each other during type checking
        for method in &class_def.methods {
            if let Some(body) = &method.body {
                let method_type = FunctionType::new_definition(
                    method.name.clone().unwrap(),
                    method.param_names.clone(),
                    method.get_params(),
                    method.get_ret(),
                    body.clone(),
                );
                self.typing
                    .current_scope_mut()
                    .register_function_type(method.name.clone().unwrap(), method_type);
            }
        }

        // Type check methods
        for method in &class_def.methods {
            self.typing.enter_scope();

            // Automatically bind self to class type (user doesn't need to declare it)
            self.typing
                .current_scope_mut()
                .bind_variable("self".to_string(), class_sort.clone());

            let params = method.get_params();
            let ret = method.get_ret();

            // Bind method parameters (excluding self which is automatically bound)
            for (i, param) in params.iter().enumerate() {
                self.typing
                    .current_scope_mut()
                    .bind_variable(method.param_names[i].clone(), param.clone());
            }

            // Type check method body if present
            if let Some(body) = &method.body {
                let body_type = self.typing.infer(body)?;
                if body_type != ret {
                    return Err(format!(
                        "Method {} body type {} does not match declared return type {}",
                        method.name.clone().unwrap(),
                        body_type,
                        ret
                    ));
                }
            }

            self.typing.exit_scope();
        }

        self.typing.exit_scope();
        Ok(())
    }

    /// Type check an object definition
    fn type_check_object(
        &mut self,
        object_def: &ObjectDef,
        class: &crate::lang::runtime::ClassValue,
    ) -> Result<(), String> {
        debug!("Type checking object: {}", object_def.name);

        // Check type arguments match class parameters
        if object_def.type_args.len() != class.type_params.len() {
            return Err(format!(
                "Object {} has {} type arguments but class {} expects {}",
                object_def.name,
                object_def.type_args.len(),
                class.name,
                class.type_params.len()
            ));
        }

        // Enter object scope once for all method checking
        self.typing.enter_scope();

        // Bind type parameters to concrete types once
        for (param, arg) in class.type_params.iter().zip(object_def.type_args.iter()) {
            debug!("Binding type parameter: {} -> {}", param, arg);
            self.typing
                .current_scope_mut()
                .bind_variable(param.clone(), arg.clone());
        }

        // Bind self to object type once
        let object_sort =
            sygus_parser::Sort::Simple(sygus_parser::Identifier::Symbol(object_def.name.clone()));
        debug!("Binding self -> {}", object_sort);
        self.typing
            .current_scope_mut()
            .bind_variable("self".to_string(), object_sort);

        // Type check each method implementation or field initialization
        for method_impl in &object_def.methods {
            let method_name = method_impl.name.as_ref().unwrap();

            // First check if it's a method in the class
            if let Some(_class_method) = class.methods.get(method_name) {
                // It's a method implementation
                let params = method_impl.get_params();
                let ret = method_impl.get_ret();

                // Bind method parameters (self is automatically bound)
                for (i, param_type) in params.iter().enumerate() {
                    let param_name = &method_impl.param_names[i];
                    debug!("Binding method parameter: {} -> {}", param_name, param_type);
                    self.typing
                        .current_scope_mut()
                        .bind_variable(param_name.clone(), param_type.clone());
                }

                // Type check method body
                if let Some(body) = &method_impl.body {
                    let body_type = self.typing.infer(body)?;
                    if body_type != ret {
                        return Err(format!(
                            "Method {} implementation body type {} does not match declared return type {}",
                            method_name, body_type, ret
                        ));
                    }
                }
            } else {
                // Check if it's a field implementation
                let field_found = class
                    .fields
                    .iter()
                    .any(|(field_name, _)| field_name == method_name);

                if !field_found {
                    return Err(format!(
                        "Field {} not found in class {}",
                        method_name, class.name
                    ));
                }

                // Get the field type and check if method implements it correctly
                if let Some((_, field_type)) = class
                    .fields
                    .iter()
                    .find(|(field_name, _)| field_name == method_name)
                {
                    // Instantiate the field type with concrete type arguments
                    let instantiated_field_type = self.typing.instantiate_type(
                        field_type,
                        &class.type_params,
                        &object_def
                            .type_args
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>(),
                    );

                    // Use the universal function type decomposition to handle all cases
                    let (expected_params, expected_return) =
                        FunctionType::decompose_arrow_sort(&instantiated_field_type);

                    // Check if the method implementation matches the field type
                    let params = method_impl.get_params();
                    let ret = method_impl.get_ret();

                    if params.len() == expected_params.len() {
                        // Function field: check parameter and return types
                        // Check each parameter type
                        for (i, (param_type, expected_param)) in
                            params.iter().zip(expected_params.iter()).enumerate()
                        {
                            if param_type != expected_param {
                                return Err(format!(
                                    "Field {} implementation parameter {} type {} does not match field parameter type {}",
                                    method_name, i, param_type, expected_param
                                ));
                            }
                        }

                        // Check return type
                        if &ret != &expected_return {
                            return Err(format!(
                                "Field {} implementation return type {} does not match field return type {}",
                                method_name, ret, expected_return
                            ));
                        }
                    } else {
                        // Non-function field: check direct type match
                        if &ret != &instantiated_field_type {
                            return Err(format!(
                                "Field {} implementation return type {} does not match field type {}",
                                method_name, ret, instantiated_field_type
                            ));
                        }
                    }
                }
            }
        }

        // Register object methods to type environment for method calls
        for method_impl in &object_def.methods {
            let method_name = method_impl.name.as_ref().unwrap();
            debug!(
                "Registering object method '{}' to type environment",
                method_name
            );

            let method_type = FunctionType::new_signature(
                method_name.clone(),
                method_impl.get_params(),
                method_impl.get_ret(),
            );

            self.typing
                .current_scope_mut()
                .register_function_type(method_name.clone(), method_type);
        }

        // Exit object scope
        self.typing.exit_scope();

        Ok(())
    }
}
