#[derive(pest_derive::Parser)]
#[grammar = "parser/sygus.v2.pest"] // relative to src
#[grammar = "parser/delis.pest"] // relative to src
pub struct DelisParser;

pub mod ast;
pub mod error;

pub use ast::*;
pub use error::*;
