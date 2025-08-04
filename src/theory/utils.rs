use crate::lang::model::{Pattern, PatternElem};
use crate::lang::model::Model;
    use sygus_parser::{GrammarDef, Sort, SortedVar};
use std::collections::HashMap;
use crate::lang::runtime::ModelSymbolTable;
use crate::parser::ast::LSTerm;

pub fn build_symbol_map(domain_syms: &[String]) -> (HashMap<String, usize>, Vec<String>) {
    let mut map = HashMap::new();
    let mut rev = vec![];
    for (i, sym) in domain_syms.iter().enumerate() {
        map.insert(sym.clone(), i);
        rev.push(sym.clone());
    }
    (map, rev)
}


pub fn pattern_match(tuple: &Vec<usize>, pat: &Pattern, id2sym: &Vec<String>) -> bool {
    if pat.elements.len() != tuple.len() { return false; }
    for (i, elem) in pat.elements.iter().enumerate() {
        match elem {
            PatternElem::SortElem(e) => {
                if let Some(sym) = id2sym.get(tuple[i]) {
                    if &e.name != sym { return false; }
                } else {
                    return false;
                }
            }
            PatternElem::Wildcard => {}
            PatternElem::Var(_) => {}
        }
    }
    true
}

pub fn eval<'a>(
    _func_name: &str,
    _args: &[usize],
    _id2sym: &Vec<String>,
) -> Option<LSTerm> {
    // remove func_interps handling, because interp-fun is not actually parsed
    None
}

pub fn eval_with_table<'a>(
    _func_name: &str,
    _args: &[usize],
    _interp_table: Option<&'a ModelSymbolTable>,
    _id2sym: &Vec<String>,
) -> Option<crate::parser::ast::LSTerm> {
    // remove func_interps handling, because interp-fun is not actually parsed
    None
}

/// automatically identify universe sort, return (domain_syms, id2sym, universe_sort)
pub fn get_universe_domain_and_map(model: &crate::lang::model::Model) -> (Vec<String>, Vec<String>, String) {
    let universe_sort = model.univ_defs.get(0).map(|u| u.sort.to_string()).unwrap();
    let mut domain_syms = vec![];
    for univ in &model.univ_defs {
        if univ.sort.to_string() == universe_sort {
            domain_syms.push(univ.name.clone());
        }
    }
    domain_syms.sort();
    let (_sym2id, id2sym) = build_symbol_map(&domain_syms);
    (domain_syms, id2sym, universe_sort)
}

/// collect all unary boolean proposition names
pub fn collect_bool_props(_models: &[&Model], _arity: usize) -> Vec<String> {
    // remove func_interps handling, because interp-fun is not actually parsed
    vec![]
}

/// generate concatenated bitvector string and total length of propositions
pub fn prop_bitvecs(
    models: &[&Model],
    props: &[String],
    eval: &dyn Fn(&str, usize, &Model, &Vec<String>) -> bool,
) -> (HashMap<String, String>, usize) {
    let mut prop_bitvecs = HashMap::new();
    let mut final_bv_length = 0;
    for prop in props {
        let mut full_bitstring_list = Vec::new();
        let mut total_bits = 0;
        for model in models {
            let (domain_syms, id2sym, _universe_sort) = crate::theory::utils::get_universe_domain_and_map(model);
            let domain_size = domain_syms.len();
            for node in 0..domain_size {
                if eval(prop, node, model, &id2sym) {
                    full_bitstring_list.push('1');
                } else {
                    full_bitstring_list.push('0');
                }
            }
            total_bits += domain_size;
        }
        let bv_str: String = full_bitstring_list.iter().rev().collect();
        if final_bv_length == 0 {
            final_bv_length = total_bits;
        }
        prop_bitvecs.insert(prop.clone(), bv_str);
    }
    (prop_bitvecs, final_bv_length)
}

/// replace sort in grammarDef
pub fn replace_grammardef_sort(
    g: &Option<GrammarDef>,
    bv_len: usize,
) -> Option<GrammarDef> {
    g.as_ref().map(|grammar| {
        let mut new_g = grammar.clone();
        new_g.sorted_vars = grammar
            .sorted_vars
            .iter()
            .map(|v| SortedVar {
                name: v.name.clone(),
                sort: Sort::bitvec(bv_len)
            })
            .collect();
        new_g.grouped_rule_lists = grammar
            .grouped_rule_lists
            .iter()
            .map(|gr| {
                let mut new_gr = gr.clone();
                new_gr.sort = Sort::bitvec(bv_len);
                new_gr
            })
            .collect();
        new_g
    })
}

/// collect constraint_models (Vec<(model_name, is_true)>)
pub fn collect_constraint_models(lsfile: &crate::parser::LSFile) -> Vec<(String, bool)> {
    let mut constraint_models = Vec::new();
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::As(sygus_cmd) = cmd {
            if let sygus_parser::SyGuSCmd::Constraint(term) = sygus_cmd {
                use sygus_parser::SyGuSTerm::*;
                match term {
                    Application(sym, args) if sym.to_string() == "models" && args.len() >= 1 => {
                        if let Identifier(model_sym) = &args[0] {
                            constraint_models.push((model_sym.to_string(), true));
                        }
                    }
                    Application(sym, args) if sym.to_string() == "not" && args.len() == 1 => {
                        if let Application(models_sym, models_args) = &args[0] {
                            if models_sym.to_string() == "models" && models_args.len() >= 1 {
                                if let Identifier(model_sym) = &models_args[0] {
                                    constraint_models.push((model_sym.to_string(), false));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    constraint_models
}
