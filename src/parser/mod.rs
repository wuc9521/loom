pub mod smtlib {
    #[derive(pest_derive::Parser)]
    #[grammar = "parser/smtlib.v2.pest"] // relative to src
    pub struct SMTLibV2Parser;
}

pub mod sygus {
    #[derive(pest_derive::Parser)]
    #[grammar = "parser/sygus.v2.pest"] // relative to src
    pub struct SyGuSV2Parser;
}

pub mod ls {
    #[derive(pest_derive::Parser)]
    #[grammar = "parser/sygus.v2.pest"] // relative to src
    #[grammar = "parser/ls.pest"] // relative to src
    pub struct LSParser;
}
