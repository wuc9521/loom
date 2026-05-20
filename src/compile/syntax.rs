use crate::{
    backends::smt2::ast::HornTerm,
    compile::AspectKey,
    parser::ast::TermTypes,
};
use rustc_hash::FxHashMap as HashMap;
use sygus_parser::{
    DTConsDecl, GrammarDef, GroupedRuleList, Identifier, Sort, SortedVar, SyGuSGTerm,
    SyGuSTerm,
};

pub fn term_sort() -> Sort {
    Sort::Simple(Identifier::Symbol("Term".to_string()))
}

pub fn state_name(aspect: &AspectKey) -> String {
    let mut parts = vec![aspect.relation.clone(), aspect.model.clone()];
    for (field, value) in &aspect.fields {
        parts.push(field.clone());
        parts.push(value.display());
    }
    parts.join(".")
}

#[derive(Debug, Clone)]
pub struct TermConstructor {
    pub name: String,
    pub result_sort: String,
    pub arg_sorts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TermSignature {
    pub constructors: Vec<TermConstructor>,
    sort_order: Vec<String>,
    start_sort: String,
}

impl TermSignature {
    pub fn from_declare_term_types(decl: &TermTypes) -> Result<Self, String> {
        let mut constructors = Vec::new();
        let mut sort_order = Vec::new();
        for (sort_decl, term_type_decl) in decl.sort_decls.iter().zip(&decl.term_type_decls) {
            sort_order.push(sort_decl.symbol.clone());
            for cons in &term_type_decl.constructors {
                constructors.push(TermConstructor {
                    name: cons.symbol.to_string(),
                    result_sort: sort_decl.symbol.clone(),
                    arg_sorts: cons.sorts.iter().map(|sort| sort.to_string()).collect(),
                });
            }
        }
        let start_sort = decl
            .sort_decls
            .first()
            .map(|decl| decl.symbol.clone())
            .ok_or_else(|| "declare-term-types is empty".to_string())?;
        Ok(Self {
            constructors,
            sort_order,
            start_sort,
        })
    }

    pub fn datatype_decls(&self) -> (Vec<sygus_parser::SortDecl>, Vec<Vec<DTConsDecl>>) {
        let sort_decls = self
            .sort_order
            .iter()
            .map(|name| sygus_parser::SortDecl {
                symbol: name.clone(),
                arity: 0,
            })
            .collect::<Vec<_>>();
        let decls = self
            .sort_order
            .iter()
            .map(|result_sort| {
                self.constructors
                    .iter()
                    .filter(|cons| &cons.result_sort == result_sort)
                    .map(|cons| DTConsDecl {
                        name: cons.name.clone(),
                        args: cons
                            .arg_sorts
                            .iter()
                            .enumerate()
                            .map(|(idx, sort)| SortedVar {
                                name: format!("{}_arg{idx}", &cons.name),
                                sort: Sort::from_name(sort),
                            })
                            .collect(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        (sort_decls, decls)
    }

    pub fn datatype_constructors(&self) -> Vec<DTConsDecl> {
        self.constructors
            .iter()
            .map(|cons| DTConsDecl {
                name: cons.name.clone(),
                args: cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| SortedVar {
                        name: format!("{}_arg{idx}", &cons.name),
                        sort: term_sort(),
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn parent_contexts(&self, child_sort: &str, child: HornTerm) -> Vec<HornTerm> {
        let mut out = Vec::new();
        for cons in &self.constructors {
            for (idx, arg_sort) in cons.arg_sorts.iter().enumerate() {
                if arg_sort != child_sort {
                    continue;
                }
                let args = cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(arg_idx, _)| {
                        if arg_idx == idx {
                            child.clone()
                        } else {
                            HornTerm::var(format!("__up{arg_idx}"))
                        }
                    })
                    .collect();
                out.push(HornTerm::app(&cons.name, args));
            }
        }
        out
    }

    pub fn bv_grammar_with_sorts(
        &self,
        decl: &TermTypes,
        sort_map: &HashMap<String, Sort>,
    ) -> Result<GrammarDef, String> {
        let sorted_vars = decl
            .sort_decls
            .iter()
            .map(|sort_decl| {
                let sort = sort_map
                    .get(&sort_decl.symbol)
                    .cloned()
                    .ok_or_else(|| format!("missing BV sort for {}", sort_decl.symbol))?;
                Ok::<SortedVar, String>(SortedVar {
                    name: self.grammar_symbol(&sort_decl.symbol),
                    sort,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let grouped_rule_lists = decl
            .sort_decls
            .iter()
            .zip(decl.term_type_decls.iter())
            .map(|(sort_decl, term_type_decl)| {
                let symbol = self.grammar_symbol(&sort_decl.symbol);
                let terms = term_type_decl
                    .constructors
                    .iter()
                    .map(|cons| {
                        let name = cons.symbol.to_string();
                        if cons.sorts.is_empty() {
                            self.gterm(&name, Vec::new())
                        } else {
                            self.gterm(
                                &name,
                                cons.sorts
                                    .iter()
                                    .map(|sort| self.grammar_symbol(&sort.to_string()))
                                    .collect(),
                            )
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(GroupedRuleList {
                    symbol,
                    sort: sort_map
                        .get(&sort_decl.symbol)
                        .cloned()
                        .ok_or_else(|| format!("missing BV sort for {}", sort_decl.symbol))?,
                    terms,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(GrammarDef {
            sorted_vars,
            grouped_rule_lists,
        })
    }

    pub fn bv_grammar(&self, decl: &TermTypes, bv_sort: &Sort) -> Result<GrammarDef, String> {
        let sort_map = decl
            .sort_decls
            .iter()
            .map(|sort_decl| (sort_decl.symbol.clone(), bv_sort.clone()))
            .collect::<HashMap<_, _>>();
        self.bv_grammar_with_sorts(decl, &sort_map)
    }

    pub fn grammar_symbol_for_sort(&self, sort_name: &str) -> String {
        self.grammar_symbol(sort_name)
    }

    fn grammar_symbol(&self, sort_name: &str) -> String {
        if sort_name == self.start_sort {
            "Start".to_string()
        } else {
            format!("NT.{sort_name}")
        }
    }

    fn gterm(&self, name: &str, args: Vec<String>) -> Result<SyGuSGTerm, String> {
        let id = Identifier::from_str(name)
            .map_err(|e| format!("invalid grammar identifier {name}: {e}"))?;
        if args.is_empty() {
            return Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Identifier(id)));
        }
        Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(
            id,
            args.into_iter()
                .map(|arg| {
                    Identifier::from_str(&arg)
                        .map(SyGuSTerm::Identifier)
                        .map_err(|e| format!("invalid grammar argument {arg}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )))
    }

    pub fn start_sort(&self) -> &str {
        &self.start_sort
    }

    pub fn sort_order(&self) -> &[String] {
        &self.sort_order
    }
}
