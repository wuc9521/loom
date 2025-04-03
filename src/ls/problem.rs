use crate::ls::parser::*;
use itertools::Itertools;

use pest::{iterators::Pair, Parser};


pub type Error = pest::error::Error<Rule>;
pub struct LSProblem {
    
}

impl LSProblem {
    pub fn parse(content :&str) -> Result<LSProblem, Error> {
        let [content]: [_; 1] = LSParser::parse(Rule::LSProg, content)?.collect_vec().try_into().unwrap();
        match content{
            Pair => {
                unimplemented!();
            }
        }
    }
}