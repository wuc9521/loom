use ll_frontend::parser::*;
use ll_frontend::pretty::*;
use std::fs;
use std::io;

fn main() -> Result<(), io::Error> {
    let filename = "examples/ll/graph.smt";
    let input = fs::read_to_string(filename)?;
    match ProgramParser::new().parse(&input) {
        Ok(program) => {
            println!("Parsing successful!");
            println!("{:#?}", program);
        }
        Err(e) => {
            report_parse_error(e, &input);
        }
    }
    Ok(())
}
