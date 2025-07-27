use derive_more::Display;


#[derive(Debug, Display)]
pub enum LSParseError {
    #[display(fmt = "[ERROR] Parsing error: {}", _0)]
    PestError(pest::error::Error<crate::parser::Rule>),

    #[display(fmt = "[ERROR] Invalid syntax: {}", _0)]
    InvalidSyntax(String),

    #[display(fmt = "[ERROR] Unsupported feature: {}", _0)]
    UnsupportedFeature(String),
}

impl From<pest::error::Error<crate::parser::Rule>> for LSParseError {
    fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
        LSParseError::PestError(err)
    }
}
