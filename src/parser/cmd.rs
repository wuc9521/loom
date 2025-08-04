use crate::lang::types::FunctionType;
use crate::parser::{ClassDef, LSCmd, LSParseError, LSTerm, ObjectDef, Rule};
use derive_more::Display;
use pest::iterators::Pair;
use sygus_parser::Sort;
use sygus_parser::SyGuSCmd;

impl LSCmd {
    /// internal function: parse a single command and validate the constraint
    pub fn parse(cmd: Pair<'_, Rule>) -> Result<LSCmd, LSParseError> {
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

                Ok(LSCmd::As(sygus_cmd))
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
            Rule::FuncDef | Rule::FuncDefRec => {
                // TODO: don't register non-recursve functions at declaration
                let func_type = FunctionType::parse(cmd.clone()).map_err(|e| {
                    LSParseError::InvalidSyntax(format!(
                        "Failed to parse function definition: {}",
                        e
                    ))
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
                Ok(LSCmd::ThSpec(ThSpecCmd {
                    name: symbol,
                    args,
                    sexpr,
                }))
            }
            Rule::ClassDef => {
                let class_def = ClassDef::parse(cmd)?;
                Ok(LSCmd::DefineClass(class_def))
            }
            Rule::ObjectDef => {
                let object_def = ObjectDef::parse(cmd)?;
                Ok(LSCmd::DefineObj(object_def))
            }
            // EnumSortDef = { "(" ~ #LSTkDefSortEnum="define-sort-enum" ~ Symbol ~ "(" ~ SortElem* ~ ")" ~ ")" }
            Rule::EnumSortDef => {
                let mut inner_pairs = cmd.clone().into_inner();
                let name = inner_pairs.next().unwrap().as_str().to_string();
                let elements = inner_pairs
                    .map(|p| p.as_str().to_string())
                    .collect::<Vec<_>>();
                Ok(LSCmd::DefineSortEnum {
                    name,
                    elems: elements,
                })
            }

            Rule::LSCmd => {
                let mut inner_pairs = cmd.clone().into_inner();
                if let Some(inner_cmd) = inner_pairs.next() {
                    return LSCmd::parse(inner_cmd);
                }
                Err(LSParseError::InvalidSyntax(format!("{}", cmd)))
            }
            _ => Err(LSParseError::InvalidSyntax(format!("{}", cmd))),
        }
    }
}

// ModelDef = { "(" ~ "define-model" ~ Symbol ~ (":" ~ "signature" ~ Symbol)? ~ SetUniv* ~ ")" }
#[derive(Clone, Debug, Display)]
#[display(fmt = "({} {})", name, args)]
pub struct ThSpecCmd {
    pub name: String,
    pub args: String, // may be refined to Vec<String>
    pub sexpr: Option<lexpr::Value>,
}

impl ThSpecCmd {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let args = inner.map(|p| p.as_str()).collect::<Vec<_>>().join(" ");
        let sexpr = lexpr::from_str(&format!("({} {})", name, args)).ok();
        Ok(Self { name, args, sexpr })
    }
}
