use crate::parser::ast::LSTerm;
use crate::parser::{LSParseError, Rule};
use derive_more::Display;
use log::debug;
use pest::iterators::Pair;
use sygus_parser::{Identifier, Sort, SortedVar};

#[derive(Debug, Clone, Display)]
#[display(
    fmt = "{} :: {}",
    "self.get_name().unwrap_or(&\"<#unknown>\".to_string())",
    "self.sort.to_string()"
)]
pub struct FunctionType {
    pub name: Option<String>,
    pub param_names: Vec<String>,
    pub sort: Sort, // Curried arrow type: (-> param1 (-> param2 (-> param3 return_type)))
    pub body: Option<LSTerm>,
    pub is_constructor: bool,
}

impl FunctionType {
    /// get function name
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// get parameter types
    pub fn get_params(&self) -> Vec<Sort> {
        FunctionType::decompose_arrow_sort(&self.sort).0
    }

    /// get return type
    pub fn get_ret(&self) -> Sort {
        FunctionType::decompose_arrow_sort(&self.sort).1
    }

    /// Convert a Sort to FunctionType
    /// This handles arbitrary arrow types recursively, including:
    /// - Simple types: Int -> (() -> Int)
    /// - Arrow types: (-> Pos Int) -> (Pos -> Int)
    /// - Curried functions: (-> A (-> B (-> C D))) -> (A B C -> D)
    /// - Complex types: (-> (_ BitVec 64) Int) -> ((_ BitVec 64) -> Int)
    pub fn from_sort(name: String, sort: &Sort) -> Self {
        let (params, ret) = FunctionType::decompose_arrow_sort(sort);
        FunctionType::new_signature(name, params, ret)
    }

    /// Recursively decompose an arrow sort into parameter types and return type
    /// This handles arbitrary nesting of arrow types
    pub fn decompose_arrow_sort(sort: &Sort) -> (Vec<Sort>, Sort) {
        match sort {
            Sort::Parameterized(Identifier::Symbol(sym), args) if sym == "->" => {
                if args.len() == 2 {
                    let (mut params, ret) = FunctionType::decompose_arrow_sort(&args[1]);
                    params.insert(0, args[0].clone());
                    (params, ret)
                } else {
                    // Malformed arrow type, treat as function with no parameters
                    (vec![], sort.clone())
                }
            }
            _ => {
                // Non-arrow type, this is the return type
                (vec![], sort.clone())
            }
        }
    }

    /// Build a curried arrow type from parameter types and return type
    pub fn build_arrow_type(params: Vec<Sort>, ret: Sort) -> Sort {
        let mut result = ret;
        for param in params.into_iter().rev() {
            result = Sort::Parameterized(Identifier::Symbol("->".to_string()), vec![param, result]);
        }
        result
    }

    /// get parameter names
    pub fn get_param_names(&self) -> &[String] {
        &self.param_names
    }

    /// get function body
    pub fn get_body(&self) -> Option<&LSTerm> {
        self.body.as_ref()
    }

    /// FuncDef = {"(" ~ #LSTkDefineFun="define-fun" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ LSTerm ~ ")" }
    /// FuncDefRec = {"(" ~ #LSTkDefineFunRec="define-fun-rec" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ LSTerm ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        debug!("parse {:?}: {}", pair.as_rule(), pair.as_str());
        let inner = pair.clone();
        match pair.as_rule() {
            Rule::MethodDef => {
                return Self::parse(inner.into_inner().next().unwrap());
            }
            Rule::FuncDef => {
                let func_inner = inner.into_inner();
                return Self::parse_inner(func_inner);
            }
            Rule::FuncDefRec => {
                let func_inner = inner.into_inner();
                return Self::parse_inner(func_inner);
            }
            _ => {
                debug!("unexpected rule: {:?}", pair.as_rule());
                unreachable!();
            }
        }
    }

    fn parse_inner(mut func_inner: pest::iterators::Pairs<'_, Rule>) -> Result<Self, LSParseError> {
        // Get function name (first Symbol)
        let name = func_inner
            .next()
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected function name".to_string()))?
            .as_str()
            .to_string();
        debug!("name: {}", name);

        // Get parameters (SortedVar* inside parentheses)
        let mut args = vec![];
        let mut ret_sort = None;
        let mut body = None;

        while let Some(next) = func_inner.next() {
            debug!("processing: {:?} - {}", next.as_rule(), next.as_str());
            match next.as_rule() {
                Rule::SortedVar => {
                    let sorted_var = SortedVar::from_str(next.as_str()).unwrap();
                    debug!("arg: {:?}", &sorted_var);
                    args.push(sorted_var);
                }
                Rule::Sort => {
                    ret_sort = Some(Sort::from_str(next.as_str()).unwrap());
                    debug!("ret_sort: {:?}", ret_sort);
                }
                _ => {
                    // This should be the function body
                    body = Some(LSTerm::parse(next)?);
                    debug!("body: {}", body.as_ref().unwrap());
                    break;
                }
            }
        }

        let ret_sort = ret_sort
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected return sort".to_string()))?;

        let body =
            body.ok_or_else(|| LSParseError::InvalidSyntax("Expected function body".to_string()))?;

        Ok(FunctionType::new_definition(
            name,
            args.iter().map(|sv| sv.name.clone()).collect(),
            args.iter().map(|sv| sv.sort.clone()).collect(),
            ret_sort,
            body,
        ))
    }

    /// create function signature (only type information, no function body)
    pub fn new_signature(name: String, params: Vec<Sort>, ret: Sort) -> Self {
        Self {
            name: Some(name),
            param_names: vec![],
            sort: FunctionType::build_arrow_type(params, ret),
            body: None,
            is_constructor: false,
        }
    }

    pub fn new_constructor(name: String, params: Vec<Sort>, ret: Sort) -> Self {
        Self {
            name: Some(name),
            param_names: vec![],
            sort: FunctionType::build_arrow_type(params, ret),
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
            sort: FunctionType::build_arrow_type(params, ret),
            body: Some(body),
            is_constructor: false,
        }
    }
}

/// adapter: convert sygus-parser's datatype to our type system
pub mod adapter {
    use sygus_parser::{DTDecl, SyGuSCmd};

    pub fn from_dtdecl(name: &str, dtdecl: &DTDecl) -> super::super::basic::DataType {
        let mut constructors = vec![];
        for constructor_decl in dtdecl {
            let mut fields = vec![];
            for field_decl in &constructor_decl.args {
                fields.push(super::super::basic::Field {
                    name: field_decl.name.clone(),
                    sort: field_decl.sort.clone(),
                });
            }
            constructors.push(super::super::basic::Constructor {
                name: constructor_decl.name.clone(),
                fields,
            });
        }
        super::super::basic::DataType {
            name: name.to_string(),
            constructors,
        }
    }

    pub fn from_sygus_cmd(cmd: &SyGuSCmd) -> Option<(String, super::super::basic::DataType)> {
        match cmd {
            SyGuSCmd::DeclareDatatype(name, datatype) => {
                Some((name.clone(), from_dtdecl(name, datatype)))
            }
            _ => None,
        }
    }
} 