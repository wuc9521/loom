use derive_more::Display;

#[derive(Debug, Display)]
pub enum DelisParseError {
    #[display("[ERROR] Pest parser error: {}", _0)]
    PestError(Box<pest::error::Error<crate::parser::Rule>>),

    #[display("[ERROR] Invalid syntax: {}", _0)]
    InvalidSyntax(String),

    #[display("[ERROR] Unsupported feature: {}", _0)]
    UnsupportedFeature(String),
}

impl From<pest::error::Error<crate::parser::Rule>> for DelisParseError {
    fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
        DelisParseError::PestError(Box::new(err))
    }
}
