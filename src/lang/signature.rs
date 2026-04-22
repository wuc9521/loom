use crate::parser::{DelisParseError, Rule};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::Sort;

#[derive(Clone, Debug, Display, PartialEq)]
// FuncDecl = { "(" ~ "declare-fun" ~ Symbol ~ "(" ~ Sort* ~ ")" ~ Sort ~ ")" }
#[display(
    "(declare-fun {} ({}) {})",
    name,
    arg_sorts.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" "),
    ret_sort
)]
pub struct FuncSig {
    pub name: String,
    pub arg_sorts: Vec<Sort>,
    pub ret_sort: Sort,
}

impl FuncSig {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner = pair.into_inner();
        let func_name = inner
            .next()
            .ok_or_else(|| {
                DelisParseError::InvalidSyntax("declare-fun expects a function name".to_string())
            })?
            .as_str()
            .to_string();
        let mut sort_list: Vec<Sort> = Vec::new();
        for sort in inner {
            if sort.as_rule() == Rule::Sort {
                let sort = Sort::from_str(sort.as_str())
                    .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?;
                sort_list.push(sort);
            }
        }
        let ret_sort = sort_list.pop().ok_or_else(|| {
            DelisParseError::InvalidSyntax("declare-fun expects a return sort".to_string())
        })?;
        Ok(FuncSig {
            name: func_name,
            arg_sorts: sort_list,
            ret_sort,
        })
    }
}
