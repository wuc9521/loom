#[derive(pest_derive::Parser)]
#[grammar = "smt-lib/smt-lib.v2.pest"] // relative to src
pub struct SMTLibV2Parser;