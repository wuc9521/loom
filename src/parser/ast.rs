use crate::lang::{
    func::{FuncDef, FuncDefRec},
    model::{MatchCase, Model, ThSpecCmd},
    signature::FuncSig,
};
use crate::parser::{LSParseError, LSParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use sygus_parser::{Attribute, Identifier, Literal, Sort, SortedVar, SyGuSCmd};

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
    ModelDef(Model),

    #[display(fmt = "{}", _0)]
    DefineFunc(FuncDef),

    #[display(fmt = "{}", _0)]
    DefineFuncRec(FuncDefRec),

    #[display(fmt = "{}", _0)]
    ThSpec(ThSpecCmd),

    #[display(
        fmt = "define-sort-enum {} ({})",
        name,
        "elems.iter().map(|e| format!(\"{}\", e)).collect::<Vec<_>>().join(\" \")"
    )]
    DefineEnumSort { name: String, elems: Vec<String> },

    #[display(fmt = "(define-theory {} ({}))", name, "cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\" \")")]
    DefineTheory { name: String, cmds: Vec<LSCmd> },
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

/// parse a complete LS file content
pub fn parse(content: &str) -> Result<LSFile, LSParseError> {
    let [file]: [_; 1] = LSParser::parse(Rule::LSProg, content)?
        .collect_vec()
        .try_into()
        .unwrap();
    let mut ls_file = LSFile::new();

    // collect all commands
    let cmds_raw: Vec<_> = file
        .into_inner()
        .filter(|c| c.as_rule() != Rule::EOI)
        .map(|c| {
            let [cmd, ..]: [_; 1] = c.into_inner().collect_vec().try_into().unwrap();
            cmd
        })
        .collect();

    // process each command
    for cmd in cmds_raw {
        let parsed_cmd = parse_cmd(cmd)?;
        ls_file.push(parsed_cmd);
    }

    Ok(ls_file)
}

/// internal function: parse a single command and validate the constraint
pub fn parse_cmd(cmd: Pair<'_, Rule>) -> Result<LSCmd, LSParseError> {
    match cmd.as_rule() {
        Rule::SyGuSCmd => {
            let cmd_str = cmd.as_str();
            let sygus_cmd = SyGuSCmd::from_str(cmd_str).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse SyGuS command: {}", e))
            })?;

            // validate the constraint format
            if let SyGuSCmd::Constraint(ref term) = sygus_cmd {
                validate_constraint(term, cmd_str)?;
            }

            Ok(LSCmd::As(sygus_cmd))
        }
        Rule::ThDefCmd => {
            let mut inner = cmd.clone().into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let cmds = inner.map(|c| parse_cmd(c)).collect::<Result<Vec<_>, _>>()?;
            Ok(LSCmd::DefineTheory { name, cmds })
        }
        Rule::FuncDecl => {
            let sig = FuncSig::parse(cmd.clone()).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse function declaration: {}", e))
            })?;
            Ok(LSCmd::DeclareFunc(sig))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd.clone()).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse model definition: {}", e))
            })?;
            Ok(LSCmd::ModelDef(model))
        }
        Rule::FuncDef => {
            let func = FuncDef::parse(cmd.clone()).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse function definition: {}", e))
            })?;
            Ok(LSCmd::DefineFunc(func))
        }
        Rule::FuncDefRec => {
            let func = FuncDefRec::parse(cmd.clone())
                .map_err(|e| LSParseError::InvalidSyntax(format!("{}", e)))?;
            Ok(LSCmd::DefineFuncRec(func))
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
        // EnumSortDef = { "(" ~ #LSTkDefSortEnum="define-sort-enum" ~ Symbol ~ "(" ~ SortElem* ~ ")" ~ ")" }
        Rule::EnumSortDef => {
            let mut inner_pairs = cmd.clone().into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            let elements = inner_pairs
                .map(|p| p.as_str().to_string())
                .collect::<Vec<_>>();
            Ok(LSCmd::DefineEnumSort {
                name,
                elems: elements,
            })
        }
        Rule::LSCmd => {
            let mut inner_pairs = cmd.clone().into_inner();
            if let Some(inner_cmd) = inner_pairs.next() {
                return parse_cmd(inner_cmd);
            }
            Err(LSParseError::InvalidSyntax(format!("{}", cmd)))
        }
        _ => Err(LSParseError::InvalidSyntax(format!("{}", cmd))),
    }
}

/// validate the constraint format
fn validate_constraint(term: &sygus_parser::SyGuSTerm, cmd_str: &str) -> Result<(), LSParseError> {
    use sygus_parser::SyGuSTerm::*;

    let supported = match term {
        Application(sym, args) if sym.to_string() == "models" && args.len() >= 1 => true,
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

    Ok(())
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
