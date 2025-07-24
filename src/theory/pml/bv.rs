use crate::lang::model::ModelDef;
use crate::parser::LSFile;
use crate::theory::utils::{eval, get_universe_domain_and_map};
use crate::theory::{
    pml::{get_relation_adjacency, get_transition_pred, utils::*},
    Logic,
};
use std::collections::{HashMap, HashSet};
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
        Cmd::define_fun("b1->bool")
            .arg("b", Sort::bitvec(1))
            .ret(Sort::bool())
            .body(Bool::eq(Term::var("b"), BitVec::const_(1, 1)))
            .build(),
    );
    // 3. 收集所有 ModelDef
    let mut models = HashMap::new();
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::ModelDef(model) = cmd {
            models.insert(model.name.clone(), model);
        }
    }
    // 4. 按constraint出现顺序收集模型名和正负
    let mut constraint_models = Vec::new(); // Vec<(model_name, is_true)>
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
    // 5. 按顺序收集所有需要的模型（去重，保留首次出现顺序）
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
    // 6. 收集所有 proposition
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
    // 7. proposition bitvector定义
    let mut final_bv_length = 0;
    let mut grammar_terms = Vec::new();
    let mut prop_bitvecs = HashMap::new();
    for prop in &all_props {
        let (bv_str, length) = generate_concatenated_bitvector(
            &loaded_models,
            |node, model, id2sym, _| matches!(eval(prop, &[node], &model.func_interps, id2sym), Some(crate::lang::model::InterpedValue::Literal(lit)) if lit.to_string() == "true"),
        );
        if final_bv_length == 0 {
            final_bv_length = length;
        }
        cmds.push(
            Cmd::define_fun(&prop)
                .ret(Sort::bitvec(final_bv_length))
                .body(Term::lit(Literal::BinConst(bv_str.clone())))
                .build(),
        );
        grammar_terms.push(prop.clone());
        prop_bitvecs.insert(prop.clone(), bv_str);
    }
    let bv_sort = Sort::bitvec(final_bv_length);
    cmds.push(modal_and(&bv_sort));
    cmds.push(modal_or(&bv_sort));
    cmds.push(modal_not(&bv_sort));
    let mut let_bindings = Vec::new();
    let mut current_offset = 0;
    for (i, model) in loaded_models.iter().enumerate() {
        let (domain_syms, _id2sym, _universe_sort) = get_universe_domain_and_map(model);
        let domain_size = domain_syms.len();
        let high = current_offset + domain_size - 1;
        let low = current_offset;
        let seg_name = format!("seg{}", i);
        let_bindings.push(sygus_parser::VarBinding {
            name: seg_name.clone(),
            term: Box::new(BitVec::extract(high, low, Term::var("phi_val"))),
        });
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
                                "b1->bool",
                                vec![BitVec::extract(n, n, Term::var(&format!("seg{}", i)))],
                            )
                        })
                        .collect();
                    new_segment_bits
                        .push(Term::app("bool->bv1", vec![Term::app(op, neighbor_checks)]))
                }
            }
            let mut seg_term = None;
            for bit in new_segment_bits.into_iter().rev() {
                seg_term = Some(if let Some(acc) = seg_term {
                    BitVec::concat(bit, acc)
                } else {
                    bit
                });
            }
            model_segments.push(seg_term.unwrap());
        }
        let mut body = None;
        for seg in model_segments.into_iter().rev() {
            body = Some(if let Some(acc) = body {
                BitVec::concat(seg, acc)
            } else {
                seg
            });
        }
        //TODO: check here
        SyGuSTerm::Let(let_bindings.clone(), Box::new(body.unwrap()))
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

    // 10. synth-fun
    // 查找输入文件的 synth-fun，递归替换所有 sort 为 (_ BitVec N)
    fn replace_sort_with_bv(bv_len: usize) -> Sort {
        Sort::bitvec(bv_len)
    }
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
                    sort: replace_sort_with_bv(bv_len),
                })
                .collect();
            new_g.grouped_rule_lists = grammar
                .grouped_rule_lists
                .iter()
                .map(|gr| {
                    let mut new_gr = gr.clone();
                    new_gr.sort = replace_sort_with_bv(bv_len);
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
                    sort: replace_sort_with_bv(final_bv_length),
                })
                .collect();
            let new_sort = replace_sort_with_bv(final_bv_length);
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
    // 11. constraint
    let mut current_offset = 0;
    for (model_name, is_true) in &constraint_models {
        if let Some(model) = models.get(model_name) {
            let bit_to_check_index = current_offset;
            let target_bit = if *is_true { "1" } else { "0" };
            cmds.push(
                Cmd::constraint()
                    .body(Bool::eq(
                        BitVec::extract(
                            bit_to_check_index,
                            bit_to_check_index,
                            Term::var(syn_name.as_ref().unwrap()),
                        ),
                        Term::lit(Literal::BinConst(target_bit.to_string())),
                    ))
                    .build(),
            );
            current_offset += model
                .univ_defs
                .iter()
                .find(|u| u.sort.to_string() == "U")
                .map(|u| u.elems.len())
                .unwrap_or(0);
        }
    }
    // 12. check-synth
    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
