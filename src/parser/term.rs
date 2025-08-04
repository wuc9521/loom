use derive_more::Display;
use sygus_parser::{Attribute, Identifier, Literal, Sort, SortedVar};
use crate::parser::{LSVarBinding, MatchCase};
use crate::parser::{LSParseError, Rule};
use pest::iterators::Pair;

#[derive(Clone, Debug, Display)]
pub enum LSTerm {
    #[display(fmt = "{}", _0)]
    Identifier(Identifier),

    #[display(fmt = "{}", _0)]
    Literal(Literal),

    #[display(fmt = "{}.{}", object, method)]
    MethodCallTerm { object: String, method: String },

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
        "params.iter().map(|p| format!(\"{}\", p)).collect::<Vec<_>>().join(\" \")"
    )]
    MuTerm {
        name: String,
        args: Vec<SortedVar>,
        ret_sort: Sort,
        body: Box<LSTerm>,
        params: Vec<LSTerm>,
    },

    #[display(
        fmt = "(nu ({} ({}) {}) {}) {}",
        name,
        "args.iter().map(|s| format!(\"{}\", s)).collect::<Vec<_>>().join(\" \")",
        ret_sort,
        body,
        "params.iter().map(|p| format!(\"{}\", p)).collect::<Vec<_>>().join(\" \")"
    )]
    NuTerm {
        name: String,
        args: Vec<SortedVar>,
        ret_sort: Sort,
        body: Box<LSTerm>,
        params: Vec<LSTerm>,
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
            Rule::MethodCallTerm => {
                // MethodCallTerm = { Symbol ~ "." ~ Symbol }
                let mut parts = pair.as_str().split('.');
                let object = parts.next().unwrap().to_string();
                let method = parts.next().unwrap().to_string();
                Ok(LSTerm::MethodCallTerm { object, method })
            }
            Rule::AppTerm => {
                // AppTerm = { "(" ~ MethodCall ~ LSTerm* ~ ")" | "(" ~ Identifier ~ LSTerm+ ~ ")" }
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();

                match first.as_rule() {
                    Rule::MethodCall => {
                        // Method call: (object.method args...)
                        let method_call = first.as_str();
                        let mut parts = method_call.split('.');
                        let object = parts.next().unwrap().to_string();
                        let method = parts.next().unwrap().to_string();

                        let mut args = vec![];
                        while let Some(arg) = inner.next() {
                            args.push(LSTerm::parse(arg)?);
                        }

                        // For method calls, we'll use AppTerm with a special identifier format
                        let method_id = Identifier::Symbol(format!("{}.{}", object, method));
                        Ok(LSTerm::AppTerm(method_id, args))
                    }
                    Rule::Identifier => {
                        // Regular function call: (func args...)
                        let sym = first.as_str().to_string();
                        let mut args = vec![];
                        while let Some(arg) = inner.next() {
                            args.push(LSTerm::parse(arg)?);
                        }
                        Ok(LSTerm::AppTerm(Identifier::Symbol(sym), args))
                    }
                    _ => Err(LSParseError::InvalidSyntax(format!(
                        "Expected Identifier or MethodCall, found {:?}",
                        first.as_rule()
                    ))),
                }
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

            Rule::MatchTerm => {
                // MatchTerm = { "(" ~ "match" ~ LSTerm ~ "(" ~ MatchCase+ ~ ")" ~ ")" }
                let mut inner = pair.into_inner();

                // the first one should be the expression to be matched
                let term = LSTerm::parse(inner.next().unwrap())?;

                let mut cases = vec![];

                // parse all cases
                while let Some(case_pair) = inner.next() {
                    // each case should be like (pattern value)
                    if case_pair.as_rule() == Rule::MatchCase {
                        let case = MatchCase::parse(case_pair)?;
                        cases.push(case);
                    } else {
                        unreachable!()
                    }
                }

                let match_term = LSTerm::MatchTerm {
                    term: Box::new(term),
                    cases,
                };
                Ok(match_term)
            }

            Rule::LambdaTerm => {
                // LambdaTerm = { "(" ~ #LSTkLambda="lambda" ~ "(" ~ SortedVar+ ~ ")" ~ LSTerm ~ ")" }
                let mut inner = pair.clone().into_inner();
                let mut vars = vec![];
                while let Some(var) = inner.peek() {
                    if var.as_rule() == Rule::SortedVar {
                        vars.push(SortedVar::from_str(var.as_str()).unwrap());
                        inner.next();
                    } else {
                        break;
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

                // parse sorted variables
                while let Some(next) = inner.peek() {
                    if next.as_rule() == Rule::SortedVar {
                        vars.push(SortedVar::from_str(inner.next().unwrap().as_str()).unwrap());
                    } else {
                        break;
                    }
                }

                // parse return sort
                let ret_sort_pair = inner.next().unwrap();
                let ret_sort = Sort::from_str(ret_sort_pair.as_str()).unwrap();

                // parse body
                let body = LSTerm::parse(inner.next().unwrap())?;

                // parse parameter
                let mut params = vec![];
                while let Some(param) = inner.next() {
                    params.push(LSTerm::parse(param)?);
                }

                match pair.as_rule() {
                    Rule::MuTerm => Ok(LSTerm::MuTerm {
                        name,
                        args: vars,
                        ret_sort,
                        body: Box::new(body),
                        params,
                    }),
                    Rule::NuTerm => Ok(LSTerm::NuTerm {
                        name,
                        args: vars,
                        ret_sort,
                        body: Box::new(body),
                        params,
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
            Rule::LSTerm => {
                // If we get an LSTerm rule, parse its inner content
                let mut inner = pair.into_inner();
                if let Some(inner_pair) = inner.next() {
                    LSTerm::parse(inner_pair)
                } else {
                    Err(LSParseError::InvalidSyntax("Empty LSTerm".to_string()))
                }
            }
            _ => {
                unreachable!()
            }
        }
    }
}

