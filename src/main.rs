use std::fs;
use std::path::PathBuf;
#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar); // 注意要和文件名 grammar.lalrpop 对应

fn main() {
    // 从命令行读取要解析的文件路径，或者默认解析 "example.sl"
    // 你可以用 `cargo run -- path/to/input.sl` 来指定文件
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("../examples/ll/graph.smt") // 你可以把脚本改名
    };

    let input = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Cannot read file {:?}", path));
    let parser = grammar::FileParser::new();
    match parser.parse(&input) {
        Ok(ast) => {
            println!("Parse OK! AST = {:#?}", ast);
        }
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
        }
    }
}
