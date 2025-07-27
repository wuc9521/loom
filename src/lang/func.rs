use crate::parser::ast::LSTerm;
use crate::parser::{LSParseError, Rule};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::{Sort, SortedVar};

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-fun-rec {} ({}) {} {})",
    name,
    "arg_sorts.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
    ret_sort,
    body
)]
pub struct FuncDefRec {
    pub name: String,
    pub arg_sorts: Vec<Sort>,
    pub ret_sort: Sort,
    pub body: LSTerm, // TODO: refine to LSTerm
}

impl FuncDefRec {
    // FuncDefRec = {"(" ~ #LSTkDefineFunRec="define-fun-rec" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ LSTerm ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let mut arg_sorts = vec![];
        let mut ret_sort = None;
        // Parse sorted variables
        while let Some(var_pair) = inner.next() {
            if var_pair.as_rule() == Rule::SortedVar {
                let sorted_var = SortedVar::from_str(var_pair.as_str()).unwrap();
                println!("@wuc9521 SortedVar: {}", sorted_var);
                arg_sorts.push(sorted_var.sort);
            } else if var_pair.as_rule() == Rule::Sort {
                let sort_str = var_pair.as_str().to_string();
                println!("@wuc9521 Sort: {}", sort_str);
                ret_sort = Some(Sort::from_str(&sort_str).unwrap());
                break;
            } else {
                eprintln!("Unexpected token: {}", var_pair.as_str());
                break;
            }
        }
        let body_pair = inner.next().unwrap();
        // println!("@wuc9521 Body: {}", body_pair.as_str());
        let body = LSTerm::parse(body_pair)?;
        Ok(Self {
            name,
            arg_sorts,
            ret_sort: ret_sort.unwrap(),
            body,
        })
    }
}
