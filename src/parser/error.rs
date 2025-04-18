use std::fmt;

#[derive(Debug)]
pub enum LSParseError {
    PestError(pest::error::Error<crate::parser::Rule>),
    InvalidSyntax(String),
    UnsupportedFeature(String),
}


impl fmt::Display for LSParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LSParseError::PestError(err) => write!(f, "Pest parser error: {}", err),
            LSParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            LSParseError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
        }
    }
}

impl From<pest::error::Error<crate::parser::Rule>> for LSParseError{
    fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
        LSParseError::PestError(err)
    }
}
