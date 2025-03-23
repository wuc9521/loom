use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use lalrpop_util::ParseError::*;

pub fn report_parse_error(error: ParseError<usize, Token, &str>, input: &str) {
    match error {
        InvalidToken { location } => {
            let (line, col, line_str) = byte_offset_to_position(input, location);
            eprintln!(
                "Invalid token at line {}, column {}:\n{}",
                line, col, line_str
            );
            eprintln!("{:>width$}^", "", width = col - 1); // caret under the offending char
        }
        UnrecognizedEof { location, expected } => {
            let (line, col, line_str) = byte_offset_to_position(input, location);
            eprintln!("Unexpected end of file at line {}, column {}", line, col);
            eprintln!("Expected one of: {:?}", expected);
            eprintln!("{}", line_str);
            eprintln!("{:>width$}^", "", width = col - 1);
        }
        UnrecognizedToken {
            token: (start, _tok, _end),
            expected,
        } => {
            let (line, col, line_str) = byte_offset_to_position(input, start);
            eprintln!("Unrecognized token at line {}, column {}", line, col);
            eprintln!("Expected one of: {:?}", expected);
            eprintln!("{}", line_str);
            eprintln!("{:>width$}^", "", width = col - 1);
        }
        ExtraToken {
            token: (start, _tok, _end),
        } => {
            let (line, col, line_str) = byte_offset_to_position(input, start);
            eprintln!("Extra token at line {}, column {}", line, col);
            eprintln!("{}", line_str);
            eprintln!("{:>width$}^", "", width = col - 1);
        }
        User { error } => {
            eprintln!("User error: {:?}", error);
        }
    }
}

/// Helper function: convert byte offset into (line, column, full line string)
fn byte_offset_to_position(input: &str, offset: usize) -> (usize, usize, String) {
    let mut line_num = 1;
    let mut col_num = 1;
    let mut line_start = 0;
    for (i, c) in input.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line_num += 1;
            line_start = i + 1;
            col_num = 1;
        } else {
            col_num += 1;
        }
    }

    let line_end = input[line_start..]
        .find('\n')
        .map(|idx| line_start + idx)
        .unwrap_or(input.len());

    let line_str = input[line_start..line_end].to_string();

    (line_num, col_num, line_str)
}
