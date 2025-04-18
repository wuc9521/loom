use crate::lang::{
    model::Model,
    signature::{FuncSig, QVarSig},
};
use crate::parser::{LSParseError, LSParser, Rule};
use itertools::Itertools;
use pest::Parser;
use sygus_parser::SyGuSCmd;

#[derive(Clone, Debug)]
pub enum LSCmd {
    As(SyGuSCmd), // as a sygus command.
    FuncDecl(FuncSig),
    QVarDecl(QVarSig),
    ModelDef(Model),
}

// #[derive(Display)]
pub type LSFile = Vec<LSCmd>;

pub fn parse(content: &str) -> Result<LSFile, LSParseError> {
    let [file]: [_; 1] = LSParser::parse(Rule::LSProg, content)?
        .collect_vec()
        .try_into()
        .unwrap();
    let mut ls_file = LSFile::new();
    for cmd in file.into_inner().filter(|c| c.as_rule() != Rule::EOI) {
        let [cmd, ..]: [_; 1] = cmd.into_inner().collect_vec().try_into().unwrap();
        match cmd.as_rule() {
            Rule::SyGuSCmd => {
                let sygus_single_file = SyGuSCmd::from_str(cmd.as_str()).unwrap();
                ls_file.push(LSCmd::As(sygus_single_file.clone()));
            }
            Rule::FuncDecl => {
                // "(" ~ "declare-fun" ~ Symbol ~ "(" ~ Sort* ~ ")" ~ Sort ~ ")"
                ls_file.push(LSCmd::FuncDecl(FuncSig::parse(cmd.clone()).unwrap()));
            }
            Rule::QVarDecl => {
                // "(" ~ "declare-qvar" ~ Symbol ~ Sort ~ ")"
                ls_file.push(LSCmd::QVarDecl(QVarSig::parse(cmd.clone()).unwrap()));
            }
            Rule::ModelDef => {
                // "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ UnivDef* ~ FuncInterp* ~ ")"
                ls_file.push(LSCmd::ModelDef(Model::parse(cmd.clone()).unwrap()));
            }
            _ => {
                Err(LSParseError::InvalidSyntax(format!(
                    "Invalid command: {:?}",
                    cmd.as_str()
                )))?;
            }
        }
    }
    Ok(Vec::new())
}
