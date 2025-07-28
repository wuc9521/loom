use crate::lang::{
    func::FuncDefRec,
    model::{MatchCase, Model, ThSpecCmd},
    signature::FuncSig,
};
use crate::parser::{LSParseError, LSParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use sygus_parser::{Attribute, Identifier, Literal, Sort, SortedVar, SyGuSCmd, SyGuSTerm::*};
use log::error;

#[derive(Clone, Debug, Display)]
#[display(fmt = "({} {})", name, term)]
pub struct LSVarBinding {
    pub name: String,
    pub term: LSTerm,
}

impl LSVarBinding {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let term = LSTerm::parse(inner.next().unwrap())?;
        Ok(Self { name, term })
    }
}

#[derive(Clone, Debug, Display)]
pub enum LSTerm {
    #[display(fmt = "{}", _0)]
    Identifier(Identifier),

    #[display(fmt = "{}", _0)]
    Literal(Literal),

    #[display(
        fmt = "({} {})",
        _0,
        "_1.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    AppTerm(Identifier, Vec<LSTerm>),

    #[display(
        fmt = "(! {} {})",
        _0,
        "_1.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    AttrTerm(Box<LSTerm>, Vec<Attribute>),

    #[display(
        fmt = "(match {} ({}))",
        term,
        "cases.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    MatchTerm {
        term: Box<LSTerm>,
        cases: Vec<MatchCase>,
    },

    #[display(
        fmt = "(lambda ({}) {})",
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        body
    )]
    LambdaTerm {
        args: Vec<SortedVar>,
        body: Box<LSTerm>,
    },

    #[display(
        fmt = "(mu ({} ({}) {}) {}) {}",
        name,
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        ret_sort,
        body,
        param
    )]
    MuTerm {
        name: String,
        args: Vec<SortedVar>,
        ret_sort: Sort,
        body: Box<LSTerm>,
        param: Box<LSTerm>,
    },

    #[display(
        fmt = "(nu ({} ({}) {}) {}) {}",
        name,
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        ret_sort,
        body,
        param
    )]
    NuTerm {
        name: String,
        args: Vec<SortedVar>,
        ret_sort: Sort,
        body: Box<LSTerm>,
        param: Box<LSTerm>,
    },

    #[display(
        fmt = "(forall ({}) {})",
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        body
    )]
    ForallTerm {
        args: Vec<SortedVar>,
        body: Box<LSTerm>,
    },

    #[display(
        fmt = "(exists ({}) {})",
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        body
    )]
    ExistsTerm {
        args: Vec<SortedVar>,
        body: Box<LSTerm>,
    },

    #[display(
        fmt = "(let ({}) {})",
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        body
    )]
    LetTerm {
        args: Vec<LSVarBinding>,
        body: Box<LSTerm>,
    },
}
impl LSTerm {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        match pair.as_rule() {
            Rule::Identifier => {
                // Identifier = { Symbol }
                Ok(LSTerm::Identifier(Identifier::Symbol(
                    pair.as_str().to_string(),
                )))
            }
            Rule::Literal => {
                // Literal = { SpecConstant | Decimal | HexConst | BinConst | StringConst }
                Ok(LSTerm::Literal(Literal::from_str(pair.as_str())))
            }
            Rule::AppTerm => {
                // AppTerm = { "(" ~ Identifier ~ LSTerm+ ~ ")" }
                let mut inner = pair.into_inner();
                let sym = inner.next().unwrap().as_str().to_string();
                let mut args = vec![];
                while let Some(arg) = inner.next() {
                    args.push(LSTerm::parse(arg)?);
                }
                let app_term = LSTerm::AppTerm(Identifier::Symbol(sym), args);  
                Ok(app_term)
            }
            Rule::AttrTerm => {
                // AttrTerm = { "(" ~ "!" ~ LSTerm ~ Attribute+ ~ ")" }
                let mut inner = pair.into_inner();
                let term = LSTerm::parse(inner.next().unwrap())?;
                let mut attrs = vec![];
                while let Some(attr) = inner.next() {
                    attrs.push(Attribute::from_str(attr.as_str()).unwrap());
                }
                Ok(LSTerm::AttrTerm(Box::new(term), attrs))
            }
            Rule::LSTerm => {
                // If we get an LSTerm rule, parse its inner content
                let mut inner = pair.into_inner();
                if let Some(inner_pair) = inner.next() {
                    LSTerm::parse(inner_pair)
                } else {
                    Err(LSParseError::InvalidSyntax("Empty LSTerm".to_string()))
                }
            }
            Rule::MatchTerm => {
                // MatchTerm = { "(" ~ #LSTkMatch="match" ~ LSTerm ~ "(" ~ MatchCase+ ~ ")" ~ ")" }
                let mut inner = pair.into_inner();
                let term = LSTerm::parse(inner.next().unwrap())?;
                let mut cases = vec![];
                while let Some(case) = inner.next() {
                    let case = MatchCase::parse(case)?;
                    cases.push(case);
                }
                Ok(LSTerm::MatchTerm {
                    term: Box::new(term),
                    cases,
                })
            }
            Rule::LambdaTerm => {
                // LambdaTerm = { "(" ~ #LSTkLambda="lambda" ~ "(" ~ SortedVar+ ~ ")" ~ LSTerm ~ ")" }
                let mut inner = pair.into_inner();
                let mut vars = vec![];
                while let Some(var) = inner.next() {
                    if var.as_rule() == Rule::SortedVar {
                        vars.push(SortedVar::from_str(var.as_str()).unwrap());
                    }
                }
                let body = LSTerm::parse(inner.next().unwrap())?;
                Ok(LSTerm::LambdaTerm {
                    args: vars,
                    body: Box::new(body),
                })
            }
            Rule::MuTerm | Rule::NuTerm => {
                // MuTerm = { "(" ~ "(" ~ #LSTkMu="mu" ~ "(" ~ Symbol ~ "(" ~ SortedVar+ ~ ")" ~ Sort ~ ")" ~ LSTerm ~ ")" ~ LSTerm ~ ")" }
                let mut inner = pair.clone().into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let mut vars = vec![];

                // Parse sorted variables
                while let Some(next) = inner.peek() {
                    if next.as_rule() == Rule::SortedVar {
                        vars.push(SortedVar::from_str(inner.next().unwrap().as_str()).unwrap());
                    } else {
                        break;
                    }
                }

                // Parse return sort
                let ret_sort_pair = inner.next().unwrap();
                let ret_sort = Sort::from_str(ret_sort_pair.as_str()).unwrap();

                // Parse body
                let body = LSTerm::parse(inner.next().unwrap())?;

                // Parse parameter
                let param = LSTerm::parse(inner.next().unwrap())?;

                match pair.as_rule() {
                    Rule::MuTerm => Ok(LSTerm::MuTerm {
                        name,
                        args: vars,
                        ret_sort,
                        body: Box::new(body),
                        param: Box::new(param),
                    }),
                    Rule::NuTerm => Ok(LSTerm::NuTerm {
                        name,
                        args: vars,
                        ret_sort,
                        body: Box::new(body),
                        param: Box::new(param),
                    }),
                    _ => unreachable!(),
                }
            }
            Rule::ForallTerm | Rule::ExistsTerm => {
                // ForallTerm = { "(" ~ #LSTkForall="forall" ~ "(" ~ SortedVar+ ~ ")" ~ LSTerm ~ ")" }
                // ExistsTerm = { "(" ~ #LSTkExists="exists" ~ "(" ~ SortedVar+ ~ ")" ~ LSTerm ~ ")" }
                let mut inner = pair.clone().into_inner();
                let mut vars = vec![];
                while let Some(var) = inner.peek() {
                    if var.as_rule() == Rule::SortedVar {
                        vars.push(SortedVar::from_str(inner.next().unwrap().as_str()).unwrap());
                    } else {
                        break;
                    }
                }
                let body = LSTerm::parse(inner.next().unwrap())?;
                match pair.as_rule() {
                    Rule::ForallTerm => Ok(LSTerm::ForallTerm {
                        args: vars,
                        body: Box::new(body),
                    }),
                    Rule::ExistsTerm => Ok(LSTerm::ExistsTerm {
                        args: vars,
                        body: Box::new(body),
                    }),
                    _ => unreachable!(),
                }
            }
            Rule::LetTerm => {
                // LetTerm = { "(" ~ #LSTkLet="let" ~ "(" ~ LSVarBinding+ ~ ")" ~ LSTerm ~ ")" }
                let mut inner = pair.into_inner();
                let mut vars = vec![];
                while let Some(var) = inner.next() {
                    if var.as_rule() == Rule::LSVarBinding {
                        let var = LSVarBinding::parse(var)?;
                        vars.push(var);
                    } else {
                        let body = LSTerm::parse(var)?;
                        return Ok(LSTerm::LetTerm {
                            args: vars,
                            body: Box::new(body),
                        });
                    }
                }
                Err(LSParseError::InvalidSyntax(
                    "Expected LSVarBinding".to_string(),
                ))
            }
            _ => {
                unreachable!()
            }
        }
    }
}

#[derive(Clone, Debug, Display)]
pub enum LSCmd {
    #[display(fmt = "{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display(fmt = "{}", _0)]
    DeclareFunc(FuncSig),

    #[display(fmt = "{}", _0)]
    DefineModel(Model),

    #[display(fmt = "{}", _0)]
    DefineFuncRec(FuncDefRec),

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

pub fn parse(content: &str) -> Result<LSFile, LSParseError> {
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
                ls_file.push(LSCmd::DeclareFunc(sig));
            }
            Rule::ModelDef => {
                let model = Model::parse(cmd.clone()).unwrap();
                ls_file.push(LSCmd::DefineModel(model));
            }
            Rule::FuncDefRec => {
                let res = FuncDefRec::parse(cmd.clone());
                match res {
                    Ok(func) => ls_file.push(LSCmd::DefineFuncRec(func)),
                    Err(e) => error!("{}", e),
                }
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
                    "Invalid command: {}",
                    cmd.as_str()
                )))?;
            }
        }
    }
    Ok(ls_file)
}

pub fn parse_cmds(
    input: &str,
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
            Ok(LSCmd::DeclareFunc(sig))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd.clone()).unwrap();
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
            "Invalid command: {}",
            cmd.as_str()
        ))),
    }
}

pub fn parse_commands(
    input: &str,
) -> Vec<Result<LSCmd, LSParseError>> {
    match parse(input) {
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
