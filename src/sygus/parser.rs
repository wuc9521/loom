#[derive(pest_derive::Parser)]
#[grammar = "sygus/sygus.v2.pest"] // relative to src
pub struct SyGuSV2Parser;