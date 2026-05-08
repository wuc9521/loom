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
use sygus_parser::{Identifier, Sort, SortDecl, SyGuSCmd, SyGuSTerm};

#[derive(Clone, Debug, Display)]
#[display(
    "({}{})",
    symbol,
    if sorts.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            sorts
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
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
    "({})",
    constructors.iter().map(|cons| cons.to_string()).collect::<Vec<_>>().join(" ")
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
    "(declare-term-types ({}) ({}))",
    sort_decls.iter().map(|decl| decl.to_string()).collect::<Vec<_>>().join(" "),
    term_type_decls.iter().map(|decl| decl.to_string()).collect::<Vec<_>>().join(" ")
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
#[display("(define-finite {}{})", name, format_sexpr_body(body))]
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
        let mut body = Vec::new();
        for item in inner {
            match item.as_rule() {
                Rule::FiniteRecord => body.push(item.as_str().to_string()),
                Rule::WhereClause => {
                    let expr = item
                        .into_inner()
                        .next()
                        .ok_or_else(|| {
                            DelisParseError::InvalidSyntax(
                                "define-finite :where expects an expression".to_string(),
                            )
                        })?
                        .as_str()
                        .to_string();
                    body.push(":where".to_string());
                    body.push(expr);
                }
                _ => {
                    return Err(DelisParseError::InvalidSyntax(format!(
                        "Unexpected define-finite body item: {:?}",
                        item.as_str()
                    )));
                }
            }
        }
        Ok(Self { name, body })
    }
}

#[derive(Clone, Debug, Display)]
#[display(
    "(define-semantics {}{}{})",
    format_sexpr_body(std::slice::from_ref(&header)),
    return_sort
        .as_ref()
        .map(|sort| format!(" :returns {sort}"))
        .unwrap_or_default(),
    format_sexpr_body(std::slice::from_ref(&body))
)]
pub struct Semantics {
    pub header: String,
    pub body: String,
    pub return_sort: Option<Sort>,
}

impl Semantics {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, DelisParseError> {
        let mut header = None;
        let mut body = None;
        let mut return_sort = None;
        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::SemanticsHeader => header = Some(item.as_str().to_string()),
                Rule::ReturnsClause => {
                    let sort = item.into_inner().next().ok_or_else(|| {
                        DelisParseError::InvalidSyntax(
                            "define-semantics :returns expects a sort".to_string(),
                        )
                    })?;
                    return_sort = Some(
                        Sort::from_str(sort.as_str())
                            .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?,
                    );
                }
                Rule::SemanticsBody => body = Some(item.as_str().to_string()),
                _ => {
                    return Err(DelisParseError::InvalidSyntax(format!(
                        "Unexpected define-semantics item: {:?}",
                        item.as_str()
                    )));
                }
            }
        }
        Ok(Self {
            header: header.ok_or_else(|| {
                DelisParseError::InvalidSyntax(
                    "define-semantics expects a relation header".to_string(),
                )
            })?,
            body: body.ok_or_else(|| {
                DelisParseError::InvalidSyntax("define-semantics expects a match body".to_string())
            })?,
            return_sort,
        })
    }
}

#[derive(Clone, Debug, Display)]
pub enum DelisCmd {
    #[display("{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display("{}", _0)]
    TermTypes(TermTypes),

    #[display("{}", _0)]
    FiniteAspect(FiniteAspect),

    #[display("{}", _0)]
    Semantics(Semantics),

    #[display("(declare-axiom {})", _0)]
    DeclareAxiom(sygus_parser::SyGuSTerm),

    #[display("{}", _0)]
    DeclareFunc(FuncSig),

    #[display("{}", _0)]
    DefineModel(Model),
}

#[derive(Debug, Display, Clone)]
// display every cmd
#[display(
    "{}",
    cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\n")
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

impl Default for DelisProg {
    fn default() -> Self {
        Self::new()
    }
}

fn update_runtime(runtime: &mut Option<&mut Runtime>, cmd: &DelisCmd) {
    let Some(runtime) = runtime.as_deref_mut() else {
        return;
    };
    match cmd {
        DelisCmd::DeclareFunc(sig) => {
            runtime.insert(sig.name.clone(), Symbol::Func(sig.clone()));
        }
        DelisCmd::DefineModel(model) => {
            runtime.insert(model.name.clone(), Symbol::Model(model.clone()));
            for univ in &model.univ_defs {
                runtime.insert(univ.sort.to_string(), Symbol::Univ(univ.clone()));
            }
        }
        _ => {}
    }
}

fn parse_delis_cmd_pair(cmd: Pair<'_, Rule>) -> Result<DelisCmd, DelisParseError> {
    match cmd.as_rule() {
        Rule::SyGuSCmd => {
            let cmd_str = cmd.as_str();
            let sygus_cmd = SyGuSCmd::from_str(cmd_str)
                .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?;
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
        Rule::DeclareAxiom => {
            let mut inner = cmd.into_inner();
            let term = inner
                .next()
                .ok_or_else(|| {
                    DelisParseError::InvalidSyntax(
                        "declare-axiom expects a boolean SyGuS term".to_string(),
                    )
                })?
                .as_str();
            let parsed = SyGuSTerm::from_str(term)
                .map_err(|e| DelisParseError::InvalidSyntax(e.to_string()))?;
            Ok(DelisCmd::DeclareAxiom(parsed))
        }
        Rule::FuncDecl => {
            let sig = FuncSig::parse(cmd)?;
            Ok(DelisCmd::DeclareFunc(sig))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd)?;
            Ok(DelisCmd::DefineModel(model))
        }
        _ => Err(DelisParseError::InvalidSyntax(format!(
            "Invalid command: {:?}",
            cmd.as_str()
        ))),
    }
}

pub fn parse(
    content: &str,
    mut runtime: Option<&mut Runtime>,
) -> Result<DelisProg, DelisParseError> {
    let mut parsed = DelisParser::parse(Rule::DelisProg, content)?;
    let file = parsed.next().ok_or_else(|| {
        DelisParseError::InvalidSyntax("parser returned empty Delis program".to_string())
    })?;
    let mut ls_file = DelisProg::new();
    for wrapper in file.into_inner().filter(|c| c.as_rule() != Rule::EOI) {
        let cmd = wrapper.into_inner().next().ok_or_else(|| {
            DelisParseError::InvalidSyntax("empty command wrapper in DelisProg".to_string())
        })?;
        let parsed_cmd = parse_delis_cmd_pair(cmd)?;
        update_runtime(&mut runtime, &parsed_cmd);
        ls_file.push(parsed_cmd);
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
    let parsed_cmd = parse_delis_cmd_pair(cmd)?;
    update_runtime(&mut runtime, &parsed_cmd);
    Ok(parsed_cmd)
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
                assert_eq!(cmd.header, "((Regex.Sem (r Regex) (asp ReAsp)))");
                assert_eq!(cmd.body, "((match r ((a) true)))");
                assert_eq!(cmd.return_sort, None);
            }
            other => panic!("expected define-semantics, got {other:?}"),
        }
    }

    #[test]
    fn parses_define_semantics_returns() {
        let cmd = parse_command(
            "(define-semantics ((Term.Sem (t Term) (asp EnvAsp))) :returns U ((match t ((x) (x asp)))))",
            None,
        )
        .expect("define-semantics :returns should parse");

        match cmd {
            DelisCmd::Semantics(cmd) => {
                assert_eq!(cmd.header, "((Term.Sem (t Term) (asp EnvAsp)))");
                assert_eq!(
                    cmd.return_sort.as_ref().map(ToString::to_string),
                    Some("U".to_string())
                );
            }
            other => panic!("expected define-semantics, got {other:?}"),
        }
    }

    #[test]
    fn parses_declare_axiom() {
        let cmd = parse_command("(declare-axiom (= (re.opt x) (re.union re.eps x)))", None)
            .expect("declare-axiom should parse");

        match cmd {
            DelisCmd::DeclareAxiom(term) => {
                assert_eq!(term.to_string(), "(= (re.opt x) (re.union re.eps x))");
            }
            other => panic!("expected declare-axiom, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unsupported_define_fun_rec() {
        let err = parse_command("(define-fun-rec f ((x Int)) Int x)", None)
            .expect_err("define-fun-rec is not a supported Delis command");

        assert!(err.to_string().contains("Pest parser error"));
    }

    #[test]
    fn rejects_malformed_define_semantics_shape() {
        let err = parse_command("(define-semantics ((Regex.Sem (r Regex))))", None)
            .expect_err("define-semantics must include a match body");

        assert!(err.to_string().contains("Pest parser error"));
    }

    #[test]
    fn rejects_malformed_define_finite_shape() {
        let err = parse_command("(define-finite ReAsp (not-a-record (i Int)))", None)
            .expect_err("define-finite must include a record body");

        assert!(err.to_string().contains("Pest parser error"));
    }
}
