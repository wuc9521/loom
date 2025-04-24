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
            // called `Result::unwrap()` on an `Err` value: PestError(Error { variant: ParsingError { positives: [UnivElem], negatives: [] }, location: Pos(202), line_col: Pos((10, 25)), path: None, line: "    (define-universe U ($0 $1 $2 $3 $4 $5 $6 $7))", continued_line: None, parse_attempts: None })
            LSParseError::PestError(err) => {
                write!(f, "Pest parser error: {}", err.to_string())
            }
            LSParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            LSParseError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
        }
    }
}

impl From<pest::error::Error<crate::parser::Rule>> for LSParseError {
    fn from(err: pest::error::Error<crate::parser::Rule>) -> Self {
        LSParseError::PestError(err)
    }
}
