use crate::{
    lang::{
        model::Model,
        runtime::{Runtime, Symbol},
        signature::FuncSig,
    },
    parser::{DelisParseError, DelisParser, Rule},
};
use derive_more::Display;
use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use sygus_parser::{Identifier, Sort, SortDecl, SyGuSCmd};

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "({}{})",
    symbol,
    "if sorts.is_empty() { String::new() } else { format!(\" {}\", sorts.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(\" \")) }"
)]
pub struct TermTypeConsDecl {
    pub symbol: Identifier,
    pub sorts: Vec<Sort>,
}

impl TermTypeConsDecl {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let inner_pairs = pair.into_inner().collect_vec();
        match inner_pairs.as_slice() {
            [symbol_pair, sorts @ ..] => {
                let symbol = Identifier::from_str(symbol_pair.as_str())
                    .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?;
                let sorts = sorts
                    .iter()
                    .map(|sort_pair| {
                        Sort::from_str(sort_pair.as_str())
                            .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self { symbol, sorts })
            }
            _ => Err(DelisParseError::InvalidSyntax(format!(
                "Expected TermTypeConsDecl, found: {:?}",
                inner_pairs
            ))),
        }
    }
}

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "({})",
    "constructors.iter().map(|cons| cons.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct TermTypeDecl {
    pub constructors: Vec<TermTypeConsDecl>,
}

impl TermTypeDecl {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let constructors = pair
            .into_inner()
            .map(TermTypeConsDecl::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { constructors })
    }
}

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(declare-term-types ({}) ({}))",
    "sort_decls.iter().map(|decl| decl.to_string()).collect::<Vec<_>>().join(\" \")",
    "term_type_decls.iter().map(|decl| decl.to_string()).collect::<Vec<_>>().join(\" \")"
)]
pub struct TermTypes {
    pub sort_decls: Vec<SortDecl>,
    pub term_type_decls: Vec<TermTypeDecl>,
}

impl TermTypes {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let inner_pairs = pair.into_inner().collect_vec();
        let mut sort_decls = Vec::new();
        let mut term_type_decls = Vec::new();

        for inner_pair in inner_pairs {
            match inner_pair.as_rule() {
                Rule::DelisTermSortDecl => {
                    let inner = inner_pair.into_inner().collect_vec();
                    match inner.as_slice() {
                        [symbol_pair, arity_pair] => {
                            sort_decls.push(SortDecl {
                                symbol: symbol_pair.as_str().to_string(),
                                arity: arity_pair.as_str().parse::<usize>().map_err(|e| {
                                    DelisParseError::InvalidSyntax(format!(
                                        "Invalid term type arity: {}",
                                        e
                                    ))
                                })?,
                            });
                        }
                        _ => {
                            return Err(DelisParseError::InvalidSyntax(format!(
                                "Expected DelisTermSortDecl, found: {:?}",
                                inner
                            )));
                        }
                    }
                }
                Rule::DelisTermTypeDecl => {
                    term_type_decls.push(TermTypeDecl::parse(inner_pair)?);
                }
                _ => {
                    return Err(DelisParseError::InvalidSyntax(format!(
                        "Unexpected structure in TermTypes parsing: {:?}",
                        inner_pair
                    )));
                }
            }
        }

        if sort_decls.len() != term_type_decls.len() {
            return Err(DelisParseError::InvalidSyntax(format!(
                "declare-term-types expects the same number of sort declarations and production groups, got {} and {}",
                sort_decls.len(),
                term_type_decls.len()
            )));
        }

        Ok(Self {
            sort_decls,
            term_type_decls,
        })
    }
}

fn format_sexpr_body(body: &[String]) -> String {
    if body.is_empty() {
        String::new()
    } else {
        format!(" {}", body.join(" "))
    }
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "(define-finite {}{})", name, "format_sexpr_body(body)")]
pub struct FiniteAspect {
    pub name: String,
    pub body: Vec<String>,
}

impl FiniteAspect {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut inner = pair.into_inner();
        let name = inner
            .next()
            .ok_or_else(|| {
                DelisParseError::InvalidSyntax(
                    "define-finite expects a finite domain name".to_string(),
                )
            })?
            .as_str()
            .to_string();
        let body = inner.map(|p| p.as_str().to_string()).collect();
        Ok(Self { name, body })
    }
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "(define-semantics{})", "format_sexpr_body(body)")]
pub struct Semantics {
    pub body: Vec<String>,
}

impl Semantics {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        Ok(Self {
            body: pair.into_inner().map(|p| p.as_str().to_string()).collect(),
        })
    }
}

#[derive(Clone, Debug, Display)]
pub enum DelisCmd {
    #[display(fmt = "{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display(fmt = "{}", _0)]
    TermTypes(TermTypes),

    #[display(fmt = "{}", _0)]
    FiniteAspect(FiniteAspect),

    #[display(fmt = "{}", _0)]
    Semantics(Semantics),

    #[display(fmt = "{}", _0)]
    DeclareFunc(FuncSig),

    #[display(fmt = "{}", _0)]
    DefineModel(Model),
}

#[derive(Debug, Display, Clone)]
// display every cmd
#[display(
    fmt = "{}",
    "cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\"\n\")"
)]
pub struct DelisProg {
    pub cmds: Vec<DelisCmd>,
}

impl DelisProg {
    pub fn new() -> Self {
        DelisProg { cmds: Vec::new() }
    }
    pub fn push(&mut self, cmd: DelisCmd) {
        self.cmds.push(cmd);
    }
}

pub fn parse(
    content: &str,
    mut runtime: Option<&mut Runtime>,
) -> Result<DelisProg, DelisParseError> {
    let [file]: [_; 1] = DelisParser::parse(Rule::DelisProg, content)?
        .collect_vec()
        .try_into()
        .unwrap();
    let mut ls_file = DelisProg::new();
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
                ls_file.push(DelisCmd::As(sygus_cmd));
            }
            Rule::DeclareTermTypes => {
                let decl = TermTypes::parse(cmd.clone())?;
                ls_file.push(DelisCmd::TermTypes(decl));
            }
            Rule::DefineFinite => {
                let decl = FiniteAspect::parse(cmd.clone())?;
                ls_file.push(DelisCmd::FiniteAspect(decl));
            }
            Rule::DefineSemantics => {
                let decl = Semantics::parse(cmd.clone())?;
                ls_file.push(DelisCmd::Semantics(decl));
            }
            Rule::FuncDecl => {
                let sig = FuncSig::parse(cmd.clone()).unwrap();
                if let Some(ref mut runtime) = runtime {
                    runtime.insert(sig.name.clone(), Symbol::Func(sig.clone()));
                }
                ls_file.push(DelisCmd::DeclareFunc(sig));
            }
            Rule::ModelDef => {
                let model = Model::parse(cmd.clone()).unwrap();
                if let Some(ref mut runtime) = runtime {
                    runtime.insert(model.name.clone(), Symbol::Model(model.clone()));
                    for univ in &model.univ_defs {
                        runtime.insert(univ.sort.to_string(), Symbol::Univ(univ.clone()));
                    }
                }
                ls_file.push(DelisCmd::DefineModel(model));
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
    let wrapper = pairs
        .next()
        .ok_or_else(|| DelisParseError::InvalidSyntax("Empty command".to_string()))?;
    let cmd = wrapper
        .into_inner()
        .next()
        .ok_or_else(|| DelisParseError::InvalidSyntax("Empty command".to_string()))?;
    match cmd.as_rule() {
        Rule::SyGuSCmd => {
            let cmd_str = cmd.as_str();
            let sygus_cmd = SyGuSCmd::from_str(cmd_str).unwrap();
            Ok(DelisCmd::As(sygus_cmd))
        }
        Rule::DeclareTermTypes => {
            let decl = TermTypes::parse(cmd)?;
            Ok(DelisCmd::TermTypes(decl))
        }
        Rule::DefineFinite => {
            let decl = FiniteAspect::parse(cmd)?;
            Ok(DelisCmd::FiniteAspect(decl))
        }
        Rule::DefineSemantics => {
            let decl = Semantics::parse(cmd)?;
            Ok(DelisCmd::Semantics(decl))
        }
        Rule::FuncDecl => {
            let sig = FuncSig::parse(cmd.clone()).unwrap();
            if let Some(ref mut runtime) = runtime {
                runtime.insert(sig.name.clone(), Symbol::Func(sig.clone()));
            }
            Ok(DelisCmd::DeclareFunc(sig))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd.clone()).unwrap();
            if let Some(ref mut runtime) = runtime {
                runtime.insert(model.name.clone(), Symbol::Model(model.clone()));
                for univ in &model.univ_defs {
                    runtime.insert(univ.sort.to_string(), Symbol::Univ(univ.clone()));
                }
            }
            Ok(DelisCmd::DefineModel(model))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_define_finite() {
        let cmd = parse_command(
            "(define-finite ReAsp (record (model Model) (i Int) (j Int)) :where (<= i j))",
            None,
        )
        .expect("define-finite should parse");

        match cmd {
            DelisCmd::FiniteAspect(cmd) => {
                assert_eq!(cmd.name, "ReAsp");
                assert_eq!(cmd.body.len(), 3);
                assert_eq!(cmd.body[0], "(record (model Model) (i Int) (j Int))");
                assert_eq!(cmd.body[1], ":where");
                assert_eq!(cmd.body[2], "(<= i j)");
            }
            other => panic!("expected define-finite, got {other:?}"),
        }
    }

    #[test]
    fn parses_define_semantics() {
        let cmd = parse_command(
            "(define-semantics ((Regex.Sem (r Regex) (asp ReAsp))) ((match r ((a) true))))",
            None,
        )
        .expect("define-semantics should parse");

        match cmd {
            DelisCmd::Semantics(cmd) => {
                assert_eq!(cmd.body.len(), 2);
                assert_eq!(cmd.body[0], "((Regex.Sem (r Regex) (asp ReAsp)))");
                assert_eq!(cmd.body[1], "((match r ((a) true)))");
            }
            other => panic!("expected define-semantics, got {other:?}"),
        }
    }
}
