use crate::lang::model::ModelDef;
use crate::parser::LSFile;
use crate::theory::re::utils::{re_inter, re_comp, re_union};
use crate::theory::utils::{gen_constraints, get_universe_domain_and_map};
use crate::theory::Logic;
use std::collections::{HashMap, HashSet};
use sygus_parser::api::{BitVec, Bool, Cmd, Term};
use sygus_parser::{Literal, Sort, SortedVar, SyGuSCmd};

pub fn translate_to_bv_cmds(lsfile: &LSFile) -> Vec<SyGuSCmd> {
    let mut cmds = Vec::new();
    cmds.push(SyGuSCmd::SetLogic(Logic::BV.to_string()));
    cmds.push(
        Cmd::define_fun("bool->bv1")
            .arg("b", Sort::bool())
            .ret(Sort::bitvec(1))
            .body(Term::ite(
                Term::var("b"),
                BitVec::const_(1, 1),
                BitVec::const_(0, 1),
            ))
            .build(),
    );
    // 1. collect all models
    let mut models = HashMap::new();
    let mut domain_sort = None;
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::ModelDef(model) = cmd {
            // dynamically get domain_sort
            if domain_sort.is_none() {
                if let Some(univ) = model.univ_defs.get(0) {
                    domain_sort = Some(univ.sort.to_string());
                }
            }
            models.insert(model.name.clone(), model);
        }
    }
    // 2. collect model names and positive/negative constraints
    let constraint_models = crate::theory::utils::collect_constraint_models(lsfile);
    // 3. collect all needed models (deduplicate, preserve first occurrence order)
    let mut all_model_names = Vec::new();
    for (name, _) in &constraint_models {
        if models.contains_key(name) && !all_model_names.contains(name) {
            all_model_names.push(name.clone());
        }
    }
    let loaded_models: Vec<&ModelDef> = all_model_names
        .iter()
        .filter_map(|n| models.get(n).copied())
        .collect();
    // 4. collect propositions
    let mut all_props = HashSet::new();
    for model in &loaded_models {
        for interp in &model.func_interps {
            if let crate::lang::model::FuncInterp::MatchInterp(name, cases) = interp {
                let arity = cases
                    .iter()
                    .find_map(|c| c.pattern.as_ref())
                    .map(|p| p.elements.len())
                    .unwrap_or(0);
                let is_bool = cases.iter().any(|c| matches!(&c.value, crate::lang::model::InterpedValue::Literal(lit) if lit.to_string() == "true" || lit.to_string() == "false"));
                if is_bool && arity == 1 {
                    all_props.insert(name.clone());
                }
            }
        }
    }
    let mut all_props: Vec<_> = all_props.into_iter().collect();
    all_props.sort();
    // 5. define proposition bitvector
    let mut total_bv_length = 0;
    let mut grammar_terms = Vec::new();
    let mut prop_bitvecs = HashMap::new();
    for prop in &all_props {
        let (bv_str, length) = {
            let mut full_bitstring_list = Vec::new();
            for model in &loaded_models {
                let (domain_syms, id2sym, _universe_sort) = get_universe_domain_and_map(model);
                let domain_size = domain_syms.len();
                for start in 0..=domain_size {
                    for end in start..=domain_size {
                        // only when start+1==end is atomic proposition
                        if end == start + 1 {
                            let node = start;
                            let val = crate::theory::utils::eval(
                                prop,
                                &[node],
                                &model.func_interps,
                                &id2sym,
                            );
                            if matches!(val, Some(crate::lang::model::InterpedValue::Literal(lit)) if lit.to_string() == "true")
                            {
                                full_bitstring_list.push('1');
                            } else {
                                full_bitstring_list.push('0');
                            }
                        } else {
                            full_bitstring_list.push('0');
                        }
                    }
                }
            }
            let bv_str: String = full_bitstring_list.iter().rev().collect();
            (bv_str, full_bitstring_list.len())
        };
        if total_bv_length == 0 {
            total_bv_length = length;
        }
        cmds.push(
            Cmd::define_fun(&prop)
                .ret(Sort::bitvec(total_bv_length))
                .body(Term::lit(Literal::BinConst(bv_str.clone())))
                .build(),
        );
        grammar_terms.push(prop.clone());
        prop_bitvecs.insert(prop.clone(), bv_str);
    }
    let bv_sort = Sort::bitvec(total_bv_length);
    cmds.push(re_union(&bv_sort));
    cmds.push(re_inter(&bv_sort));
    cmds.push(re_comp(&bv_sort));
    // 6. structural operations re-concat/re-star
    // --- re-concat ---
    // first enumerate all substrings of all models, build global index
    let mut subwords = Vec::new();
    let mut subword_offsets = Vec::new(); // start offset of each model
    let mut current_offset = 0;
    for model in &loaded_models {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        let mut model_subwords = Vec::new();
        for start in 0..=domain_size {
            for end in start..=domain_size {
                model_subwords.push((start, end));
                subwords.push((current_offset, start, end)); // (offset, start, end)
            }
        }
        subword_offsets.push(current_offset);
        current_offset += model_subwords.len();
    }
    let total_bv_length = current_offset;
    // build mapping from (model_idx, start, end) to global index
    let mut subword_idx_map = Vec::new(); // Vec<HashMap<(usize, usize), usize>>
    let mut global_idx = 0;
    for model in &loaded_models {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        let mut map = HashMap::new();
        for start in 0..=domain_size {
            for end in start..=domain_size {
                map.insert((start, end), global_idx);
                global_idx += 1;
            }
        }
        subword_idx_map.push(map);
    }
    // re-concat
    let mut concat_bits = Vec::new();
    for (model_idx, model) in loaded_models.iter().enumerate() {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        let map = &subword_idx_map[model_idx];
        for start in 0..=domain_size {
            for end in start..=domain_size {
                let mut split_checks = Vec::new();
                for k in start..=end {
                    let idx1 = map[&(start, k)];
                    let idx2 = map[&(k, end)];
                    let check1 = Term::eq(
                        BitVec::extract(idx1, idx1, Term::var("phi1")),
                        BitVec::const_(1, 1),
                    );
                    let check2 = Term::eq(
                        BitVec::extract(idx2, idx2, Term::var("phi2")),
                        BitVec::const_(1, 1),
                    );
                    split_checks.push(Bool::and(vec![check1, check2]));
                }
                let bit = if split_checks.is_empty() {
                    BitVec::const_(0, 1)
                } else {
                    Term::app("bool->bv1", vec![Bool::or(split_checks)])
                };
                concat_bits.push(bit);
            }
        }
    }
    let mut concat_expr = concat_bits[0].clone();
    for bit in concat_bits.iter().skip(1) {
        concat_expr = BitVec::concat(bit.clone(), concat_expr);
    }
    cmds.push(
        Cmd::define_fun("re.++")
            .arg("phi1", bv_sort.clone())
            .arg("phi2", bv_sort.clone())
            .ret(bv_sort.clone())
            .body(concat_expr)
            .build(),
    );
    // --- re-star ---
    let mut empty_str_bits = Vec::new();
    for (_, model) in loaded_models.iter().enumerate() {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        for start in 0..=domain_size {
            for end in start..=domain_size {
                if start == end {
                    empty_str_bits.push(BitVec::const_(1, 1));
                } else {
                    empty_str_bits.push(BitVec::const_(0, 1));
                }
            }
        }
    }
    let mut let_bindings = Vec::new();
    let mut prev_var = "star_iter_0".to_string();
    let prev_expr = {
        let mut expr = empty_str_bits[0].clone();
        for bit in empty_str_bits.iter().skip(1) {
            expr = BitVec::concat(bit.clone(), expr);
        }
        expr
    };
    let max_len = loaded_models
        .iter()
        .map(|m| {
            let (domain_syms, _, _) = get_universe_domain_and_map(m);
            domain_syms.len()
        })
        .max()
        .unwrap_or(0);
    let_bindings.push((prev_var.clone(), prev_expr.clone()));
    for i in 0..max_len {
        let mut one_step_bits = Vec::new();
        for (model_idx, model) in loaded_models.iter().enumerate() {
            let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
            let domain_size = domain_syms.len();
            let map = &subword_idx_map[model_idx];
            for start in 0..=domain_size {
                for end in start..=domain_size {
                    let mut split_checks = Vec::new();
                    for k in (start + 1)..=end {
                        let idx_phi = map[&(start, k)];
                        let idx_star = map[&(k, end)];
                        let check_phi = Term::eq(
                            BitVec::extract(idx_phi, idx_phi, Term::var("phi")),
                            BitVec::const_(1, 1),
                        );
                        let check_star = Term::eq(
                            BitVec::extract(idx_star, idx_star, Term::var(&prev_var)),
                            BitVec::const_(1, 1),
                        );
                        split_checks.push(Bool::and(vec![check_phi, check_star]));
                    }
                    let bit = if split_checks.is_empty() {
                        BitVec::const_(0, 1)
                    } else {
                        Term::app("bool->bv1", vec![Bool::or(split_checks)])
                    };
                    one_step_bits.push(bit);
                }
            }
        }
        let mut one_step_expr = one_step_bits[0].clone();
        for bit in one_step_bits.iter().skip(1) {
            one_step_expr = BitVec::concat(bit.clone(), one_step_expr);
        }
        let cur_var = format!("star_iter_{}", i + 1);
        let cur_expr = BitVec::bvor(Term::var(&prev_var), one_step_expr);
        let_bindings.push((cur_var.clone(), cur_expr.clone()));
        prev_var = cur_var;
        // prev_expr = cur_expr;
    }
    // nested let, each layer binds only one variable, body is the next let, innermost body is the final variable name
    let mut star_body = Term::var(&let_bindings.last().unwrap().0);
    for (let_name, expr) in let_bindings.iter().rev() {
        star_body = Term::let_binding(vec![(let_name.clone(), expr.clone())], star_body);
    }
    cmds.push(
        Cmd::define_fun("re.*")
            .arg("phi", bv_sort.clone())
            .ret(bv_sort.clone())
            .body(star_body)
            .build(),
    );
    // 7. grammar/synth-fun, check here.
    fn replace_grammardef_sort(
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
                    sort: Sort::bitvec(bv_len),
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

    let mut syn_name = None;
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::As(sygus_parser::SyGuSCmd::SynthFun(
            name,
            args,
            _sort,
            grammar,
        )) = cmd
        {
            let new_args: Vec<SortedVar> = args
                .iter()
                .map(|v| sygus_parser::SortedVar {
                    name: v.name.clone(),
                    sort: bv_sort.clone(),
                })
                .collect();
            let new_sort = bv_sort.clone();
            let new_grammar = replace_grammardef_sort(grammar, total_bv_length);
            let mut builder = Cmd::synth_fun(name.as_str());
            for arg in new_args.iter() {
                builder = builder.arg(&arg.name, arg.sort.clone());
            }
            let builder = builder.ret(new_sort);
            let builder = if let Some(g) = new_grammar {
                builder.grammar(g)
            } else {
                builder
            };
            cmds.push(builder.build());
            syn_name = Some(name.clone());
        }
    }
    if let Some(syn_name) = syn_name {
        let constraints = gen_constraints(
            &models,
            &syn_name,
            &constraint_models
        );
        cmds.extend(constraints);
    }
    // 8. check-synth
    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
