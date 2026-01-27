use crate::lang::model::{
    DefineUniv, FuncInterp, InterpedValue, MatchCase, Model, Pattern, PatternElem, UnivElem,
};
use crate::parser::{DelisCmd, DelisProg};
use crate::signature::FuncSig;
use std::collections::HashSet;
use sygus_parser::ast::{SyGuSGTerm, SyGuSTerm};
use sygus_parser::{Cmd, GTerm};
use sygus_parser::{Identifier, Literal, Sort, SyGuSCmd, SyGuSFile};

pub fn desugar(sygus: &SyGuSFile) -> DelisProg {
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
                    let len = s.chars().count();
                    let univ = DefineUniv {
                        sort: Sort::from_name("Pos"),
                        elems: (0..len)
                            .map(|i| UnivElem::new(&format!("${}", i)))
                            .collect(),
                    };
                    let mut func_interps = Vec::new();
                    for c in &char_set {
                        let fname = c.to_string();
                        let mut cases = Vec::new();
                        for (i, ch) in s.chars().enumerate() {
                            let pat = Pattern {
                                elements: vec![PatternElem::Var(format!("${}", i))],
                            };
                            let val = InterpedValue::Literal(Literal::Bool(*c == ch));
                            cases.push(MatchCase {
                                pattern: Some(pat),
                                value: val,
                            });
                        }
                        func_interps.push(FuncInterp::MatchInterp(fname, cases));
                    }
                    let model = Model {
                        name: name.clone(),
                        univ_defs: vec![univ],
                        func_interps,
                        ext_cmds: vec![],
                        ext_ctx: None,
                    };
                    models.push(DelisCmd::DefineModel(model));
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
                            output_cmds.push(DelisCmd::As(SyGuSCmd::Constraint(new_term)));
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
                                        output_cmds
                                            .push(DelisCmd::As(SyGuSCmd::Constraint(new_term)));
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
                // output other constraint commands
                output_cmds.push(DelisCmd::As(cmd.clone()));
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
                output_cmds.push(DelisCmd::As(SyGuSCmd::SynthFun(
                    name.clone(),
                    args.clone(),
                    sort.clone(),
                    new_grammar,
                )));
            }
            _ => {
                // output other commands
                output_cmds.push(DelisCmd::As(cmd.clone()));
            }
        }
    }
    let mut final_cmds = Vec::new();
    final_cmds.push(DelisCmd::As(SyGuSCmd::SetLogic("RE".to_string())));
    let mut inserted_sort = false;
    for cmd in output_cmds.iter() {
        if !inserted_sort {
            if let DelisCmd::DeclareFunc(FuncSig {
                name,
                arg_sorts,
                ret_sort,
            }) = cmd
            {
                if arg_sorts.len() == 1
                    && arg_sorts[0].to_string() == "Pos"
                    && ret_sort.to_string() == "Bool"
                {
                    final_cmds.push(DelisCmd::As(Cmd::declare_sort(name, 0).build()));
                    inserted_sort = true;
                }
            }
        }
        final_cmds.push(cmd.clone());
    }
    if !inserted_sort {
        // if there is no declare-fun, insert one
        final_cmds.insert(0, DelisCmd::As(Cmd::declare_sort("Pos", 0).build()));
    }

    // 4. declare-fun a/b (Pos) Bool, insert after declare-sort
    let mut char_funs: Vec<_> = char_set.iter().map(|c| c.to_string()).collect();
    char_funs.sort();
    let mut insert_idx = final_cmds
        .iter()
        .position(|cmd| matches!(cmd, DelisCmd::DeclareFunc(_)))
        .unwrap_or(1); // if there is no declare-fun, insert after declare-sort
    for fname in char_funs {
        final_cmds.insert(
            insert_idx,
            DelisCmd::DeclareFunc(FuncSig {
                name: fname,
                arg_sorts: vec![Sort::from_name("Pos")],
                ret_sort: Sort::bool(),
            }),
        );
        insert_idx += 1;
    }

    // 5. insert all models
    // insert after declare-fun
    let mut model_insert_idx = final_cmds
        .iter()
        .position(|cmd| matches!(cmd, DelisCmd::DeclareFunc(_)))
        .map(|idx| idx + char_set.len())
        .unwrap_or(final_cmds.len());
    for m in models {
        final_cmds.insert(model_insert_idx, m);
        model_insert_idx += 1;
    }

    DelisProg { cmds: final_cmds }
}
