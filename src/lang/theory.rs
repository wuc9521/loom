use crate::parser::LSFile;
use crate::theory::Logic;
use lexpr::Value;

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub name: String,
    pub arg_sorts: Vec<String>,
    pub ret_sort: String,
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SortDecl {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Clone)]
pub struct Theory {
    pub name: String,
    pub funs: Vec<FunDecl>,
    pub sorts: Vec<SortDecl>,
}

pub trait SygusDesugar {
    fn desugar(&self, sygus_file: &sygus_parser::SyGuSFile) -> LSFile;
}

impl SygusDesugar for Logic {
    fn desugar(&self, sygus_file: &sygus_parser::SyGuSFile) -> LSFile {
        match self {
            Logic::RE => crate::theory::re::sugar::desugar(sygus_file),
            _ => unimplemented!("desugar not implemented for this logic"),
        }
    }
}

fn iter_sexp_list(val: &Value) -> Vec<&Value> {
    if val.is_list() {
        val.list_iter().unwrap().collect()
    } else if let Some(slice) = val.as_slice() {
        slice.iter().collect()
    } else {
        vec![]
    }
}

impl Theory {
    pub fn from_sexp(sexp: &Value) -> Option<Self> {
        // (theory NAME ...)
        let list = sexp.to_ref_vec()?;
        if list.len() < 2 {
            return None;
        }
        if list[0].as_symbol()? != "theory" {
            return None;
        }
        let name = list[1].as_symbol()?.to_string();
        let mut funs = Vec::new();
        let mut sorts = Vec::new();
        for item in &list[2..] {
            if let Some(vec) = item.to_ref_vec() {
                if vec.len() == 2 {
                    if let Some(kw) = vec[0].as_symbol() {
                        match kw {
                            ":funs" => {
                                for f in iter_sexp_list(vec[1]) {
                                    if let Some(fdecl) = parse_fun_decl(f) {
                                        funs.push(fdecl);
                                    }
                                }
                            }
                            ":sorts" => {
                                for s in iter_sexp_list(vec[1]) {
                                    if let Some(sdecl) = parse_sort_decl(s) {
                                        sorts.push(sdecl);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Some(Theory { name, funs, sorts })
    }
}

fn parse_fun_decl(val: &Value) -> Option<FunDecl> {
    // (name arg1 arg2 ... ret_sort [attrs...])
    // last one is return type or attributes
    let list = val.to_ref_vec()?;
    if list.len() < 2 {
        return None;
    }
    let name = list[0].as_symbol()?.to_string();
    let mut arg_sorts = Vec::new();
    let mut attrs = Vec::new();
    for i in 1..list.len() - 1 {
        if let Some(s) = list[i].as_symbol() {
            arg_sorts.push(s.to_string());
        }
    }
    let last = list.last()?;
    if let Some(s) = last.as_symbol() {
        Some(FunDecl {
            name,
            arg_sorts,
            ret_sort: s.to_string(),
            attrs,
        })
    } else if let Some(sublist) = last.to_ref_vec() {
        if sublist.is_empty() {
            return None;
        }
        let ret_sort = sublist[0].as_symbol()?.to_string();
        for attr in &sublist[1..] {
            if let Some(a) = attr.as_symbol() {
                attrs.push(a.to_string());
            }
        }
        Some(FunDecl {
            name,
            arg_sorts,
            ret_sort,
            attrs,
        })
    } else {
        None
    }
}

fn parse_sort_decl(val: &Value) -> Option<SortDecl> {
    // (name arity)
    let list = val.to_ref_vec()?;
    if list.len() != 2 {
        return None;
    }
    let name = list[0].as_symbol()?.to_string();
    let arity = list[1].as_u64()? as usize;
    Some(SortDecl { name, arity })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexpr::from_str;

    #[test]
    fn test_parse_theory() {
        let sexp = r#"
        (theory FOL
         :sorts ((Int 0))
         :funs ((NUMERAL Int)
                (- Int Int)
                (- Int Int Int :left-assoc)
                (+ Int Int Int :left-assoc)
                (* Int Int Int :left-assoc)
                (div Int Int Int :left-assoc)
                (mod Int Int Int)
                (abs Int Int)
                (<= Int Int Bool :chainable)
                (<  Int Int Bool :chainable)
                (>= Int Int Bool :chainable)
                (>  Int Int Bool :chainable)))
        "#;
        let v = from_str(sexp).unwrap();
        let theory = Theory::from_sexp(&v).unwrap();
        assert_eq!(theory.name, "Ints");
        assert_eq!(theory.sorts.len(), 1);
        assert_eq!(theory.funs.len(), 12);
        assert_eq!(theory.funs[0].name, "NUMERAL");
        assert_eq!(theory.funs[0].ret_sort, "Int");
        assert_eq!(theory.funs[2].attrs[0], ":left-assoc");
    }
}
