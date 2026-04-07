use crate::parser::DelisProg;
use crate::theory::re::ctx::ReCtx;
use crate::theory::re::utils::{re_comp, re_inter, re_union};
use crate::theory::Logic;
use sygus_parser::api::{BitVec, Bool, Term};
use sygus_parser::ast::{GroupedRuleList, SyGuSGTerm, SyGuSTerm};
use sygus_parser::{Cmd, GrammarDef, Identifier, Literal, Sort, SortedVar, SyGuSCmd};

/// Decode cvc5 output (define-fun commands) to final result
pub fn decode(define_funs: &[SyGuSCmd]) -> Result<String, String> {
    let mut result = String::new();
    for define_fun in define_funs {
        if let SyGuSCmd::DefineFun(name, args, sort, body) = define_fun {
            result.push_str(&format!(
                "(define-fun {} ({}) {} {})",
                name,
                args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                match sort {
                    Sort::Simple(_) => "RegLan".to_string(),
                    _ => {
                        return Err(format!(
                            "Unsupported sort while decoding cvc5 output: {:?}",
                            sort
                        ));
                    }
                },
                body
            ));
        }
    }
    Ok(result)
}

pub fn to_sygus(lsfile: &DelisProg) -> Vec<SyGuSCmd> {
    let ctx = ReCtx::new(lsfile);
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
    let total_bv_length = ctx.total_bv_length;
    let bv_sort = Sort::bitvec(total_bv_length);
    // 1. define proposition bitvector
    for prop in &ctx.props {
        let (bv_str, _) = {
            let mut full_bitstring_list = Vec::new();
            for layout in &ctx.layouts {
                for &(start, end) in &layout.subwords {
                    if end == start + 1 {
                        if layout.eval_prop_at(prop, start) {
                            full_bitstring_list.push('1');
                        } else {
                            full_bitstring_list.push('0');
                        }
                    } else {
                        full_bitstring_list.push('0');
                    }
                }
            }
            let bv_str: String = full_bitstring_list.iter().rev().collect();
            (bv_str, full_bitstring_list.len())
        };
        cmds.push(
            Cmd::define_fun(&prop)
                .ret(Sort::bitvec(total_bv_length))
                .body(Term::lit(Literal::BinConst(bv_str.clone())))
                .build(),
        );
    }
    cmds.push(re_union(&bv_sort));
    cmds.push(re_inter(&bv_sort));
    cmds.push(re_comp(&bv_sort));
    // 6. structural operations re-concat/re-star
    // --- re-concat ---
    let mut concat_bits = Vec::new();
    for layout in &ctx.layouts {
        for &(start, end) in &layout.subwords {
            let mut split_checks = Vec::new();
            for k in start..=end {
                let idx1 = layout.global_offset + layout.subword_index(start, k);
                let idx2 = layout.global_offset + layout.subword_index(k, end);
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
    for layout in &ctx.layouts {
        for &(start, end) in &layout.subwords {
            if start == end {
                empty_str_bits.push(BitVec::const_(1, 1));
            } else {
                empty_str_bits.push(BitVec::const_(0, 1));
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
    let max_len = ctx
        .layouts
        .iter()
        .map(|layout| layout.domain_size)
        .max()
        .unwrap_or(0);
    let_bindings.push((prev_var.clone(), prev_expr.clone()));
    for i in 0..max_len {
        let mut one_step_bits = Vec::new();
        for layout in &ctx.layouts {
            for &(start, end) in &layout.subwords {
                let mut split_checks = Vec::new();
                for k in (start + 1)..=end {
                    let idx_phi = layout.global_offset + layout.subword_index(start, k);
                    let idx_star = layout.global_offset + layout.subword_index(k, end);
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
        let mut one_step_expr = one_step_bits[0].clone();
        for bit in one_step_bits.iter().skip(1) {
            one_step_expr = BitVec::concat(bit.clone(), one_step_expr);
        }
        let cur_var = format!("star_iter_{}", i + 1);
        let cur_expr = BitVec::bvor(Term::var(&prev_var), one_step_expr);
        let_bindings.push((cur_var.clone(), cur_expr.clone()));
        prev_var = cur_var;
    }
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
    fn grammar_symbol_for_sort(sort_name: &str, ret_sort_name: &str) -> String {
        if sort_name == ret_sort_name {
            "Start".to_string()
        } else {
            sort_name.to_string()
        }
    }

    fn delis_term_type_to_grammar(
        decl: &crate::parser::ast::DeclareTermTypesCmd,
        ret_sort: &Sort,
    ) -> Option<GrammarDef> {
        let ret_sort_name = ret_sort.to_string();
        decl.sort_decls
            .iter()
            .position(|sort_decl| sort_decl.symbol == ret_sort_name)?;

        let sorted_vars = decl
            .sort_decls
            .iter()
            .map(|sort_decl| SortedVar {
                name: grammar_symbol_for_sort(&sort_decl.symbol, &ret_sort_name),
                sort: Sort::from_name(&sort_decl.symbol),
            })
            .collect::<Vec<_>>();

        let grouped_rule_lists = decl
            .sort_decls
            .iter()
            .zip(decl.term_type_decls.iter())
            .map(|(sort_decl, term_type_decl)| {
                let symbol = grammar_symbol_for_sort(&sort_decl.symbol, &ret_sort_name);
                let terms = term_type_decl
                    .constructors
                    .iter()
                    .map(|cons_decl| {
                        if cons_decl.sorts.is_empty() {
                            let symbol = cons_decl.symbol.to_string();
                            let normalized_symbol = symbol
                                .strip_prefix("re.")
                                .map(str::to_string)
                                .unwrap_or(symbol);
                            SyGuSGTerm::SyGuSTerm(SyGuSTerm::Identifier(
                                Identifier::from_str(&normalized_symbol).unwrap(),
                            ))
                        } else {
                            let args = cons_decl
                                .sorts
                                .iter()
                                .map(|sort| {
                                    SyGuSTerm::Identifier(Identifier::from_str(
                                        &grammar_symbol_for_sort(
                                            &sort.to_string(),
                                            &ret_sort_name,
                                        ),
                                    )
                                    .unwrap())
                                })
                                .collect::<Vec<_>>();
                            SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(
                                cons_decl.symbol.clone(),
                                args,
                            ))
                        }
                    })
                    .collect::<Vec<_>>();
                GroupedRuleList {
                    symbol,
                    sort: Sort::from_name(&sort_decl.symbol),
                    terms,
                }
            })
            .collect::<Vec<_>>();

        Some(GrammarDef {
            sorted_vars,
            grouped_rule_lists,
        })
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
    let declare_term_types = lsfile.cmds.iter().find_map(|cmd| match cmd {
        crate::parser::DelisCmd::DeclareTermTypes(decl) => Some(decl),
        _ => None,
    });
    let mut syn_name = None;
    for cmd in &lsfile.cmds {
        if let crate::parser::DelisCmd::As(sygus_parser::SyGuSCmd::SynthFun(
            name,
            args,
            sort,
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
            let fallback_grammar = if grammar.is_none() {
                declare_term_types.and_then(|decl| delis_term_type_to_grammar(decl, sort))
            } else {
                None
            };
            let source_grammar = grammar.clone().or(fallback_grammar);
            let new_grammar = replace_grammardef_sort(&source_grammar, total_bv_length);
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
    // 8. generate constraints, use global offset
    if let Some(syn_name) = syn_name {
        for (model_name, is_true) in &ctx.constraint_models {
            if let Some(layout) = ctx.layout_for_model(model_name) {
                let bit_to_check_index =
                    layout.global_offset + layout.subword_index(0, layout.domain_size);
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
    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
