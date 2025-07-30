use crate::lang::{
    model::{MatchCase, Model, ThSpecCmd},
    types::FunctionType,
};
use crate::parser::{LSParseError, LSParser, Rule};
use derive_more::Display;
use itertools::Itertools;
use log::debug;
use pest::{iterators::Pair, Parser};
use sygus_parser::{Attribute, Identifier, Literal, Sort, SortedVar, SyGuSCmd};

/// Helper function to parse arrow sorts: (-> T1 T2 ... Tn)
pub fn parse_arrow_sort(pair: Pair<'_, Rule>) -> Result<Sort, LSParseError> {
    let mut inner = pair.into_inner();
    let mut sorts = Vec::new();

    while let Some(sort_pair) = inner.next() {
        sorts.push(parse_sort(sort_pair)?);
    }

    if sorts.len() < 2 {
        return Err(LSParseError::InvalidSyntax(
            "Arrow sort must have at least 2 sorts".to_string(),
        ));
    }

    // Build nested arrow sort from right to left
    let mut result = sorts.pop().unwrap();
    while let Some(arg_sort) = sorts.pop() {
        result = Sort::Parameterized(Identifier::Symbol("->".to_string()), vec![arg_sort, result]);
    }

    Ok(result)
}

/// Helper function to parse a sort, handling arrow types
pub fn parse_sort(pair: Pair<'_, Rule>) -> Result<Sort, LSParseError> {
    // Check if this is an arrow sort
    if pair.as_str().starts_with("(->") {
        parse_arrow_sort(pair)
    } else {
        Sort::from_str(pair.as_str())
            .map_err(|e| LSParseError::InvalidSyntax(format!("Failed to parse Sort: {}", e)))
    }
}

/// Helper function to parse a sorted variable
pub fn parse_sorted_var(pair: Pair<'_, Rule>) -> Result<SortedVar, LSParseError> {
    Ok(SortedVar::from_str(pair.as_str()).unwrap())
}

/// Class field definition
#[derive(Clone, Debug, Display)]
#[display(fmt = "[{} {}]", name, sort)]
pub struct ClassField {
    pub name: String,
    pub sort: Sort,
}

/// Class method definition
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-fun {} ({}) {} ...)",
    name,
    "params.iter().map(|p| format!(\"({} {})\", p.name, p.sort)).collect::<Vec<_>>().join(\" \")",
    ret_sort
)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<SortedVar>,
    pub ret_sort: Sort,
    pub body: Option<LSTerm>,
    pub is_recursive: bool,
}

/// Class definition
#[derive(Clone, Debug, Display)]
#[display(fmt = "(define-class ({} [{}]) ...)", name, "type_params.join(\" \")")]
pub struct ClassDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<FunctionType>,
}

/// Object method definition
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-fun {} ({}) {} ...)",
    name,
    "params.iter().map(|p| format!(\"({} {})\", p.name, p.sort)).collect::<Vec<_>>().join(\" \")",
    ret_sort
)]
pub struct ObjectMethod {
    pub name: String,
    pub params: Vec<SortedVar>,
    pub ret_sort: Sort,
    pub body: LSTerm,
}

/// Object definition
#[derive(Clone, Debug, Display)]
#[display(
    fmt = "(define-object {} ({} {}) ...)",
    name,
    class_name,
    "type_args.join(\" \")"
)]
pub struct ObjectDef {
    pub name: String,
    pub class_name: String,
    pub type_args: Vec<String>,
    pub methods: Vec<ObjectMethod>,
}

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
                debug!(
                    "LSTerm::parse encountered unknown rule: {:?} with content: '{}'",
                    pair.as_rule(),
                    pair.as_str()
                );
                unreachable!()
            }
        }
    }
}

#[derive(Clone, Debug, Display)]
pub enum LSCmd {
    #[display(fmt = "(echo {})", _0)]
    Echo(LSTerm),

    #[display(fmt = "{}", _0)]
    As(SyGuSCmd), // as a sygus command.

    #[display(fmt = "{}", _0)]
    DeclareFunc(FunctionType),

    #[display(fmt = "{}", _0)]
    ModelDef(Model),

    #[display(fmt = "{}", _0)]
    DefineFunc(FunctionType),

    #[display(
        fmt = "(define-fun-rec {} ({}) {} {})",
        "_0.get_name().unwrap_or(&\"unknown\".to_string())",
        "_0.get_params().iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" \")",
        "_0.get_ret()",
        "_0.get_body().map(|b| b.to_string()).unwrap_or(\"<body>\".to_string())"
    )]
    DefineFuncRec(FunctionType),

    #[display(fmt = "{}", _0)]
    ThSpec(ThSpecCmd),

    #[display(
        fmt = "define-sort-enum {} ({})",
        "name",
        "elems.iter().map(|e| format!(\"{}\", e)).collect::<Vec<_>>().join(\" \")"
    )]
    DefineEnumSort { name: String, elems: Vec<String> },

    #[display(
        fmt = "(define-theory {} ({}))",
        name,
        "cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    DefineTheory { name: String, cmds: Vec<LSCmd> },

    #[display(fmt = "ClassDef({:?})", "_0")]
    ClassDef(ClassDef),

    #[display(fmt = "ObjectDef({:?})", "_0")]
    ObjectDef(ObjectDef),
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
        Rule::EchoCmd => {
            let mut inner = cmd.clone().into_inner();
            let term = LSTerm::parse(inner.next().unwrap())?;
            Ok(LSCmd::Echo(term))
        }
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
            let mut inner = cmd.clone().into_inner();
            let func_name = inner.next().unwrap().as_str().to_string();
            let mut sort_list: Vec<Sort> = Vec::new();
            while let Some(sort) = inner.next() {
                if sort.as_rule() == Rule::Sort {
                    let sort = Sort::from_str(sort.as_str()).unwrap();
                    sort_list.push(sort);
                }
            }
            // the last one is the return type
            let ret_sort = sort_list.pop().unwrap();
            let func_type = FunctionType::new_signature(func_name, sort_list, ret_sort);
            Ok(LSCmd::DeclareFunc(func_type))
        }
        Rule::ModelDef => {
            let model = Model::parse(cmd.clone()).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse model definition: {}", e))
            })?;
            Ok(LSCmd::ModelDef(model))
        }
        Rule::FuncDef | Rule::FuncDefRec => {
            // TODO: don't register non-recursve functions at declaration
            let func_type = FunctionType::parse(cmd.clone()).map_err(|e| {
                LSParseError::InvalidSyntax(format!("Failed to parse function definition: {}", e))
            })?;
            match cmd.as_rule() {
                Rule::FuncDef => Ok(LSCmd::DefineFunc(func_type)),
                Rule::FuncDefRec => Ok(LSCmd::DefineFuncRec(func_type)),
                _ => unreachable!(),
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
            Ok(LSCmd::ThSpec(crate::lang::model::ThSpecCmd {
                name: symbol,
                args,
                sexpr,
            }))
        }
        Rule::ClassDef => {
            let class_def = ClassDef::parse(cmd)?;
            Ok(LSCmd::ClassDef(class_def))
        }
        Rule::ObjectDef => {
            let object_def = ObjectDef::parse(cmd)?;
            Ok(LSCmd::ObjectDef(object_def))
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
        LSCmd::DeclareFunc(FunctionType::new_signature(
            name.to_string(),
            arg_sorts,
            ret_sort,
        ))
    }
}

// Parse class definition - FIXED VERSION
impl ClassDef {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let inner = pair.into_inner();
        // First element after the tagged keyword should be the class signature
        let mut name = String::new();
        let mut type_params = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        // We need to handle the class signature specially
        let mut found_signature = false;

        for item in inner {
            match item.as_rule() {
                Rule::Symbol => {
                    if !found_signature {
                        // This is the class name
                        name = item.as_str().to_string();
                    } else {
                        // These are type parameters
                        type_params.push(item.as_str().to_string());
                    }
                }
                Rule::FieldDecl => {
                    found_signature = true;
                    fields.push(ClassField::parse(item)?);
                }
                Rule::MethodDef => {
                    found_signature = true;
                    let mut inner = item.into_inner();
                    let func_type = FunctionType::parse(inner.next().unwrap())?;
                    methods.push(func_type);
                }
                _ => {
                    // Handle the signature as a whole if it comes as a single token
                    if item.as_str().contains('[') && item.as_str().contains(']') {
                        // Parse the class signature manually
                        let sig_str = item.as_str();
                        let parts: Vec<&str> = sig_str.split(&['[', ']'][..]).collect();
                        if parts.len() >= 2 {
                            name = parts[0].trim().to_string();
                            if !parts[1].trim().is_empty() {
                                type_params =
                                    parts[1].split_whitespace().map(|s| s.to_string()).collect();
                            }
                        }
                        found_signature = true;
                    }
                }
            }
        }

        if name.is_empty() {
            return Err(LSParseError::InvalidSyntax(
                "Missing class name".to_string(),
            ));
        }

        Ok(ClassDef {
            name,
            type_params,
            fields,
            methods,
        })
    }
}

// Parse class field
impl ClassField {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        // FieldDecl = { "[" ~ Symbol ~ Sort ~ "]" }
        let mut inner = pair.into_inner();

        let name = inner
            .next()
            .ok_or_else(|| {
                LSParseError::InvalidSyntax("Expected field name in FieldDecl".to_string())
            })?
            .as_str()
            .to_string();

        let sort = parse_sort(inner.next().ok_or_else(|| {
            LSParseError::InvalidSyntax("Expected field sort in FieldDecl".to_string())
        })?)?;

        Ok(ClassField { name, sort })
    }
}

// Parse object definition
impl ObjectDef {
    // ObjectDef = {  "(" ~ #LSTkDefObject="define-object" ~ Symbol ~ "(" ~ Symbol ~ Sort* ~ ")" ~ MethodDef* ~ ")"  }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        debug!("{:?}: {:?}", pair.as_rule(), pair.as_str());
        let mut inner = pair.into_inner();
        debug!("inner: {:?}", inner.as_str());
        let name = inner
            .next()
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected object name".to_string()))?
            .as_str()
            .to_string();
        debug!("name: {:?}", name);
        // Parse class instantiation: (ClassName TypeArg1 TypeArg2 ...)
        let class_inner = inner.next().ok_or_else(|| {
            LSParseError::InvalidSyntax("Expected class instantiation".to_string())
        })?;
        debug!("class_inner: {:?}", class_inner.as_str());
        let class_name = class_inner.as_str().to_string();
        let mut type_args = Vec::new();
        while let Some(sort_pair) = inner.peek() {
            if sort_pair.as_rule() == Rule::Sort {
                let sort = parse_sort(sort_pair.clone())?;
                type_args.push(sort.to_string());
                debug!("type_args: {:?}", type_args);
                inner.next();
            } else {
                debug!("break at {:?}", sort_pair.as_rule());
                break;
            }
        }
        debug!("inner: {:?}", inner.as_str());
        let mut methods = Vec::new();
        while let Some(method_pair) = inner.peek() {
            if method_pair.as_rule() == Rule::MethodDef {
                let method_impl = FunctionType::parse(method_pair.into_inner().next().unwrap())?;
                methods.push(ObjectMethod {
                    name: method_impl.get_name().unwrap().clone(),
                    params: method_impl
                        .get_params()
                        .iter()
                        .map(|p| SortedVar {
                            name: p.to_string(),
                            sort: p.clone(),
                        })
                        .collect(),
                    ret_sort: method_impl.get_ret().clone(),
                    body: method_impl.get_body().unwrap().clone(),
                });
                inner.next();
            } else {
                debug!("break at {:?}", method_pair.as_rule());
                break;
            }
        }

        Ok(ObjectDef {
            name,
            class_name,
            type_args,
            methods,
        })
    }
}
