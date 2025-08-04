use crate::lang::types::adapter;
use crate::parser::LSCmd;
use crate::parser::{ClassDef, ClassField};
use crate::parser::{LSParseError, Rule};
use log::{debug, error};
use pest::iterators::Pair;
use sygus_parser::{Sort, SyGuSCmd};

impl ClassDef {
    // ClassDef = {  "(" ~ #LSTkDefClass="define-class" ~ "(" ~ Symbol ~ "[" ~ Symbol* ~ "]" ~ ")" ~  "(" ~FieldDecl*  ~ ")" ~ (LSCmd)* ~ ")" }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, LSParseError> {
        let mut inner = pair.into_inner();
        let name = inner
            .next()
            .ok_or_else(|| LSParseError::InvalidSyntax("Expected class name".to_string()))?
            .as_str()
            .to_string();
        debug!("name: {}", name);
        let mut type_params = vec![];
        let mut fields = vec![];
        let mut methods = vec![];
        let mut datatypes = vec![];
        while let Some(field_pair) = inner.peek() {
            match field_pair.as_rule() {
                Rule::Symbol => {
                    let type_param = field_pair.as_str().to_string();
                    type_params.push(type_param.clone());
                    debug!("type_param: {}", type_param);
                }
                Rule::FieldDecl => {
                    let field = ClassField::parse(field_pair.clone())?;
                    fields.push(field.clone());
                    debug!("field: {}", field);
                }
                Rule::LSCmd => {
                    println!("field_pair: {:?}", field_pair.as_str());
                    let cmd = LSCmd::parse(field_pair.clone())?;
                    match cmd {
                        LSCmd::DefineFunc(func) | LSCmd::DefineFuncRec(func) => {
                            methods.push(func.clone());
                        }
                        LSCmd::As(SyGuSCmd::DeclareDatatype(name, decl)) => {
                            datatypes.push(adapter::from_dtdecl(&name, &decl));
                        }
                        _ => {
                            error!("invalid cmd: {:?}", cmd);
                            return Err(LSParseError::InvalidSyntax(format!("{}", cmd)));
                        }
                    }
                }
                _ => {
                    debug!("break at {:?}", field_pair.as_rule());
                    break;
                }
            }
            inner.next();
        }
        Ok(ClassDef {
            name,
            type_params,
            fields,
            methods,
            datatypes,
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

        let sort = inner
            .next()
            .ok_or_else(|| {
                LSParseError::InvalidSyntax("Expected field sort in FieldDecl".to_string())
            })?
            .as_str()
            .to_string();
        let sort = Sort::from_str(&sort).unwrap();

        Ok(ClassField { name, sort })
    }
}
