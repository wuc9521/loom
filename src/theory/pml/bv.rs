use crate::lang::model::Model;
use crate::parser::ast::LSTerm;
use crate::parser::LSFile;
use crate::theory::utils::{
    collect_bool_props, collect_constraint_models, eval, get_universe_domain_and_map, prop_bitvecs,
    replace_grammardef_sort,
};
use crate::theory::{
    pml::{get_relation_adjacency, get_transition_pred, utils::*},
    Logic,
};
use std::collections::HashMap;
use sygus_parser::api::{BitVec, Bool, Cmd, Term};
use sygus_parser::{Literal, Sort, SortedVar, SyGuSCmd, SyGuSTerm};

pub fn generate_concatenated_bitvector<F>(models: &[&Model], eval_func: F) -> (String, usize)
where
    F: Fn(usize, &Model, &Vec<String>, &String) -> bool,
{
    let mut full_bitstring_list = Vec::new();
    let mut total_bits = 0;
    for model in models {
        let (domain_syms, id2sym, universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        for node in 0..domain_size {
            if eval_func(node, model, &id2sym, &universe_sort) {
                full_bitstring_list.push('1');
            } else {
                full_bitstring_list.push('0');
            }
        }
        total_bits += domain_size;
    }
    // LSB-first
    let bv_str = format!("{}", full_bitstring_list.iter().rev().collect::<String>());
    (bv_str, total_bits)
}

pub fn to_sygus(lsfile: &LSFile) -> Vec<SyGuSCmd> {
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
    cmds.push(
        Cmd::define_fun("bv1->bool")
            .arg("b", Sort::bitvec(1))
            .ret(Sort::bool())
            .body(Bool::eq(Term::var("b"), BitVec::const_(1, 1)))
            .build(),
    );
    // 1. collect all models
    let mut models = HashMap::new();
    let mut domain_sort = None;
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::ModelDef(model) = cmd {
            if domain_sort.is_none() {
                if let Some(univ) = model.univ_defs.get(0) {
                    domain_sort = Some(univ.sort.to_string());
                }
            }
            models.insert(model.name.clone(), model);
        }
    }
    let constraint_models = collect_constraint_models(lsfile);
    // 2. collect all needed models (deduplicate, preserve first occurrence order)
    let mut all_model_names = Vec::new();
    for (name, _) in &constraint_models {
        if models.contains_key(name) && !all_model_names.contains(name) {
            all_model_names.push(name.clone());
        }
    }
    let loaded_models: Vec<&Model> = all_model_names
        .iter()
        .filter_map(|n| models.get(n).copied())
        .collect();
    // 3. build global offset mapping (model name -> (offset, domain_size))
    let mut model_offsets = HashMap::new();
    let mut current_offset = 0;
    for model in &loaded_models {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        model_offsets.insert(model.name.clone(), (current_offset, domain_size));
        current_offset += domain_size;
    }
    let final_bv_length = current_offset;
    // 4. collect propositions and concatenate with bitvector, 1-bit per proposition
    let props = collect_bool_props(&loaded_models, 1);
    let (prop_bitvecs, _bv_len) = prop_bitvecs(
        &loaded_models,
        &props,
        &|prop, node, model, id2sym| matches!(eval(prop, &[node], &model.func_interps, id2sym), Some(LSTerm::Literal(lit)) if lit.to_string() == "true"),
    );
    for prop in &props {
        cmds.push(
            Cmd::define_fun(&prop)
                .ret(Sort::bitvec(final_bv_length))
                .body(Term::lit(Literal::BinConst(prop_bitvecs[prop].clone())))
                .build(),
        );
    }
    let bv_sort = Sort::bitvec(final_bv_length);
    cmds.push(modal_and(&bv_sort));
    cmds.push(modal_or(&bv_sort));
    cmds.push(modal_not(&bv_sort));
    // 5. modal.box/modal.dia, extract bit from global offset
    let build_modal_body = |is_box: bool| -> SyGuSTerm {
        let mut all_bits = Vec::new();
        for model in &loaded_models {
            let (offset, domain_size) = model_offsets[&model.name];
            let transition_pred = get_transition_pred(model).unwrap();
            let adjacency = get_relation_adjacency(model, transition_pred);
            for node in 0..domain_size {
                let neighbors = &adjacency[node];
                let op = if is_box { "and" } else { "or" };
                let fallback = if is_box { "1" } else { "0" };
                let bit = if neighbors.is_empty() {
                    SyGuSTerm::Literal(Literal::BinConst(fallback.to_string()))
                } else {
                    let neighbor_checks: Vec<_> = neighbors
                        .iter()
                        .map(|&n| {
                            Term::app(
                                "bv1->bool",
                                vec![BitVec::extract(
                                    offset + n,
                                    offset + n,
                                    Term::var("phi_val"),
                                )],
                            )
                        })
                        .collect();
                    Term::app("bool->bv1", vec![Term::app(op, neighbor_checks)])
                };
                all_bits.push(bit);
            }
        }
        let mut expr = all_bits[0].clone();
        for bit in all_bits.iter().skip(1) {
            expr = BitVec::concat(bit.clone(), expr);
        }
        expr
    };
    // box
    cmds.push(
        Cmd::define_fun("modal.box")
            .arg("phi_val", bv_sort.clone())
            .ret(bv_sort.clone())
            .body(build_modal_body(true))
            .build(),
    );
    // dia
    cmds.push(
        Cmd::define_fun("modal.dia")
            .arg("phi_val", bv_sort.clone())
            .ret(bv_sort.clone())
            .body(build_modal_body(false))
            .build(),
    );
    // 6. grammar/synth-fun
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
            let new_grammar = replace_grammardef_sort(grammar, final_bv_length);
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
    // 7. generate constraints, global offset + init_idx
    if let Some(syn_name) = syn_name {
        for (model_name, is_true) in &constraint_models {
            if let Some((offset, _domain_size)) = model_offsets.get(model_name) {
                // look up models and get ext_ctx,
                if let Some(model) = loaded_models.iter().find(|m| &m.name == model_name) {
                    if let Some(crate::theory::ModelCtx::PML(ctx)) = &model.ext_ctx {
                        let (domain_syms, id2sym, _universe_sort) =
                            get_universe_domain_and_map(model);
                        let sym2id: std::collections::HashMap<String, usize> = domain_syms
                            .iter()
                            .enumerate()
                            .map(|(i, s)| (s.clone(), i))
                            .collect();
                        let mut init_idx = 0; // fallback: 0
                        if let Some(init_name) = &ctx.init_name {
                            if let Some(LSTerm::AppTerm(sym, _)) =
                                eval(init_name, &[], &model.func_interps, &id2sym)
                            {
                                if let Some(&idx) = sym2id.get(&sym.to_string()) {
                                    init_idx = idx;
                                }
                            }
                        }
                        let bit_to_check_index = *offset + init_idx;
                        let target_bit = if *is_true { "1" } else { "0" };
                        cmds.push(
                            Cmd::constraint()
                                .body(Bool::eq(
                                    BitVec::extract(
                                        bit_to_check_index,
                                        bit_to_check_index,
                                        Term::var(&syn_name),
                                    ),
                                    Term::lit(Literal::BinConst(target_bit.to_string())),
                                ))
                                .build(),
                        );
                    }
                }
            }
        }
    }
    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
