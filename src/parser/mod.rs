#[derive(pest_derive::Parser)]
#[grammar = "parser/sygus.v2.pest"] // relative to src
#[grammar = "parser/ls.pest"] // relative to src
pub struct LSParser;

pub mod ast;
pub mod error;

pub use ast::*;
pub use error::*;
