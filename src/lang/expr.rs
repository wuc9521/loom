use crate::lang::interpreter::{Interpreter, SymbolLookupResult};
use crate::lang::model::MatchCase;
use crate::lang::runtime::{Function, Value};
use crate::parser::ast::LSTerm;
use log::{debug, error};

impl Interpreter {
    /// evaluate an expression
    pub fn eval(&mut self, term: &LSTerm) -> Result<Value, String> {
        match term {
            LSTerm::Identifier(ident) => {
                let sym = ident.to_string();

                // Check if this is 'self' - handle specially in class context
                if sym == "self" {
                    // Try to find self in current scope first
                    if let Some(value) = self.lookup_symbol_value("self") {
                        return Ok(value);
                    }
                    // If not found in scope, check if we're in object context
                    if let Some(current_obj_name) = &self.current_object {
                        if let Some(obj_value) = self.lookup_symbol_value(current_obj_name) {
                            return Ok(obj_value);
                        }
                    }
                    return Err("'self' not available in current context".to_string());
                }

                match self.lookup_symbol(&sym) {
                    SymbolLookupResult::Variable(value) => {
                        // if the value is a function, call it (no argument function call)
                        match &value {
                            Value::Function(func) => {
                                debug!("Calling function without arguments: {}", func.name);
                                self.call_function(func, &[])
                            }
                            _ => Ok(value),
                        }
                    }
                    SymbolLookupResult::Constructor(info) => {
                        if info.field_types.is_empty() {
                            // nullary constructor, create data value
                            Ok(Value::DataValue {
                                constructor: info.name,
                                datatype: info.datatype,
                                fields: vec![],
                            })
                        } else {
                            // constructor with arguments, return constructor value
                            Ok(Value::Constructor {
                                name: info.name,
                                datatype: info.datatype,
                                arity: info.field_types.len(),
                            })
                        }
                    }
                    SymbolLookupResult::EnumElement { name, enum_type: _ } => {
                        Ok(Value::Symbol(name))
                    }
                    SymbolLookupResult::ToBeSynthed { name, return_type } => {
                        debug!("Encountered synthesis function placeholder: {}", name);
                        Ok(Value::SynthesisPlaceholder {
                            name: name.clone(),
                            return_type: return_type.clone(),
                        })
                    }
                    SymbolLookupResult::NotFound => {
                        error!("undefined symbol: {}", sym);
                        Err(format!("Undefined symbol: {}", sym))
                    }
                }
            }
            LSTerm::Literal(lit) => match lit {
                sygus_parser::Literal::Numeral(n) => {
                    let value = Value::Int(*n as i64);
                    Ok(value)
                }
                sygus_parser::Literal::Decimal(d) => {
                    let value = Value::String(d.to_string());
                    Ok(value)
                }
                _ => Ok(Value::Symbol(lit.to_string())),
            },
            LSTerm::MethodCallTerm { object, method } => {
                // Handle direct method call without arguments
                self.eval_method_call(object, method, &[])
            }
            LSTerm::AppTerm(func, args) => {
                debug!(
                    "evaluate function call: {} (parameters number: {})",
                    func.to_string(),
                    args.len()
                );
                self.eval_func_app(&func.to_string(), args)
            }
            LSTerm::AttrTerm(term, _attrs) => self.eval(term),
            LSTerm::MatchTerm { term, cases } => self.eval_match_term(term, cases),
            LSTerm::LambdaTerm { args, body } => {
                let param_names: Vec<String> = args.iter().map(|sv| sv.name.clone()).collect();
                let function = Function {
                    name: "lambda".to_string(),
                    params: param_names,
                    body: *body.clone(),
                    closure: self.current_scope().clone(), // Capture current scope for closure
                };
                Ok(Value::Function(function))
            }
            LSTerm::MuTerm {
                name,
                args: _args,
                body,
                param: _,
                ret_sort: _,
            } => self.eval_mu(name, body),
            LSTerm::NuTerm {
                name: _name,
                args: _args,
                body: _body,
                param: _,
                ret_sort: _,
            } => {
                // TODO: implement nu expression
                unimplemented!()
            }
            LSTerm::ForallTerm { args: _, body } => {
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
            LSTerm::ExistsTerm { args: _, body } => {
                self.enter_scope();
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
            LSTerm::LetTerm { args, body } => {
                debug!("evaluate let expression (bindings number: {})", args.len());
                self.enter_scope();
                for binding in args {
                    let value = self.eval(&binding.term)?;
                    self.current_scope_mut().define(binding.name.clone(), value);
                }
                let result = self.eval(body)?;
                self.exit_scope();
                Ok(result)
            }
        }
    }

    /// evaluate a function call
    pub fn eval_func_app(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Value, String> {
        debug!(
            "evaluating function call: {} with {} arguments",
            func_name,
            args.len()
        );

        // Check if this is a method call (object.method)
        if func_name.contains('.') {
            let parts: Vec<&str> = func_name.split('.').collect();
            if parts.len() == 2 {
                let (receiver, method) = (parts[0], parts[1]);
                return self.eval_method_call(receiver, method, args);
            }
        }

        // check if it is a constructor
        match self.lookup_symbol(func_name) {
            SymbolLookupResult::Constructor(info) => self.eval_constructor_call(&info, args),
            SymbolLookupResult::Variable(Value::Function(func)) => self.call_function(&func, args),
            SymbolLookupResult::EnumElement { name, enum_type: _ } => {
                // Enum elements are constants, not functions
                if args.is_empty() {
                    Ok(Value::Symbol(name))
                } else {
                    Err(format!(
                        "Enum element '{}' is not a function and cannot be called with arguments",
                        func_name
                    ))
                }
            }
            SymbolLookupResult::ToBeSynthed { name, return_type } => {
                debug!("Encountered synthesis function call: {}", name);
                // For synthesis functions, return a placeholder value with the actual return type
                Ok(Value::SynthesisPlaceholder {
                    name: name.clone(),
                    return_type: return_type.clone(),
                })
            }
            _ => self.eval_builtin_function(func_name, args),
        }
    }

    /// Evaluate a method call (object.method)
    fn eval_method_call(
        &mut self,
        receiver: &str,
        method: &str,
        args: &[LSTerm],
    ) -> Result<Value, String> {
        debug!(
            "evaluating method call: {}.{} with {} arguments",
            receiver,
            method,
            args.len()
        );

        // Handle "self" specially
        if receiver == "self" {
            if let Some(current_obj_name) = &self.current_object {
                if let Some(Value::Object(obj)) = self.lookup_symbol_value(current_obj_name) {
                    if let Some(func) = obj.methods.get(method) {
                        return self.call_function(func, args);
                    }
                }
            }
            return Err("'self' used outside of object context".to_string());
        }

        // Look up the receiver object
        let receiver_value = self
            .lookup_symbol_value(receiver)
            .ok_or_else(|| format!("Receiver '{}' not found", receiver))?;

        match receiver_value {
            Value::Object(obj) => {
                // Look up the method in the object
                if let Some(func) = obj.methods.get(method) {
                    // Set current object context
                    let prev_object = self.current_object.clone();
                    self.current_object = Some(receiver.to_string());

                    let result = self.call_function(func, args);

                    // Restore previous context
                    self.current_object = prev_object;

                    result
                } else {
                    Err(format!(
                        "Method '{}' not found in object '{}'",
                        method, receiver
                    ))
                }
            }
            _ => Err(format!(
                "'{}' is not an object and cannot have methods",
                receiver
            )),
        }
    }

    /// Call a function with arguments
    fn call_function(&mut self, func: &Function, args: &[LSTerm]) -> Result<Value, String> {
        debug!(
            "calling function: {} with {} arguments",
            func.name,
            args.len()
        );

        // check argument count
        if args.len() != func.params.len() {
            return Err(format!(
                "Function {} expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            ));
        }

        // enter function scope
        self.enter_scope();

        // Restore closure environment (captured variables from function definition)
        for (name, value) in &func.closure.bindings {
            self.current_scope_mut().define(name.clone(), value.clone());
        }

        // evaluate arguments and bind to parameters
        for (param_name, arg) in func.params.iter().zip(args.iter()) {
            let arg_value = self.eval(arg)?;
            self.define_symbol(param_name.clone(), arg_value);
        }

        // Check if the first parameter is an object and bind it as 'self'
        if !func.params.is_empty() {
            let first_param_name = &func.params[0];
            if let Some(first_arg_value) = self.lookup_symbol_value(first_param_name) {
                match &first_arg_value {
                    Value::Object(_) => {
                        // Bind the first parameter as 'self' for class methods
                        self.current_scope_mut()
                            .define("self".to_string(), first_arg_value);
                    }
                    _ => {}
                }
            }
        }

        // evaluate function body
        let result = self.eval(&func.body)?;

        // exit function scope
        self.exit_scope();

        debug!("function {} returned: {}", func.name, result);
        Ok(result)
    }

    fn eval_match_term(&mut self, term: &LSTerm, cases: &[MatchCase]) -> Result<Value, String> {
        // evaluate the expression to be matched
        let value = self.eval(term)?;
        debug!("Matching value: {}", value);

        // Check if the value is a synthesis placeholder
        if let Value::SynthesisPlaceholder {
            name,
            return_type: _,
        } = &value
        {
            debug!(
                "Synthesis placeholder '{}' encountered in pattern matching, returning placeholder",
                name
            );
            // For synthesis placeholders, return the placeholder itself
            // This allows the constraint solver to handle the pattern matching
            return Ok(value);
        }

        // try each case
        for case in cases {
            if let Some(pattern) = &case.pattern {
                debug!("Trying pattern: {}", pattern);

                // create pattern matcher
                let matcher = crate::lang::pattern::PatternMatcher::new();

                // Create pattern context with current type environment
                // Get the current type environment from the interpreter's type inferer
                // Since we can't access private methods, we'll create a context with the global scope
                let type_env = self.typing.current_scope().clone();
                let context = crate::lang::pattern::PatternContext::new(type_env);

                // perform pattern matching with context
                match matcher.matches_with_context(&value, pattern, context) {
                    Ok(result) => {
                        if result.matched {
                            debug!("Pattern matched, evaluating body");
                            // pattern matched, evaluate the body
                            return self.eval(&case.value);
                        } else {
                            debug!("Pattern did not match, trying next case");
                            // pattern did not match, try next case
                            continue;
                        }
                    }
                    Err(e) => {
                        debug!("Pattern matching error: {}", e);
                        // pattern matching error, try next case
                        continue;
                    }
                }
            } else {
                // no pattern (default case), evaluate the body
                debug!("No pattern (default case), evaluating body");
                return self.eval(&case.value);
            }
        }

        // no pattern matched
        Err(format!("No pattern matched for value: {}", value))
    }

    pub fn eval_mu(&mut self, binder: &str, _body: &Box<LSTerm>) -> Result<Value, String> {
        debug!("Evaluating mu expression with binder: {}", binder);
        // TODO
        Ok(Value::Bool(false))
    }
}
