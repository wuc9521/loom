#[derive(pest_derive::Parser)]
#[grammar = "parser/sygus.v2.pest"] // relative to src
#[grammar = "parser/ls.pest"] // relative to src
pub struct LSParser;

pub mod ast;
pub mod cmd;
pub mod error;
pub mod file;
pub mod pattern;
pub mod term;
pub mod class;
pub mod obj;

pub use ast::*;
pub use cmd::*;
pub use error::*;
pub use pattern::*;
pub use term::*;
pub use obj::*;