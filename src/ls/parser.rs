#[derive(pest_derive::Parser)]
#[grammar = "sygus/sygus.v2.pest"] // relative to src
#[grammar = "ls/ls.pest"] // relative to src
pub struct LSParser;