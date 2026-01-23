use crate::lang::runtime::Runtime;
use crate::lang::{
    model::{Model, ThSpecCmd},
    signature::FuncSig,
};
use crate::parser::{DelisParseError, DelisParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use pest::Parser;
use sygus_parser::{Sort, SyGuSCmd};

#[derive(Clone, Debug, Display)]
pub enum DelisCmd {
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
pub struct DelisFile {
    pub cmds: Vec<DelisCmd>,
}

impl DelisFile {
    pub fn new() -> Self {
        DelisFile { cmds: Vec::new() }
    }
    pub fn push(&mut self, cmd: DelisCmd) {
        self.cmds.push(cmd);
    }
}

pub fn parse(
    content: &str,
    mut runtime: Option<&mut Runtime>,
) -> Result<DelisFile, DelisParseError> {
    let [file]: [_; 1] = DelisParser::parse(Rule::DelisProg, content)?
        .collect_vec()
        .try_into()
        .unwrap();
    let mut ls_file = DelisFile::new();
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
                        return Err(DelisParseError::UnsupportedFeature(format!(
                            "[PARSER] Only (models ...) and (not (models ...)) constraints are supported, got: {}",
                            cmd_str
                        )));
                    }
                }
                ls_file.push(DelisCmd::As(sygus_cmd));
            }
            Rule::FuncDecl => {
                let sig = FuncSig::parse(cmd.clone()).unwrap();
                if let Some(ref mut runtime) = runtime {
                    runtime.insert(
                        sig.name.clone(),
                        crate::lang::runtime::Symbol::Func(sig.clone()),
                    );
                }
                ls_file.push(DelisCmd::DeclareFunc(sig));
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
                ls_file.push(DelisCmd::DefineModel(model));
            }
            // ThSpecCmd's are parsed as sexpr.
            Rule::ThSpecCmd => {
                let mut inner = cmd.clone().into_inner();
                let symbol = inner.next().unwrap().as_str().to_string();
                let args = inner
                    .map(|p| p.as_str().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let sexpr = lexpr::from_str(&format!("({} {})", symbol, args)).ok();
                ls_file.push(DelisCmd::ThSpec(crate::lang::model::ThSpecCmd {
                    name: symbol,
                    args,
                    sexpr,
                }));
            }
            _ => {
                Err(DelisParseError::InvalidSyntax(format!(
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
) -> Result<DelisCmd, DelisParseError> {
    let mut pairs = DelisParser::parse(Rule::DelisCmd, input)?;
    let cmd = pairs
        .next()
        .ok_or_else(|| DelisParseError::InvalidSyntax("Empty command".to_string()))?;
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
                    return Err(DelisParseError::UnsupportedFeature(format!(
                        "[PARSER] Only (models ...) and (not (models ...)) constraints are supported, got: {}",
                        cmd_str
                    )));
                }
            }
            Ok(DelisCmd::As(sygus_cmd))
        }
        Rule::FuncDecl => {
            let sig = FuncSig::parse(cmd.clone()).unwrap();
            if let Some(ref mut runtime) = runtime {
                runtime.insert(
                    sig.name.clone(),
                    crate::lang::runtime::Symbol::Func(sig.clone()),
                );
            }
            Ok(DelisCmd::DeclareFunc(sig))
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
            Ok(DelisCmd::DefineModel(model))
        }
        Rule::ThSpecCmd => {
            let mut inner = cmd.clone().into_inner();
            let symbol = inner.next().unwrap().as_str().to_string();
            let args = inner
                .map(|p| p.as_str().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let sexpr = lexpr::from_str(&format!("({} {})", symbol, args)).ok();
            Ok(DelisCmd::ThSpec(crate::lang::model::ThSpecCmd {
                name: symbol,
                args,
                sexpr,
            }))
        }
        _ => Err(DelisParseError::InvalidSyntax(format!(
            "Invalid command: {:?}",
            cmd.as_str()
        ))),
    }
}

pub fn parse_commands(
    input: &str,
    runtime: Option<&mut Runtime>,
) -> Vec<Result<DelisCmd, DelisParseError>> {
    match parse(input, runtime) {
        Ok(ls_file) => ls_file.cmds.into_iter().map(Ok).collect(),
        Err(e) => vec![Err(e)],
    }
}

impl DelisCmd {
    pub fn declare_fun(name: &str, arg_sorts: Vec<Sort>, ret_sort: Sort) -> Self {
        DelisCmd::DeclareFunc(FuncSig {
            name: name.to_string(),
            arg_sorts,
            ret_sort,
        })
    }
}
