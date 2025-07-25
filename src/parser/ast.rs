use crate::lang::runtime::Runtime;
use crate::lang::{
    model::{Model, ThSpecCmd},
    signature::FuncSig,
};
use crate::parser::{LSParseError, LSParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use pest::Parser;
use sygus_parser::{Sort, SyGuSCmd};

#[derive(Clone, Debug, Display)]
pub enum LSCmd {
    #[display(fmt = "{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display(fmt = "{}", _0)]
    DeclareFunc(FuncSig),

    #[display(fmt = "{}", _0)]
    DefineModel(Model),

    #[display(fmt = "{}", _0)]
    ThSpec(ThSpecCmd),
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

pub fn parse(content: &str, mut runtime: Option<&mut Runtime>) -> Result<LSFile, LSParseError> {
    let [file]: [_; 1] = LSParser::parse(Rule::LSProg, content)?
        .collect_vec()
        .try_into()
        .unwrap();
    let mut ls_file = LSFile::new();
    // first scan: record set-logic
    let mut cmds_raw = vec![];
    for cmd in file.into_inner().filter(|c| c.as_rule() != Rule::EOI) {
        let [cmd, ..]: [_; 1] = cmd.clone().into_inner().collect_vec().try_into().unwrap();
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
                let sig = FuncSig::parse(cmd.clone()).unwrap();
                if let Some(ref mut runtime) = runtime {
                    runtime.insert(
                        sig.name.clone(),
                        crate::lang::runtime::Symbol::Func(sig.clone()),
                    );
                }
                ls_file.push(LSCmd::DeclareFunc(sig));
            }
            Rule::ModelDef => {
                let model = Model::parse(cmd.clone()).unwrap();
                if let Some(ref mut runtime) = runtime {
                    runtime.insert(
                        model.name.clone(),
                        crate::lang::runtime::Symbol::Model(model.clone()),
                    );
                    for univ in &model.univ_defs {
                        runtime.insert(
                            univ.sort.to_string(),
                            crate::lang::runtime::Symbol::Univ(univ.clone()),
                        );
                    }
                }
                ls_file.push(LSCmd::DefineModel(model));
            }
            Rule::ThSpecCmd => {
                let mut inner = cmd.clone().into_inner();
                let symbol = inner.next().unwrap().as_str().to_string();
                let args = inner
                    .map(|p| p.as_str().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let sexpr = lexpr::from_str(&format!("({} {})", symbol, args)).ok();
                ls_file.push(LSCmd::ThSpec(crate::lang::model::ThSpecCmd {
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

pub fn parse_command(
    input: &str,
    mut runtime: Option<&mut Runtime>,
) -> Result<LSCmd, LSParseError> {
    let mut pairs = LSParser::parse(Rule::LSCmd, input)?;
    let cmd = pairs
        .next()
        .ok_or_else(|| LSParseError::InvalidSyntax("Empty command".to_string()))?;
    match cmd.as_rule() {
        Rule::SyGuSCmd => {
            let cmd_str = cmd.as_str();
            let sygus_cmd = SyGuSCmd::from_str(cmd_str).unwrap();
            if let SyGuSCmd::Constraint(ref term) = sygus_cmd {
                use sygus_parser::SyGuSTerm::*;
                let supported = match term {
                    Application(sym, args) if sym.to_string() == "models" && args.len() >= 1 => {
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
            Ok(LSCmd::As(sygus_cmd))
        }
        Rule::FuncDecl => {
            let sig = FuncSig::parse(cmd.clone()).unwrap();
            if let Some(ref mut runtime) = runtime {
                runtime.insert(
                    sig.name.clone(),
                    crate::lang::runtime::Symbol::Func(sig.clone()),
                );
            }
            Ok(LSCmd::DeclareFunc(sig))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd.clone()).unwrap();
            if let Some(ref mut runtime) = runtime {
                runtime.insert(
                    model.name.clone(),
                    crate::lang::runtime::Symbol::Model(model.clone()),
                );
                for univ in &model.univ_defs {
                    runtime.insert(
                        univ.sort.to_string(),
                        crate::lang::runtime::Symbol::Univ(univ.clone()),
                    );
                }
            }
            Ok(LSCmd::DefineModel(model))
        }
        Rule::ThSpecCmd => {
            let mut inner = cmd.clone().into_inner();
            let symbol = inner.next().unwrap().as_str().to_string();
            let args = inner
                .map(|p| p.as_str().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let sexpr = lexpr::from_str(&format!("({} {})", symbol, args)).ok();
            Ok(LSCmd::ThSpec(crate::lang::model::ThSpecCmd {
                name: symbol,
                args,
                sexpr,
            }))
        }
        _ => Err(LSParseError::InvalidSyntax(format!(
            "Invalid command: {:?}",
            cmd.as_str()
        ))),
    }
}

pub fn parse_commands(
    input: &str,
    runtime: Option<&mut Runtime>,
) -> Vec<Result<LSCmd, LSParseError>> {
    match parse(input, runtime) {
        Ok(ls_file) => ls_file.cmds.into_iter().map(Ok).collect(),
        Err(e) => vec![Err(e)],
    }
}

impl LSCmd {
    pub fn declare_fun(name: &str, arg_sorts: Vec<Sort>, ret_sort: Sort) -> Self {
        LSCmd::DeclareFunc(FuncSig {
            name: name.to_string(),
            arg_sorts,
            ret_sort,
        })
    }
}
