use crate::lang::model::ModelDef;
use crate::parser::LSFile;
use crate::theory::{pml::get_relation_adjacency, Logic};
use std::collections::{HashMap, HashSet};
use sygus_parser::SyGuSCmd;

// 生成 bitvector 字符串和长度
pub fn generate_concatenated_bitvector<F>(models: &[&ModelDef], eval_func: F) -> (String, usize)
where
    F: Fn(usize, &ModelDef) -> bool,
{
    let mut full_bitstring_list = Vec::new();
    let mut total_bits = 0;
    for model in models {
        let domain_size = model
            .univ_defs
            .iter()
            .find(|u| u.sort.to_string() == "U")
            .map(|u| u.elems.len())
            .unwrap_or(0);
        for node in 0..domain_size {
            if eval_func(node, model) {
                full_bitstring_list.push('1');
            } else {
                full_bitstring_list.push('0');
            }
        }
        total_bits += domain_size;
    }
    // LSB-first
    let bv_str = format!("#b{}", full_bitstring_list.iter().rev().collect::<String>());
    (bv_str, total_bits)
}

pub fn generate_modal_operator_functions(
    models: &[&ModelDef],
    total_bv_length: usize,
    relation_name: &str,
) -> Vec<String> {
    let mut output = Vec::new();
    let mut cmds: Vec<SyGuSCmd> = Vec::new();
    cmds.push(SyGuSCmd::SetLogic(Logic::BV.to_string()));
    let mut let_bindings = Vec::new();
    let mut current_offset = 0;
    for (i, model) in models.iter().enumerate() {
        let domain_size = model
            .univ_defs
            .iter()
            .find(|univ| univ.sort.to_string() == "U")
            .map(|u| u.elems.len())
            .unwrap_or(0);
        let high = current_offset + domain_size - 1;
        let low = current_offset;
        let_bindings.push(format!("(seg{} ((_ extract {} {}) phi_val))", i, high, low));
        current_offset += domain_size;
    }
    // box 算子
    output.push(format!(
        "(define-fun modal.box ((phi_val (_ BitVec {}))) (_ BitVec {})",
        total_bv_length, total_bv_length
    ));
    output.push("  (let (".to_string());
    for binding in &let_bindings {
        output.push(format!("    {}", binding));
    }
    output.push("  ) ; end bindings".to_string());
    let mut model_segments = Vec::new();
    for (i, model) in models.iter().enumerate() {
        let domain_size = model
            .univ_defs
            .iter()
            .find(|u| u.sort.to_string() == "U")
            .map(|u| u.elems.len())
            .unwrap_or(0);
        let adjacency = get_relation_adjacency(model, relation_name);
        let mut new_segment_bits = Vec::new();
        for node in 0..domain_size {
            let neighbors = &adjacency[node];
            if neighbors.is_empty() {
                new_segment_bits.push("#b1".to_string());
            } else {
                let neighbor_checks: Vec<_> = neighbors
                    .iter()
                    .map(|&n| format!("(b1->bool ((_ extract {} {}) seg{}))", n, n, i))
                    .collect();
                new_segment_bits.push(format!("(bool->bv1 (and {}))", neighbor_checks.join(" ")));
            }
        }
        if new_segment_bits.is_empty() {
            model_segments.push(format!("(_ bv0 {})", domain_size));
        } else {
            model_segments.push(build_bv_from_lsb_first_list(&new_segment_bits));
        }
    }
    output.push(format!(
        "    {} ; body of let",
        build_bv_from_lsb_first_list(&model_segments)
    ));
    output.push("  ) ; end let".to_string());
    output.push(")".to_string());
    // dia 算子
    output.push(format!(
        "(define-fun modal.dia ((phi_val (_ BitVec {}))) (_ BitVec {})",
        total_bv_length, total_bv_length
    ));
    output.push("  (let (".to_string());
    for binding in &let_bindings {
        output.push(format!("    {}", binding));
    }
    output.push("  ) ; end bindings".to_string());
    let mut model_segments = Vec::new();
    for (i, model) in models.iter().enumerate() {
        let domain_size = model
            .univ_defs
            .iter()
            .find(|u| u.sort.to_string() == "U")
            .map(|u| u.elems.len())
            .unwrap_or(0);
        let adjacency = get_relation_adjacency(model, relation_name);
        let mut new_segment_bits = Vec::new();
        for node in 0..domain_size {
            let neighbors = &adjacency[node];
            if neighbors.is_empty() {
                new_segment_bits.push("#b0".to_string());
            } else {
                let neighbor_checks: Vec<_> = neighbors
                    .iter()
                    .map(|&n| format!("(b1->bool ((_ extract {} {}) seg{}))", n, n, i))
                    .collect();
                new_segment_bits.push(format!("(bool->bv1 (or {}))", neighbor_checks.join(" ")));
            }
        }
        if new_segment_bits.is_empty() {
            model_segments.push(format!("(_ bv0 {})", domain_size));
        } else {
            model_segments.push(build_bv_from_lsb_first_list(&new_segment_bits));
        }
    }
    output.push(format!(
        "    {} ; body of let",
        build_bv_from_lsb_first_list(&model_segments)
    ));
    output.push("  ) ; end let".to_string());
    output.push(")".to_string());
    output
}

pub fn build_bv_from_lsb_first_list(items: &[String]) -> String {
    if items.is_empty() {
        return "(_ bv0 1)".to_string();
    }
    let mut expr = items[0].clone();
    for item in items.iter().skip(1) {
        expr = format!("(concat {} {})", item, expr);
    }
    expr
}

/// 主入口：将 LSFile 翻译为 SyGuS BV 程序字符串
pub fn translate_to_bv(lsfile: &LSFile) -> String {
    // 1. 收集所有 ModelDef
    let mut models = HashMap::new();
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::ModelDef(model) = cmd {
            models.insert(model.name.clone(), model);
        }
    }
    // 2. 收集所有 constraint，自动识别 true/false
    let mut true_models = HashSet::new();
    let mut false_models = HashSet::new();
    for cmd in &lsfile.cmds {
        if let crate::parser::LSCmd::As(sygus_cmd) = cmd {
            if let sygus_parser::SyGuSCmd::Constraint(term) = sygus_cmd {
                use sygus_parser::SyGuSTerm::*;
                match term {
                    Application(sym, args) if sym.to_string() == "models" && args.len() >= 1 => {
                        if let Identifier(model_sym) = &args[0] {
                            true_models.insert(model_sym.to_string());
                        }
                    }
                    Application(sym, args) if sym.to_string() == "not" && args.len() == 1 => {
                        if let Application(models_sym, models_args) = &args[0] {
                            if models_sym.to_string() == "models" && models_args.len() >= 1 {
                                if let Identifier(model_sym) = &models_args[0] {
                                    false_models.insert(model_sym.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    // 3. 按顺序收集所有需要的模型
    let mut all_model_names = Vec::new();
    for name in true_models.iter().chain(false_models.iter()) {
        if models.contains_key(name) && !all_model_names.contains(name) {
            all_model_names.push(name.clone());
        }
    }
    let loaded_models: Vec<&ModelDef> = all_model_names
        .iter()
        .filter_map(|n| models.get(n).copied())
        .collect();
    let num_true_models = true_models.len();
    // 4. 收集所有 proposition
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
    // 5. 生成 SyGuS (BV) 文件
    let mut output = Vec::new();
    let cmt = "; ";
    output.push("(set-logic BV)".to_string());
    output.push("".to_string());
    output.push(format!("{}--- Helper Functions ---", cmt));
    output.push("(define-fun bool->bv1 ((b Bool)) (_ BitVec 1) (ite b #b1 #b0))".to_string());
    output.push("(define-fun b1->bool ((b (_ BitVec 1))) Bool (= b #b1))".to_string());
    output.push("".to_string());
    // 6. 生成 proposition 的 bitvector 定义
    let mut final_bv_length = 0;
    let mut grammar_terms = Vec::new();
    let mut prop_bitvecs = HashMap::new();
    for prop in &all_props {
        let (bv_str, length) = generate_concatenated_bitvector(&loaded_models, |node, model| {
            model
                .func_interps
                .iter()
                .find_map(|interp| {
                    if let crate::lang::model::FuncInterp::MatchInterp(name, cases) = interp {
                        if name == prop {
                            for case in cases {
                                if let Some(pat) = &case.pattern {
                                    if pat.elements.len() == 1 {
                                        if let crate::lang::model::PatternElem::UnivElem(e) =
                                            &pat.elements[0]
                                        {
                                            if e.name == format!("${}", node) {
                                                if let crate::lang::model::InterpedValue::Literal(
                                                    lit,
                                                ) = &case.value
                                                {
                                                    return Some(lit.to_string() == "true");
                                                }
                                            }
                                        }
                                    } else if pat.elements.iter().all(|e| {
                                        matches!(e, crate::lang::model::PatternElem::Wildcard)
                                    }) {
                                        if let crate::lang::model::InterpedValue::Literal(lit) =
                                            &case.value
                                        {
                                            return Some(lit.to_string() == "true");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None
                })
                .unwrap_or(false)
        });
        if final_bv_length == 0 {
            final_bv_length = length;
            output.push(format!(
                "{}Concatenated bitvector length: {}",
                cmt, final_bv_length
            ));
        }
        output.push(format!(
            "(define-fun {} () (_ BitVec {}) {})",
            prop, final_bv_length, bv_str
        ));
        grammar_terms.push(prop.clone());
        prop_bitvecs.insert(prop.clone(), bv_str);
    }
    // 7. 生成逻辑操作
    output.push(format!("\n{}--- Logical Operations ---", cmt));
    output.push(format!(
        "(define-fun modal.and ((x (_ BitVec {})) (y (_ BitVec {}))) (_ BitVec {}) (bvand x y))",
        final_bv_length, final_bv_length, final_bv_length
    ));
    output.push(format!(
        "(define-fun modal.or ((x (_ BitVec {})) (y (_ BitVec {}))) (_ BitVec {}) (bvor x y))",
        final_bv_length, final_bv_length, final_bv_length
    ));
    output.push(format!(
        "(define-fun modal.not ((x (_ BitVec {}))) (_ BitVec {}) (bvnot x))",
        final_bv_length, final_bv_length
    ));
    output.push("".to_string());
    // 8. 生成 modal 算子（自动获取 tran_name）
    let tran_name = {
        let mut found = None;
        for cmd in &lsfile.cmds {
            if let crate::parser::LSCmd::ThSpecCmd(thcmd) = cmd {
                if thcmd.name == "set-tran" {
                    let name = thcmd
                        .args
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        found = Some(name);
                        break;
                    }
                }
            }
        }
        found.unwrap_or_else(|| "R".to_string())
    };
    output.push(format!("{}--- Modal Operators ---", cmt));
    output.extend(generate_modal_operator_functions(
        &loaded_models,
        final_bv_length,
        &tran_name,
    ));
    // 9. 生成 synth-fun
    let grammar_terms_str = grammar_terms
        .iter()
        .map(|s| format!("      {}", s))
        .collect::<Vec<_>>()
        .join("\n");
    output.push(format!("\n{}--- Synthesis Task ---", cmt));
    output.push(format!(
        "(synth-fun separator () (_ BitVec {})",
        final_bv_length
    ));
    output.push(format!("  ((Start (_ BitVec {})))", final_bv_length));
    output.push(format!("  ((Start (_ BitVec {})", final_bv_length));
    output.push("    ( (modal.and Start Start)".to_string());
    output.push("      (modal.or Start Start)".to_string());
    output.push("      (modal.not Start)".to_string());
    output.push("      (modal.box Start)".to_string());
    output.push("      (modal.dia Start)".to_string());
    output.push(grammar_terms_str);
    output.push("    )".to_string());
    output.push("  ))".to_string());
    output.push(")".to_string());
    output.push("".to_string());
    // 10. 生成 constraint
    let mut current_offset = 0;
    for (i, model) in loaded_models.iter().enumerate() {
        let is_true_model = i < num_true_models;
        let target_bit = if is_true_model { "1" } else { "0" };
        let bit_to_check_index = current_offset;
        output.push(format!(
            "(constraint (= ((_ extract {} {} ) separator) #b{})) {}Model {}",
            bit_to_check_index, bit_to_check_index, target_bit, cmt, model.name
        ));
        current_offset += model
            .univ_defs
            .iter()
            .find(|u| u.sort.to_string() == "U")
            .map(|u| u.elems.len())
            .unwrap_or(0);
    }
    output.push("\n(check-synth)".to_string());
    output.join("\n")
}
