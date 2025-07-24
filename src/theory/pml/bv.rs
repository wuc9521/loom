use crate::lang::model::ModelDef;
use crate::parser::LSFile;
use crate::theory::utils::{eval, get_universe_domain_and_map, collect_bool_props, prop_bitvecs, replace_grammardef_sort, gen_constraints, collect_constraint_models};
use crate::theory::{
    pml::{get_relation_adjacency, get_transition_pred, utils::*},
    Logic,
};
use std::collections::HashMap;
use sygus_parser::api::{BitVec, Bool, Cmd, Term};
use sygus_parser::{Literal, Sort, SortedVar, SyGuSCmd, SyGuSTerm};

pub fn generate_concatenated_bitvector<F>(models: &[&ModelDef], eval_func: F) -> (String, usize)
where
    F: Fn(usize, &ModelDef, &Vec<String>, &String) -> bool,
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
            // dynamically get domain_sort
            if domain_sort.is_none() {
                if let Some(univ) = model.univ_defs.get(0) {
                    domain_sort = Some(univ.sort.to_string());
                }
            }
            models.insert(model.name.clone(), model);
        }
    }
    let constraint_models = collect_constraint_models(lsfile);
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
    // 4. collect propositions and concatenate with bitvector
    let props = collect_bool_props(&loaded_models, 1);
    let (prop_bitvecs, final_bv_length) = prop_bitvecs(
        &loaded_models,
        &props,
        &|prop, node, model, id2sym| {
            matches!(eval(prop, &[node], &model.func_interps, id2sym), Some(crate::lang::model::InterpedValue::Literal(lit)) if lit.to_string() == "true")
        },
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
    let mut let_bindings: Vec<(String, SyGuSTerm)> = Vec::new();
    let mut current_offset = 0;
    for (i, model) in loaded_models.iter().enumerate() {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        let high = current_offset + domain_size - 1;
        let low = current_offset;
        let seg_name = format!("seg{}", i);
        let_bindings.push((
            seg_name.clone(),
            BitVec::extract(high, low, Term::var("phi_val")),
        ));
        current_offset += domain_size;
    }
    let build_modal_body = |is_box: bool| -> SyGuSTerm {
        let mut model_segments = Vec::new();
        for (i, model) in loaded_models.iter().enumerate() {
            let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
            let domain_size = domain_syms.len();
            let transition_pred = get_transition_pred(model).unwrap();
            let adjacency = get_relation_adjacency(model, transition_pred);
            let mut new_segment_bits = Vec::new();
            for node in 0..domain_size {
                let neighbors = &adjacency[node];
                let op = if is_box { "and" } else { "or" };
                let fallback = if is_box { "1" } else { "0" };
                if neighbors.is_empty() {
                    new_segment_bits
                        .push(SyGuSTerm::Literal(Literal::BinConst(fallback.to_string())));
                } else {
                    let neighbor_checks: Vec<_> = neighbors
                        .iter()
                        .map(|&n| {
                            Term::app(
                                "bv1->bool",
                                vec![BitVec::extract(n, n, Term::var(&format!("seg{}", i)))],
                            )
                        })
                        .collect();
                    new_segment_bits
                        .push(Term::app("bool->bv1", vec![Term::app(op, neighbor_checks)]))
                }
            }
            let mut seg_term = None;
            for bit in new_segment_bits.into_iter() {
                seg_term = Some(if let Some(acc) = seg_term {
                    BitVec::concat(bit, acc)
                } else {
                    bit
                });
            }
            model_segments.push(seg_term.unwrap());
        }
        let mut body = None;
        for seg in model_segments.into_iter() {
            body = Some(if let Some(acc) = body {
                BitVec::concat(seg, acc)
            } else {
                seg
            });
        }
        Term::let_binding(let_bindings.clone(), body.unwrap())
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

    //TODO: check here.
    // grammar/synth-fun
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
    // generate constraints
    let model_map: HashMap<String, &ModelDef> = all_model_names
        .iter()
        .map(|n| (n.clone(), models.get(n).copied().unwrap()))
        .collect();
    if let Some(syn_name) = syn_name {
        let constraints = gen_constraints(
            &model_map,
            &syn_name,
            &constraint_models
        );
        cmds.extend(constraints);
    }
    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
