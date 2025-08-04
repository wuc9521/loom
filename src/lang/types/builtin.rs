use crate::lang::types::{TypeInferer, FunctionType};
use crate::parser::LSTerm;
use log::{debug, error};
use sygus_parser::{Identifier, Sort};
use crate::lang::ANY_TYPE;

impl TypeInferer {
    pub fn register_builtins(&mut self) {
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
            vec![Sort::from_name(ANY_TYPE)], // parameter can be any type
            Sort::bool(),
        );
        self.global_scope_mut()
            .register_function_type("finite".to_string(), finite_func);

        // register cardinality function
        let cardinality_func = FunctionType::new_signature(
            "card".to_string(),
            vec![Sort::from_name(ANY_TYPE)], // parameter can be any type
            Sort::int(),
        );
        self.global_scope_mut()
            .register_function_type("card".to_string(), cardinality_func);

        debug!(
            "built-in functions registered, total {} functions",
            self.global_scope().function_types.len()
        );
    }

    pub fn infer_builtin_function(&mut self, func_name: &str, args: &[LSTerm]) -> Result<Sort, String> {
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
}
