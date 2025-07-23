use crate::lang::{
    model::{ModelDef, ThSpecCmd},
    signature::FuncSig,
};
use crate::parser::{LSParseError, LSParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use pest::Parser;
use sygus_parser::SyGuSCmd;

#[derive(Clone, Debug, Display)]
pub enum LSCmd {
    #[display(fmt = "{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display(fmt = "{}", _0)]
    FuncDecl(FuncSig),

    #[display(fmt = "{}", _0)]
    ModelDef(ModelDef),

    #[display(fmt = "{}", _0)]
    ThSpecCmd(ThSpecCmd),
}

#[derive(Debug, Display, Clone)]
// display every cmd
#[display(
    fmt = "{}",
    "cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\"\n\")"
)]
pub struct LSFile {
    pub cmds: Vec<LSCmd>,
}

impl LSFile {
    pub fn new() -> Self {
        LSFile { cmds: Vec::new() }
    }
    pub fn push(&mut self, cmd: LSCmd) {
        self.cmds.push(cmd);
    }
}

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
            Rule::ModelDef => {
                // "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ UnivDef* ~ FuncInterp* ~ ")"
                ls_file.push(LSCmd::ModelDef(ModelDef::parse(cmd.clone()).unwrap()));
            }
            Rule::ThSpecCmd => {
                let mut inner = cmd.clone().into_inner();
                let symbol = inner.next().unwrap().as_str().to_string();
                let args = inner.map(|p| p.as_str().to_string()).collect::<Vec<_>>().join(" ");
                ls_file.push(LSCmd::ThSpecCmd(crate::lang::model::ThSpecCmd { name: symbol, args }));
            }
            _ => {
                Err(LSParseError::InvalidSyntax(format!(
                    "Invalid command: {:?}",
                    cmd.as_str()
                )))?;
            }
        }
    }
    Ok(ls_file)
}
