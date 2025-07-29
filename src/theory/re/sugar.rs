use crate::lang::model::{
    SetUniv, MatchCase, Model, Pattern, PatternElem
};
use crate::lang::types::FunctionType;
use crate::parser::ast::LSTerm;
use crate::parser::{LSCmd, LSFile};
use std::collections::HashSet;
use sygus_parser::api::{Cmd, GTerm};
use sygus_parser::ast::{SyGuSGTerm, SyGuSTerm};
use sygus_parser::{Identifier, Literal, Sort, SyGuSCmd, SyGuSFile};

pub fn desugar(sygus: &SyGuSFile) -> LSFile {
    let mut char_set = HashSet::new();
    let mut models = Vec::new();
    let mut skip_definefun = HashSet::new();

    // 1. handle nullary String function, generate model, collect characters
    for (idx, cmd) in sygus.cmds.iter().enumerate() {
        if let SyGuSCmd::DefineFun(name, args, sort, body) = cmd {
            if args.is_empty() && sort.to_string() == "String" {
                // collect characters
                if let SyGuSTerm::Literal(lit) = body {
                    let s = lit.to_string();
                    let s = s.trim_matches('"');
                    for c in s.chars() {
                        char_set.insert(c);
                    }
                    // generate model
                    // let len = s.chars().count();
                    let univ = SetUniv {
                        name: "Pos".to_string(),
                        sort: Sort::from_name("Pos"),
                    };
                    // let mut func_interps = Vec::new(); // 移除，因为语法解析中没有实际解析 interp-fun
                    for c in &char_set {
                        let _fname = c.to_string();
                        let mut cases = Vec::new();
                        for (i, ch) in s.chars().enumerate() {
                            let pat = Pattern {
                                elements: vec![PatternElem::Var(format!("${}", i))],
                            };
                            let val = LSTerm::Literal(Literal::Bool(*c == ch));
                            cases.push(MatchCase {
                                pattern: Some(pat),
                                value: val,
                            });
                        }
                        // 移除 InterpFun 处理，因为语法解析中没有实际解析 interp-fun
                    }
                    let model = Model {
                        name: name.clone(),
                        type_params: vec![],
                        univ_defs: vec![univ],
                        // func_interps, // 移除，因为语法解析中没有实际解析 interp-fun
                        ext_cmds: vec![],
                        ext_ctx: None,
                    };
                    models.push(LSCmd::ModelDef(model));
                    skip_definefun.insert(idx);
                }
            }
        }
    }

    // 2. other commands, special handling constraint/synth-fun
    let mut output_cmds = Vec::new();
    for (idx, cmd) in sygus.cmds.iter().enumerate() {
        if skip_definefun.contains(&idx) {
            continue;
        }
        match cmd {
            SyGuSCmd::Constraint(term) => {
                // str.in_re -> models
                if let SyGuSTerm::Application(f, ts) = term {
                    if let Identifier::Symbol(fstr) = f {
                        if fstr == "str.in_re" && ts.len() == 2 {
                            let w = ts[0].clone();
                            let regex = ts[1].clone();
                            let new_term = SyGuSTerm::Application(
                                Identifier::Symbol("models".to_string()),
                                vec![w, regex],
                            );
                            output_cmds.push(LSCmd::As(SyGuSCmd::Constraint(new_term)));
                            continue;
                        }
                    }
                }
                // not(str.in_re ...) -> not(models ...)
                if let SyGuSTerm::Application(f, ts) = term {
                    if let Identifier::Symbol(fstr) = f {
                        if fstr == "not" && ts.len() == 1 {
                            if let SyGuSTerm::Application(f2, ts2) = &ts[0] {
                                if let Identifier::Symbol(f2str) = f2 {
                                    if f2str == "str.in_re" && ts2.len() == 2 {
                                        let w = ts2[0].clone();
                                        let regex = ts2[1].clone();
                                        let new_term = SyGuSTerm::Application(
                                            Identifier::Symbol("not".to_string()),
                                            vec![SyGuSTerm::Application(
                                                Identifier::Symbol("models".to_string()),
                                                vec![w, regex],
                                            )],
                                        );
                                        output_cmds.push(LSCmd::As(SyGuSCmd::Constraint(new_term)));
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
                // output other constraint commands
                output_cmds.push(LSCmd::As(cmd.clone()));
            }
            SyGuSCmd::SynthFun(name, args, sort, grammar) => {
                // replace str.to_re in grammar with character name
                let mut new_grammar = grammar.clone();
                if let Some(g) = &mut new_grammar {
                    for gr in &mut g.grouped_rule_lists {
                        for term in &mut gr.terms {
                            if let SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(f, ts)) = term {
                                if let Identifier::Symbol(fstr) = f {
                                    if fstr == "str.to_re" && ts.len() == 1 {
                                        if let SyGuSTerm::Literal(lit) = &ts[0] {
                                            let ch = lit.to_string().trim_matches('"').to_string();
                                            *term = GTerm::var(&ch);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                output_cmds.push(LSCmd::As(SyGuSCmd::SynthFun(
                    name.clone(),
                    args.clone(),
                    sort.clone(),
                    new_grammar,
                )));
            }
            _ => {
                // output other commands
                output_cmds.push(LSCmd::As(cmd.clone()));
            }
        }
    }
    let mut final_cmds = Vec::new();
    final_cmds.push(LSCmd::As(SyGuSCmd::SetLogic("RE".to_string())));
    let mut inserted_sort = false;
    for cmd in output_cmds.iter() {
        if !inserted_sort {
            if let LSCmd::DeclareFunc(func_type) = cmd
            {
                let params = func_type.get_params();
                let ret = func_type.get_ret();
                let name = func_type.get_name().unwrap_or(&"unknown".to_string()).clone();
                if params.len() == 1
                    && params[0].to_string() == "Pos"
                    && ret.to_string() == "Bool"
                {
                    final_cmds.push(LSCmd::As(Cmd::declare_sort(&name, 0).build()));
                    inserted_sort = true;
                }
            }
        }
        final_cmds.push(cmd.clone());
    }
    if !inserted_sort {
        // if there is no declare-fun, insert one
        final_cmds.insert(0, LSCmd::As(Cmd::declare_sort("Pos", 0).build()));
    }

    // 4. declare-fun a/b (Pos) Bool, insert after declare-sort
    let mut char_funs: Vec<_> = char_set.iter().map(|c| c.to_string()).collect();
    char_funs.sort();
    let mut insert_idx = final_cmds
        .iter()
        .position(|cmd| matches!(cmd, LSCmd::DeclareFunc(_)))
        .unwrap_or(1); // if there is no declare-fun, insert after declare-sort
    for fname in char_funs {
        final_cmds.insert(
            insert_idx,
            LSCmd::DeclareFunc(FunctionType::new_signature(
                fname,
                vec![Sort::from_name("Pos")],
                Sort::bool(),
            )),
        );
        insert_idx += 1;
    }

    // 5. insert all models
    // insert after declare-fun
    let mut model_insert_idx = final_cmds
        .iter()
        .position(|cmd| matches!(cmd, LSCmd::DeclareFunc(_)))
        .map(|idx| idx + char_set.len())
        .unwrap_or(final_cmds.len());
    for m in models {
        final_cmds.insert(model_insert_idx, m);
        model_insert_idx += 1;
    }

    LSFile { cmds: final_cmds }
}
