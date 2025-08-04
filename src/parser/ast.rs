use crate::parser::{LSParseError, LSTerm, Rule, ObjectDef, ThSpecCmd};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::{Identifier, Sort, SyGuSCmd};
use crate::lang::types::FunctionType;

#[derive(Debug, Display, Clone)]
// display every cmd
#[display(
    fmt = "{}",
    "cmds.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(\"\n\")"
)]
pub struct LSFile {
    pub cmds: Vec<LSCmd>,
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
        fmt = "(define-sort-enum {} ({}))",
        "name",
        "elems.iter().map(|e| format!(\"{}\", e)).collect::<Vec<_>>().join(\" \")"
    )]
    DefineSortEnum { name: String, elems: Vec<String> },

    #[display(
        fmt = "(define-class {} ({}))",
        "_0.name",
        "_0.type_params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(\" \")"
    )]
    DefineClass(ClassDef),

    #[display(fmt = "(define-object {} ({}))", "_0.name", "_0.class_name")]
    DefineObj(ObjectDef),

    #[display(fmt = "SynthFunExt({})", "_0")]
    SynthFunExt(FunctionType),
}


/// Class field definition
#[derive(Clone, Debug, Display)]
#[display(fmt = "[{} {}]", name, sort)]
pub struct ClassField {
    pub name: String,
    pub sort: Sort,
}

/// Class definition
#[derive(Clone, Debug, Display)]
#[display(fmt = "(define-class ({} [{}]) ...)", name, "type_params.join(\" \")")]
pub struct ClassDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<FunctionType>,
    pub datatypes: Vec<SyGuSCmd>,
}

/// Helper function to parse arrow sorts: (-> T1 T2 ... Tn)
pub fn parse_arrow_sort(pair: Pair<'_, Rule>) -> Result<Sort, LSParseError> {
    let mut inner = pair.into_inner();
    let mut sorts = Vec::new();

    while let Some(sort_pair) = inner.next() {
        sorts.push(Sort::from_str(sort_pair.as_str()).unwrap());
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
