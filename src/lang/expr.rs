use crate::lang::builtin::BuiltinFunctions;
use crate::lang::interpreter::Interpreter;
use crate::lang::interpreter::SymbolLookupResult;
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

                match self.lookup_symbol_enhanced(&sym) {
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
                    closure: self.current_scope().clone(),
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

        // check if it is a constructor
        match self.lookup_symbol_enhanced(func_name) {
            SymbolLookupResult::Constructor(info) => self.eval_constructor_call(&info, args),
            SymbolLookupResult::Variable(Value::Function(func)) => self.call_function(&func, args),
            _ => {
                // try builtin function
                if let Some(builtin_result) =
                    BuiltinFunctions::eval_builtin_function(func_name, args, &mut |term| {
                        self.eval(term)
                    })
                {
                    builtin_result
                } else {
                    Err(format!("'{}' is not a function", func_name))
                }
            }
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

        // evaluate arguments and bind to parameters
        for (param_name, arg) in func.params.iter().zip(args.iter()) {
            let arg_value = self.eval(arg)?;
            debug!(
                "Binding parameter '{}' to value: {:?}",
                param_name, arg_value
            );
            self.define_symbol(param_name.clone(), arg_value);
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
                            debug!("Pattern matched!");

                            // enter new scope, bind variables in pattern
                            self.enter_scope();

                            // bind matched variables to current scope
                            for (name, val) in result.context.get_runtime_scope().bindings.iter() {
                                self.current_scope_mut().define(name.clone(), val.clone());
                            }

                            // evaluate the value expression of the case
                            let result_value = self.eval(&case.value)?;

                            // exit scope
                            self.exit_scope();

                            return Ok(result_value);
                        }
                    }
                    Err(e) => {
                        debug!("Pattern matching error: {}", e);
                        // continue to try the next case
                    }
                }
            }
        }

        Err(format!("No pattern matched for value: {}", value))
    }

    pub fn eval_mu(&mut self, _binder: &str, _body: &Box<LSTerm>) -> Result<Value, String> {
        // TODO: implement mu expression
        unimplemented!()
    }
}
