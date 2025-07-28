use crate::lang::model::{Pattern, PatternElem};
use crate::lang::model::Model;
use sygus_parser::Sort;
use std::collections::{HashMap, HashSet};
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
            PatternElem::UnivElem(e) => {
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
    func_name: &str,
    args: &[usize],
    func_interps: &'a [crate::lang::model::FuncInterp],
    id2sym: &Vec<String>,
) -> Option<LSTerm> {
    // 0-arity: SimpleInterp
    if args.is_empty() {
        for interp in func_interps {
            if let crate::lang::model::FuncInterp::SimpleInterp(name, _, _, term) = interp {
                if name == func_name {
                    return Some(term.clone());
                }
            }
        }
    }
    // MatchInterp
    for interp in func_interps {
        if let crate::lang::model::FuncInterp::MatchInterp(name, _, _, _, cases) = interp {
            if name == func_name {
                for case in cases { // precise match
                    if let Some(pat) = &case.pattern {
                        if pattern_match(&args.to_vec(), pat, id2sym) {
                            return Some(case.value.clone());
                        }
                    }
                }
                for case in cases { // fallback (_)
                    if let Some(pat) = &case.pattern {
                        if pat.elements.iter().all(|e| matches!(e, crate::lang::model::PatternElem::Wildcard)) {
                            return Some(case.value.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn eval_with_table<'a>(
    func_name: &str,
    args: &[usize],
    interp_table: Option<&'a ModelSymbolTable>,
    func_interps: &'a [crate::lang::model::FuncInterp],
    id2sym: &Vec<String>,
) -> Option<crate::parser::ast::LSTerm> {
    // 1. first check small symbol table
    if let Some(table) = interp_table {
        if let Some(crate::lang::runtime::SymbolInterp::InterpFun(_name, val)) = table.get(func_name) {
            // TODO: only handle SimpleInterp for now
            if args.is_empty() {
                return Some(val.clone());
            }
            // TODO: implement MatchInterp for now
        }
    }
    // 2. fallback to func_interps
    eval(func_name, args, func_interps, id2sym)
}

/// automatically identify universe sort, return (domain_syms, id2sym, universe_sort)
pub fn get_universe_domain_and_map(model: &crate::lang::model::Model) -> (Vec<String>, Vec<String>, String) {
    let universe_sort = model.univ_defs.get(0).map(|u| u.sort.to_string()).unwrap();
    let mut domain_syms = vec![];
    for univ in &model.univ_defs {
        if univ.sort.to_string() == universe_sort {
            for elem in &univ.elems {
                domain_syms.push(elem.name.clone());
            }
        }
    }
    domain_syms.sort();
    let (_sym2id, id2sym) = build_symbol_map(&domain_syms);
    (domain_syms, id2sym, universe_sort)
}

/// collect all unary boolean proposition names
pub fn collect_bool_props(models: &[&Model], arity: usize) -> Vec<String> {
    let mut all_props = HashSet::new();
    for model in models {
        for interp in &model.func_interps {
            if let crate::lang::model::FuncInterp::MatchInterp(name, args, _, _, cases) = interp {
                let this_arity = args.len();
                let is_bool = cases.iter().any(|c| {
                    matches!(&c.value, LSTerm::Literal(lit) 
                        if lit.to_string() == "true" || lit.to_string() == "false")
                });
                if is_bool && this_arity == arity {
                    all_props.insert(name.clone());
                }
            }
        }
    }
    let mut all_props: Vec<_> = all_props.into_iter().collect();
    all_props.sort();
    all_props
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
    g: &Option<sygus_parser::GrammarDef>,
    bv_len: usize,
) -> Option<sygus_parser::GrammarDef> {
    g.as_ref().map(|grammar| {
        let mut new_g = grammar.clone();
        new_g.sorted_vars = grammar
            .sorted_vars
            .iter()
            .map(|v| sygus_parser::SortedVar {
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

/// collect all needed model names (deduplicate, preserve first occurrence order)
pub fn collect_all_model_names(constraint_models: &[(String, bool)], models: &HashMap<String, &Model>) -> Vec<String> {
    let mut all_model_names = Vec::new();
    for (name, _) in constraint_models {
        if models.contains_key(name) && !all_model_names.contains(name) {
            all_model_names.push(name.clone());
        }
    }
    all_model_names
}

/// collect ModelDef list according to all_model_names
pub fn collect_loaded_models<'a>(all_model_names: &[String], models: &'a HashMap<String, &Model>) -> Vec<&'a Model> {
    all_model_names.iter().filter_map(|n| models.get(n).copied()).collect()
}

/// automatically get domain_sort (e.g. U/Pos/S)
pub fn get_domain_sort_from_models(models: &HashMap<String, &Model>) -> Option<String> {
    for model in models.values() {
        if let Some(univ) = model.univ_defs.get(0) {
            return Some(univ.sort.to_string());
        }
    }
    None
}

pub fn get_interped_value<'a>(
    interp: &'a crate::lang::model::FuncInterp,
    args: &[usize],
    id2sym: &[String],
) -> Option<&'a crate::parser::ast::LSTerm> {
    match interp {
        crate::lang::model::FuncInterp::SimpleInterp(_, _, _, term) => {
            if args.is_empty() {
                Some(term)
            } else {
                None
            }
        }
        crate::lang::model::FuncInterp::MatchInterp(_, _, _, _, cases) => {
            // TODO: implement match logic, directly match arguments
            for case in cases { // precise match
                if let Some(pat) = &case.pattern {
                    if pattern_match(&args.to_vec(), pat, &id2sym.to_vec()) {
                        return Some(&case.value);
                    }
                }
            }
            for case in cases { // fallback (_)
                if let Some(pat) = &case.pattern {
                    if pat.elements.iter().all(|e| matches!(e, crate::lang::model::PatternElem::Wildcard)) {
                        return Some(&case.value);
                    }
                }
            }
            None
        }
    }
}
