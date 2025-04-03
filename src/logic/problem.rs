use crate::{
    logic::{logic::Logic, model::Model, signature::Signature, synthfun::SynthFun},
    parser::ls::*,
};
use itertools::Itertools;
use pest::Parser;

pub type Error = pest::error::Error<Rule>;

pub struct LSProblem {
    pub logic: Logic,
    pub signature: Signature,
    pub models: Vec<Model>,
    pub synthfuns: Vec<SynthFun>,
}

impl LSProblem {
    pub fn parse(content: &str) -> Result<LSProblem, Error> {
        let [content]: [_; 1] = LSParser::parse(Rule::LSProg, content)?
            .collect_vec()
            .try_into()
            .unwrap();
        match content {
            Pair => {
                unimplemented!();
            }
        }
    }
}
