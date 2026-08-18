#[derive(pest_derive::Parser)]
#[grammar = "parser/sygus.v2.pest"] // relative to src
#[grammar = "parser/loom.pest"] // relative to src
pub struct LoomParser;

pub mod ast;
pub mod error;
pub mod modules;

pub use ast::*;
pub use error::*;
pub use modules::*;
