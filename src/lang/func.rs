use crate::parser::ast::LSTerm;
use crate::parser::{LSParseError, Rule};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::{Sort, SortedVar};

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-fun-rec {} ({}) {} {})",
    name,
    "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
    ret_sort,
    body
)]
pub struct FuncDefRec {
    pub name: String,
    pub args: Vec<SortedVar>,
    pub ret_sort: Sort,
    pub body: LSTerm, // TODO: refine to LSTerm
}

impl FuncDefRec {
    // FuncDefRec = {"(" ~ #LSTkDefineFunRec="define-fun-rec" ~ Symbol ~ "(" ~ SortedVar* ~ ")" ~ Sort ~ LSTerm ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let mut args = vec![];
        let mut ret_sort = None;
        // Parse sorted variables
        while let Some(var_pair) = inner.next() {
            if var_pair.as_rule() == Rule::SortedVar {
                let sorted_var = SortedVar::from_str(var_pair.as_str()).unwrap();
                args.push(sorted_var);
            } else if var_pair.as_rule() == Rule::Sort {
                let sort_str = var_pair.as_str().to_string();
                ret_sort = Some(Sort::from_str(&sort_str).unwrap());
                break;
            } else {
                eprintln!("Unexpected token: {}", var_pair.as_str());
                break;
            }
        }
        let body_pair = inner.next().unwrap();
        let body = LSTerm::parse(body_pair)?;
        Ok(Self {
            name,
            args,
            ret_sort: ret_sort.unwrap(),
            body,
        })
    }
}
