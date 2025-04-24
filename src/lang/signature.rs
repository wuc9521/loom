use crate::parser::Rule;
use pest::iterators::Pair;
pub type Error = pest::error::Error<Rule>;
use derive_more::Display;
use sygus_parser::Sort;

#[derive(Clone, Debug, Display)]
// FuncDecl = { "(" ~ "declare-fun" ~ Symbol ~ "(" ~ Sort* ~ ")" ~ Sort ~ ")" }
#[display(
    fmt = "(declare-fun {} ({}) {})",
    name,
    "args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")",
    ret
)]
pub struct FuncSig {
    pub name: String,
    pub args: Vec<Sort>,
    pub ret: Sort,
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
            args: sort_list,
            ret: ret_sort,
        })
    }
}

#[derive(Clone, Debug, Display)]
// QVarDecl = { "(" ~ "declare-qvar" ~ Symbol ~ Sort ~ ")" }
#[display(fmt = "(declare-qvar {} {})", name, sort)]
pub struct QVarSig {
    pub name: String,
    pub sort: Sort,
}

impl QVarSig {
    pub fn new(name: &str, sort: Sort) -> Self {
        Self {
            name: name.to_owned(),
            sort,
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str();
        let sort = inner.next().unwrap();
        let sort = Sort::from_str(sort.as_str()).unwrap();
        Ok(Self::new(name, sort))
    }
}
