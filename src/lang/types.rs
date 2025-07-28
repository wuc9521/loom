use crate::lang::model::MatchCase;
use crate::parser::ast::{LSTerm, LSVarBinding};
use derive_more::Display;
use log::{debug, error, warn};
use std::collections::HashMap;
use sygus_parser::{Identifier, Sort, SortedVar};

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

/// type environment
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub datatypes: HashMap<String, DataType>,
    pub variables: HashMap<String, Sort>,
    pub functions: HashMap<String, FuncType>,
}

/// function type
#[derive(Debug, Clone, Display)]
#[display(
    fmt = "{} -> {}",
    "params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" -> \")",
    "ret.to_string()"
)]
pub struct FuncType {
    pub params: Vec<Sort>,
    pub ret: Sort,
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

    /// create TypeEnv from multiple sygus-parser commands
    pub fn create_type_env_from_commands(cmds: &[SyGuSCmd]) -> TypeEnv {
        let mut env = TypeEnv::new();

        for cmd in cmds {
            if let Some((name, data_type)) = from_sygus_cmd(cmd) {
                env.register_datatype(name, data_type);
            }
        }

        env
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            datatypes: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// register datatype
    pub fn register_datatype(&mut self, name: String, data_type: DataType) {
        debug!("registering data type: {}", name);
        for constructor in &data_type.constructors {
            // register function for constructor.
            let param_sorts: Vec<Sort> =
                constructor.fields.iter().map(|f| f.sort.clone()).collect();
            let constructor_func = FuncType {
                params: param_sorts,
                ret: Sort::from_name(name.as_str()),
            };
            self.register_function(constructor.name.clone(), constructor_func);
            // register function for every constructor field.
            for field in &constructor.fields {
                let accessor_func = FuncType {
                    params: vec![Sort::Simple(Identifier::Symbol(name.clone()))],
                    ret: field.sort.clone(),
                };
                self.register_function(field.name.clone(), accessor_func);
            }
        }
        self.datatypes.insert(name, data_type);
    }

    /// get data type by name
    pub fn get_datatype(&self, name: &str) -> Option<&DataType> {
        self.datatypes.get(name)
    }

    /// bind variable type
    pub fn bind_variable(&mut self, name: String, sort: Sort) {
        self.variables.insert(name, sort);
    }

    /// get variable type
    pub fn get_variable_type(&self, name: &str) -> Option<&Sort> {
        self.variables.get(name)
    }

    /// register function type
    pub fn register_function(&mut self, name: String, func_type: FuncType) {
        debug!("{} :: {}", name, func_type);
        self.functions.insert(name, func_type);
    }

    /// get function type
    pub fn get_functype(&self, name: &str) -> Option<&FuncType> {
        self.functions.get(name)
    }
}

/// type inferer
#[derive(Debug)]
pub struct TypeInferer {
    pub stack: Vec<TypeEnv>, // unified scope stack, manage all type environments
}

impl TypeInferer {
    pub fn new() -> Self {
        debug!("create new TypeInferer instance");
        let mut inferer = Self {
            stack: Vec::new(),
        };
        // initialize global scope
        inferer.stack.push(TypeEnv::new());
        inferer.register_builtins();
        debug!("TypeInferer initialized");
        inferer
    }

    fn register_builtins(&mut self) {
        debug!("register built-in types and functions");

        // register built-in constants
        self.global_scope_mut()
            .bind_variable("true".to_string(), Sort::bool());
        self.global_scope_mut()
            .bind_variable("false".to_string(), Sort::bool());
        debug!("register boolean constants: true, false");

        // register built-in functions
        let and_func = FuncType {
            params: vec![Sort::bool(), Sort::bool()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("and".to_string(), and_func);

        let or_func = FuncType {
            params: vec![Sort::bool(), Sort::bool()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("or".to_string(), or_func);

        let not_func = FuncType {
            params: vec![Sort::bool()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("not".to_string(), not_func);

        let eq_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("=".to_string(), eq_func);

        let lt_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("<".to_string(), lt_func);

        let gt_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function(">".to_string(), gt_func);

        let le_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("<=".to_string(), le_func);

        let ge_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function(">=".to_string(), ge_func);

        let add_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::int(),
        };
        self.global_scope_mut()
            .register_function("+".to_string(), add_func);

        let sub_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::int(),
        };
        self.global_scope_mut()
            .register_function("-".to_string(), sub_func);

        let mul_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::int(),
        };
        self.global_scope_mut()
            .register_function("*".to_string(), mul_func);

        let div_func = FuncType {
            params: vec![Sort::int(), Sort::int()],
            ret: Sort::int(),
        };
        self.global_scope_mut()
            .register_function("/".to_string(), div_func);

        let ite_func = FuncType {
            params: vec![Sort::bool()], // simplified handling
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("ite".to_string(), ite_func);

        let distinct_func = FuncType {
            params: vec![Sort::bool()], // simplified handling
            ret: Sort::bool(),
        };
        self.global_scope_mut()
            .register_function("distinct".to_string(), distinct_func);

        debug!(
            "built-in functions registered, total {} functions",
            self.global_scope().functions.len()
        );
    }

    /// get current scope
    fn _current_scope(&self) -> &TypeEnv {
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

        // debug!("type inference result: {:?}", result);
        if let Ok(sort) = &result {
            debug!("{} :: {}", expr, sort.to_string());
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
                Ok(then_type)
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

        // infer all case types
        let mut case_types = Vec::new();
        for case in cases.iter() {
            // enter new scope to handle each case
            self.enter_scope();
            // handle variable binding in pattern matching
            if let Some(pattern) = &case.pattern {
                self.bind_pattern_variables(pattern, &term_type)?;
            }
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

    /// bind variables in pattern matching
    fn bind_pattern_variables(
        &mut self,
        pattern: &crate::lang::model::Pattern,
        term_type: &Sort,
    ) -> Result<(), String> {
        // find data type definition.
        if let Some(data_type) = self.find_datatype_by_sort(term_type) {
            // collect variables and types to bind.
            let mut bindings = Vec::new();

            // handle variable binding in pattern.
            if pattern.elements.len() >= 2 {
                // first element should be constructor name.
                if let crate::lang::model::PatternElem::Var(constructor_name) = &pattern.elements[0]
                {
                    // find constructor definition.
                    if let Some(constructor_def) = data_type
                        .constructors
                        .iter()
                        .find(|c| c.name == *constructor_name)
                    {
                        // bind constructor parameters.
                        for (i, elem) in pattern.elements.iter().skip(1).enumerate() {
                            if let crate::lang::model::PatternElem::Var(var_name) = elem {
                                if i < constructor_def.fields.len() {
                                    let field_type = constructor_def.fields[i].sort.clone();
                                    debug!("{} :: {}", var_name, field_type);
                                    bindings.push((var_name.clone(), field_type));
                                }
                            }
                        }
                    } else {
                        warn!("constructor not found: {}", constructor_name);
                    }
                }
            }

            // bind all variables at once.
            for (var_name, var_type) in bindings {
                self.current_scope_mut().bind_variable(var_name, var_type);
            }
        } else {
            warn!(
                "could not find data type for sort: {}",
                term_type.to_string()
            );
        }

        Ok(())
    }

    /// find data type by type
    fn find_datatype_by_sort(&self, sort: &Sort) -> Option<&DataType> {
        let sort_name = sort.to_string();
        self.global_scope().get_datatype(&sort_name)
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
        // otherwise, check type alias
        let t1_resolved = self.resolve_type_alias(t1);
        let t2_resolved = self.resolve_type_alias(t2);
        t1_resolved == t2_resolved
    }

    fn find_variable_type(&self, name: &str) -> Option<&Sort> {
        // lookup in scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(sort) = scope.get_variable_type(name) {
                return Some(sort);
            }
        }
        None
    }

    fn find_function_type(&self, name: &str) -> Option<&FuncType> {
        // lookup in scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(func_type) = scope.get_functype(name) {
                return Some(func_type);
            }
        }
        None
    }

    fn resolve_type_alias(&self, sort: &Sort) -> Sort {
        match sort {
            Sort::Simple(Identifier::Symbol(name)) => {
                // check type alias
                if let Some(aliased_sort) = self.find_variable_type(name) {
                    debug!("{} := {}", name, aliased_sort);
                    self.resolve_type_alias(aliased_sort)
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
        self.current_scope_mut().register_function(
            name.to_string(),
            FuncType {
                params: param_types,
                ret: ret_sort.clone(),
            },
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
}
