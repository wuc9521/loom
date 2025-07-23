use crate::parser::Rule;
use pest::iterators::Pair;
pub type Error = pest::error::Error<Rule>;
use derive_more::Display;
use sygus_parser::Sort;

#[derive(Clone, Debug, Display, PartialEq)]
// FuncDecl = { "(" ~ "declare-fun" ~ Symbol ~ "(" ~ Sort* ~ ")" ~ Sort ~ ")" }
#[display(
    fmt = "(declare-fun {} ({}) {})",
    name,
    "arg_sorts.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")",
    ret_sort
)]
pub struct FuncSig {
    pub name: String,
    pub arg_sorts: Vec<Sort>,
    pub ret_sort: Sort,
}

impl FuncSig {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();
        let func_name = inner.next().unwrap().as_str().to_string();
        let mut sort_list: Vec<Sort> = Vec::new();
        while let Some(sort) = inner.next() {
            if sort.as_rule() == Rule::Sort {
                let sort = Sort::from_str(sort.as_str()).unwrap();
                sort_list.push(sort);
            }
        }
        // the last one is the return type
        let ret_sort = sort_list.pop().unwrap();
        // ls_file.push(LSCmd::FuncDecl(func_name, sort_list, ret_sort));
        Ok(FuncSig {
            name: func_name,
            arg_sorts: sort_list,
            ret_sort,
        })
    }
}
