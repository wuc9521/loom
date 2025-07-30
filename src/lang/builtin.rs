use crate::lang::interpreter::Interpreter;
use crate::lang::runtime::Value;
use crate::parser::ast::LSTerm;

impl Interpreter {
    pub fn eval_builtin_function(
        &mut self,
        func_name: &str,
        args: &[LSTerm],
    ) -> Result<Value, String> {
        match func_name {
            "+" => self.eval_add(args),
            "-" => self.eval_sub(args),
            "*" => self.eval_mul(args),
            // "/" => self.eval_div(args),
            "=" => self.eval_eq(args),
            "<" => self.eval_lt(args),
            ">" => self.eval_gt(args),
            "<=" => self.eval_le(args),
            ">=" => self.eval_ge(args),      
            "and" => self.eval_and(args),
            "or" => self.eval_or(args),
            "not" => self.eval_not(args),
            "ite" => self.eval_ite(args),
            "finite" => self.eval_finite(args),
            "card" => self.eval_card(args),
            _ => Err(format!("'{}' is not a function", func_name)),
        }
    }

    fn eval_lt(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("less requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)),
            _ => Err("less requires integer arguments".to_string()),
        }
    }

    fn eval_le(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("less_equal requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)),
            _ => Err("less_equal requires integer arguments".to_string()),
        }
    }

    fn eval_gt(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("greater requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)),
            _ => Err("greater requires integer arguments".to_string()),
        }
    }

    fn eval_ge(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("greater_equal requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)),
            _ => Err("greater_equal requires integer arguments".to_string()),
        }
    }

    fn eval_add(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("add requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            _ => Err("add requires integer arguments".to_string()),
        }
    }

    fn eval_sub(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("subtract requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            _ => Err("subtract requires integer arguments".to_string()),
        }
    }

    fn eval_mul(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("multiply requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            _ => Err("multiply requires integer arguments".to_string()),
        }
    }

    // fn eval_div(&mut self, args: &[LSTerm]) -> Result<Value, String> {
    //     if args.len() != 2 {
    //         return Err("divide requires exactly 2 arguments".to_string());
    //     }
    //     let a = self.eval(&args[0])?;
    //     let b = self.eval(&args[1])?;
    //     match (a, b) {
    //         (Value::Int(x), Value::Int(y)) => {
    //             if y == 0 {
    //                 Err("division by zero".to_string())
    //             } else {
    //                 Ok(Value::Int(x / y))
    //             }
    //         }
    //         _ => Err("divide requires integer arguments".to_string()),
    //     }
    // }

    fn eval_eq(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("equal requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x == y)),
            (Value::String(x), Value::String(y)) => Ok(Value::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
            _ => Err("equal requires arguments of the same type".to_string()),
        }
    }

    fn eval_not(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("not requires exactly 1 argument".to_string());
        }
        let a = self.eval(&args[0])?;
        match a {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err("not requires boolean argument".to_string()),
        }
    }

    fn eval_and(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("and requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x && y)),
            _ => Err("and requires boolean arguments".to_string()),
        }
    }

    fn eval_or(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("or requires exactly 2 arguments".to_string());
        }
        let a = self.eval(&args[0])?;
        let b = self.eval(&args[1])?;
        match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x || y)),
            _ => Err("or requires boolean arguments".to_string()),
        }
    }

    fn eval_ite(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("ite requires exactly 3 arguments".to_string());
        }
        let condition = self.eval(&args[0])?;
        match condition {
            Value::Bool(true) => self.eval(&args[1]),
            Value::Bool(false) => self.eval(&args[2]),
            _ => Err("ite condition must be boolean".to_string()),
        }
    }

    fn eval_finite(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("finite requires exactly 1 argument".to_string());
        }
        let type_name = if let LSTerm::Identifier(ident) = &args[0] {
            ident.to_string()
        } else {
            return Err("finite requires a type identifier as argument".to_string());
        };
        if let Some(is_finite) = self.typing.global_scope().is_finite_by_name(&type_name) {
            Ok(Value::Bool(is_finite))
        } else {
            Err(format!("Type {} is unknown", type_name))
        }
    }

    fn eval_card(&mut self, args: &[LSTerm]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("card requires exactly 1 argument".to_string());
        }
        let type_name = if let LSTerm::Identifier(ident) = &args[0] {
            ident.to_string()
        } else {
            return Err("card requires a type identifier as argument".to_string());
        };
        if let Some(cardinality) = self.typing.global_scope().get_cardinality(&type_name) {
            Ok(Value::Int(cardinality as i64))
        } else {
            Err(format!("Type {} is infinite or unknown", type_name))
        }
    }
}
