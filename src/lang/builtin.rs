use crate::lang::runtime::Value;
use crate::parser::ast::LSTerm;

#[derive(Debug)]
pub struct BuiltinFunctions;

impl BuiltinFunctions {
    /// evaluate a builtin function
    pub fn eval_builtin_function(
        func_name: &str,
        args: &[LSTerm],
        eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>,
    ) -> Option<Result<Value, String>> {
        match func_name {
            "+" => Some(Self::add(args, eval_fn)),
            "-" => Some(Self::subtract(args, eval_fn)),
            "*" => Some(Self::multiply(args, eval_fn)),
            "/" => Some(Self::divide(args, eval_fn)),
            "=" => Some(Self::equal(args, eval_fn)),
            "and" => Some(Self::and(args, eval_fn)),
            "or" => Some(Self::or(args, eval_fn)),
            "not" => Some(Self::not(args, eval_fn)),
            "ite" => Some(Self::ite(args, eval_fn)),
            "cond" => Some(Self::cond(args, eval_fn)),
            "true" => Some(Ok(Value::Bool(true))),
            "false" => Some(Ok(Value::Bool(false))),
            _ => None,
        }
    }

    /// builtin function implementation
    fn add(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("add requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            _ => Err("add requires integer arguments".to_string()),
        }
    }

    fn subtract(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("subtract requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            _ => Err("subtract requires integer arguments".to_string()),
        }
    }

    fn multiply(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("multiply requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            _ => Err("multiply requires integer arguments".to_string()),
        }
    }

    fn divide(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("divide requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Int(x / y))
                }
            }
            _ => Err("divide requires integer arguments".to_string()),
        }
    }

    fn equal(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("equal requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x == y)),
            (Value::String(x), Value::String(y)) => Ok(Value::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
            _ => Err("equal requires arguments of the same type".to_string()),
        }
    }

    fn not(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("not requires exactly 1 argument".to_string());
        }
        let a = eval_fn(&args[0])?;
        match a {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err("not requires boolean argument".to_string()),
        }
    }

    fn and(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("and requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x && y)),
            _ => Err("and requires boolean arguments".to_string()),
        }
    }

    fn or(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("or requires exactly 2 arguments".to_string());
        }
        let a = eval_fn(&args[0])?;
        let b = eval_fn(&args[1])?;
        match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x || y)),
            _ => Err("or requires boolean arguments".to_string()),
        }
    }

    fn ite(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("ite requires exactly 3 arguments".to_string());
        }
        let condition = eval_fn(&args[0])?;
        match condition {
            Value::Bool(true) => eval_fn(&args[1]),
            Value::Bool(false) => eval_fn(&args[2]),
            _ => Err("ite condition must be boolean".to_string()),
        }
    }

    fn cond(args: &[LSTerm], eval_fn: &mut dyn FnMut(&LSTerm) -> Result<Value, String>) -> Result<Value, String> {
        for arg in args {
            if let LSTerm::AppTerm(func, cond_args) = arg {
                if func.to_string() == "cond" && cond_args.len() == 2 {
                    let condition = eval_fn(&cond_args[0])?;
                    match condition {
                        Value::Bool(true) => return eval_fn(&cond_args[1]),
                        Value::Bool(false) => continue,
                        _ => return Err("cond condition must be boolean".to_string()),
                    }
                }
            }
        }
        Err("no matching condition found in cond".to_string())
    }
}