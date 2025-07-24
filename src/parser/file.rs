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
    let mut current_logic: Option<String> = None;
    // first scan: record set-logic
    let mut cmds_raw = vec![];
    for cmd in file.into_inner().filter(|c| c.as_rule() != Rule::EOI) {
        let [cmd, ..]: [_; 1] = cmd.clone().into_inner().collect_vec().try_into().unwrap();
        if cmd.as_rule() == Rule::SyGuSCmd {
            if let SyGuSCmd::SetLogic(logic) = SyGuSCmd::from_str(cmd.as_str()).unwrap() {
                current_logic = Some(logic.to_string());
            }
        }
        cmds_raw.push(cmd);
    }
    // second scan: process commands
    for cmd in cmds_raw {
        match cmd.as_rule() {
            Rule::SyGuSCmd => {
                let cmd_str = cmd.as_str();
                let sygus_cmd = SyGuSCmd::from_str(cmd_str).unwrap();
                if let SyGuSCmd::Constraint(ref term) = sygus_cmd {
                    use sygus_parser::SyGuSTerm::*;
                    let supported = match term {
                        Application(sym, args)
                            if sym.to_string() == "models" && args.len() >= 1 =>
                        {
                            true
                        }
                        Application(sym, args) if sym.to_string() == "not" && args.len() == 1 => {
                            if let Application(models_sym, models_args) = &args[0] {
                                models_sym.to_string() == "models" && models_args.len() >= 1
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    if !supported {
                        return Err(LSParseError::UnsupportedFeature(format!(
                            "[PARSER] Only (models ...) and (not (models ...)) constraints are supported, got: {}",
                            cmd_str
                        )));
                    }
                }
                ls_file.push(LSCmd::As(sygus_cmd));
            }
            Rule::FuncDecl => {
                ls_file.push(LSCmd::FuncDecl(FuncSig::parse(cmd.clone()).unwrap()));
            }
            Rule::ModelDef => {
                let mut model = ModelDef::parse(cmd.clone()).unwrap();
                // dispatch ext_cmds to corresponding theory's handle_extcmd
                if !model.ext_cmds.is_empty() {
                    match current_logic.as_deref() {
                        Some("PML") => {
                            let mut ctx = crate::theory::pml::PMLCtx::default();
                            for thcmd in &model.ext_cmds {
                                crate::theory::pml::handle_extcmd(&mut ctx, thcmd);
                            }
                            model.ext_ctx = Some(crate::theory::Ctx::PML(ctx));
                        }
                        Some("FOL") => {
                            let mut ctx = crate::theory::fol::FOLCtx::default();
                            for thcmd in &model.ext_cmds {
                                crate::theory::fol::handle_extcmd(&mut ctx, thcmd);
                            }
                            model.ext_ctx = Some(crate::theory::Ctx::FOL(ctx));
                        }
                        // other theories can be extended
                        _ => {
                            eprintln!(
                                "Warning: Unsupported theory: {}",
                                current_logic.as_ref().unwrap()
                            );
                        }
                    }
                }
                ls_file.push(LSCmd::ModelDef(model));
            }
            Rule::ThSpecCmd => {
                let mut inner = cmd.clone().into_inner();
                let symbol = inner.next().unwrap().as_str().to_string();
                let args = inner
                    .map(|p| p.as_str().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let sexpr = lexpr::from_str(&format!("({} {})", symbol, args)).ok();
                ls_file.push(LSCmd::ThSpecCmd(crate::lang::model::ThSpecCmd {
                    name: symbol,
                    args,
                    sexpr,
                }));
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
