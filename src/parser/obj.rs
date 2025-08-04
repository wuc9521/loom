use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::Sort;
use crate::parser::{LSParseError, Rule};
use crate::lang::types::FunctionType;
use log::debug;

/// Object definition
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-object {} ({} {}) ...)",
    name,
    class_name,
    "type_args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")"
)]
pub struct ObjectDef {
    pub name: String,
    pub class_name: String,
    pub type_args: Vec<Sort>,
    pub methods: Vec<FunctionType>,
}

// Parse object definition
impl ObjectDef {
    // ObjectDef = {  "(" ~ #LSTkDefObject="define-object" ~ Symbol ~ "(" ~ Symbol ~ Sort* ~ ")" ~ MethodDef* ~ ")"  }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        debug!("{:?}: {:?}", pair.as_rule(), pair.as_str());
        let mut inner = pair.into_inner();
        debug!("inner: {:?}", inner.as_str());
        let name = inner
            .next()
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected object name".to_string()))?
            .as_str()
            .to_string();
        debug!("name: {:?}", name);
        // Parse class instantiation: next should be class name, followed by type args
        let class_name = inner
            .next()
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected class name".to_string()))?
            .as_str()
            .to_string();

        let mut type_args = Vec::new();
        while let Some(sort_pair) = inner.peek() {
            if sort_pair.as_rule() == Rule::Sort {
                let sort = Sort::from_str(sort_pair.as_str()).unwrap();
                debug!(
                    "Parsed type_arg: {} for class {}",
                    sort.to_string(),
                    class_name
                );
                type_args.push(sort);
                inner.next(); // consume the sort
            } else {
                break;
            }
        }
        let mut methods = Vec::new();
        while let Some(method_pair) = inner.peek() {
            if method_pair.as_rule() == Rule::MethodDef {
                let method_impl = FunctionType::parse(method_pair.clone())?;
                methods.push(method_impl.clone());
                inner.next();
            } else {
                break;
            }
        }

        Ok(ObjectDef {
            name,
            class_name,
            type_args,
            methods,
        })
    }
}
