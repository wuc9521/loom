use crate::chc::ast::{HornDecl, HornProg, HornTerm, PredicatePolarity};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use sygus_parser::api::{BitVec, Bool, Cmd, Term};
use sygus_parser::{DTConsDecl, GrammarDef, Literal, Sort, SyGuSCmd, SyGuSTerm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BitPolarity {
    One,
    Zero,
}

fn pred_bit(program: &HornProg, pred: &str) -> Option<(String, BitPolarity)> {
    let info = program.predicates.get(pred)?;
    let polarity = match info.polarity {
        PredicatePolarity::Positive => BitPolarity::One,
        PredicatePolarity::Negative => BitPolarity::Zero,
    };
    Some((info.state.clone(), polarity))
}

fn state_bit(states: &[String], state: &str, term: SyGuSTerm) -> SyGuSTerm {
    let idx = states.iter().position(|s| s == state).unwrap();
    BitVec::extract(idx, idx, term)
}

fn bit_eq_polarity(
    program: &HornProg,
    states: &[String],
    pred: &str,
    term: SyGuSTerm,
) -> Option<SyGuSTerm> {
    let (state, polarity) = pred_bit(program, pred)?;
    let bit = state_bit(states, &state, term);
    let value = match polarity {
        BitPolarity::One => 1,
        BitPolarity::Zero => 0,
    };
    Some(Bool::eq(bit, BitVec::const_(value, 1)))
}

fn bitvec_literal_from_lsb(bits: &[bool]) -> SyGuSTerm {
    let bitstring = bits
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect::<String>();
    Term::lit(Literal::BinConst(bitstring))
}

enum BitExpr {
    Const(bool),
    Dynamic(SyGuSTerm),
}

fn compact_bitvec(bits: Vec<BitExpr>) -> (SyGuSTerm, bool) {
    let mut chunks = Vec::new();
    let mut const_bits = Vec::new();
    let mut has_dynamic = false;

    for bit in bits {
        match bit {
            BitExpr::Const(value) => const_bits.push(value),
            BitExpr::Dynamic(term) => {
                has_dynamic = true;
                if !const_bits.is_empty() {
                    chunks.push(bitvec_literal_from_lsb(&const_bits));
                    const_bits.clear();
                }
                chunks.push(term);
            }
        }
    }
    if !const_bits.is_empty() {
        chunks.push(bitvec_literal_from_lsb(&const_bits));
    }

    let mut body = chunks
        .first()
        .cloned()
        .unwrap_or_else(|| BitVec::const_(0, 1));
    for chunk in chunks.iter().skip(1) {
        body = BitVec::concat(chunk.clone(), body);
    }
    (body, has_dynamic)
}

fn bool_literal(term: &SyGuSTerm) -> Option<bool> {
    match term {
        SyGuSTerm::Literal(Literal::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn bool_to_bit(cond: SyGuSTerm) -> BitExpr {
    match bool_literal(&cond) {
        Some(value) => BitExpr::Const(value),
        None => BitExpr::Dynamic(Term::app("bool->bv1", vec![cond])),
    }
}

fn bool_to_bv1_cmd() -> SyGuSCmd {
    Cmd::define_fun("bool->bv1")
        .arg("b", Sort::bool())
        .ret(Sort::bitvec(1))
        .body(Term::ite(
            Term::var("b"),
            BitVec::const_(1, 1),
            BitVec::const_(0, 1),
        ))
        .build()
}

fn is_true_term(term: &SyGuSTerm) -> bool {
    matches!(term, SyGuSTerm::Literal(Literal::Bool(true)))
}

fn constructor_state_disjuncts<'a>(
    program: &HornProg,
    states: &[String],
    rules: impl Iterator<Item = &'a crate::chc::ast::HornRule>,
    cons: &DTConsDecl,
    state: &str,
) -> Vec<SyGuSTerm> {
    let mut disjuncts = Vec::new();
    for rule in rules {
        let Some((head_state, head_polarity)) = pred_bit(program, &rule.head.pred) else {
            continue;
        };
        if head_state != state || head_polarity != BitPolarity::One {
            continue;
        }
        let args = match &rule.head.arg {
            HornTerm::App { args, .. } => args.as_slice(),
            HornTerm::Var(name) if name == &cons.name && cons.args.is_empty() => &[],
            _ => continue,
        };
        let mut vars = HashMap::new();
        for (idx, arg) in args.iter().enumerate() {
            if let HornTerm::Var(var) = arg {
                vars.insert(var.clone(), idx);
            }
        }
        let mut conjuncts = Vec::new();
        let mut supported = true;
        for atom in &rule.body_atoms {
            let HornTerm::Var(var) = &atom.arg else {
                supported = false;
                break;
            };
            let Some(child_idx) = vars.get(var) else {
                supported = false;
                break;
            };
            let Some(conjunct) = bit_eq_polarity(
                program,
                states,
                &atom.pred,
                Term::var(&format!("arg{}", child_idx)),
            ) else {
                supported = false;
                break;
            };
            conjuncts.push(conjunct);
        }
        if supported {
            disjuncts.push(match conjuncts.len() {
                0 => Bool::true_(),
                1 => conjuncts.remove(0),
                _ => Bool::and(conjuncts),
            });
        }
    }
    disjuncts
}

pub fn compile_one_way(program: &HornProg, synth_name: &str, grammar: GrammarDef) -> Vec<SyGuSCmd> {
    let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];

    let mut states = BTreeSet::new();
    let mut constructors = Vec::new();
    for info in program.predicates.values() {
        states.insert(info.state.clone());
    }
    for decl in &program.decls {
        match decl {
            HornDecl::DeclareDatatypes(_, decls) => {
                constructors.extend(decls.iter().flatten().cloned());
            }
            _ => {}
        }
    }
    let states = states.into_iter().collect::<Vec<_>>();
    let state_index = states
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.clone(), idx))
        .collect::<HashMap<_, _>>();
    let bv_sort = Sort::bitvec(states.len().max(1));

    let constructor_names = constructors
        .iter()
        .map(|cons| cons.name.clone())
        .collect::<BTreeSet<_>>();
    let mut rules_by_constructor: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for rule in &program.rules {
        let Some((head_state, head_polarity)) = pred_bit(program, &rule.head.pred) else {
            continue;
        };
        if head_polarity != BitPolarity::One || !state_index.contains_key(&head_state) {
            continue;
        }
        match &rule.head.arg {
            HornTerm::App { fun, .. } => {
                rules_by_constructor
                    .entry(fun.clone())
                    .or_default()
                    .push(rule.clone());
            }
            HornTerm::Var(name) if constructor_names.contains(name) => {
                rules_by_constructor
                    .entry(name.clone())
                    .or_default()
                    .push(rule.clone());
            }
            _ => {}
        }
    }

    let mut constructor_cmds = Vec::new();
    let mut uses_bool_to_bv1 = false;
    for cons in &constructors {
        let mut builder = Cmd::define_fun(&cons.name);
        for idx in 0..cons.args.len() {
            builder = builder.arg(&format!("arg{}", idx), bv_sort.clone());
        }
        let mut bits = Vec::new();
        for state in &states {
            let mut disjuncts = constructor_state_disjuncts(
                program,
                &states,
                rules_by_constructor.get(&cons.name).into_iter().flatten(),
                cons,
                state,
            );
            if cons.args.is_empty() {
                bits.push(BitExpr::Const(disjuncts.iter().any(is_true_term)));
            } else {
                let cond = match disjuncts.len() {
                    0 => Bool::false_(),
                    1 => disjuncts.remove(0),
                    _ => Bool::or(disjuncts),
                };
                bits.push(bool_to_bit(cond));
            }
        }
        let (body, has_dynamic_bits) = compact_bitvec(bits);
        uses_bool_to_bv1 |= has_dynamic_bits;
        constructor_cmds.push(builder.ret(bv_sort.clone()).body(body).build());
    }
    if uses_bool_to_bv1 {
        cmds.push(bool_to_bv1_cmd());
    }
    cmds.extend(constructor_cmds);

    cmds.push(
        Cmd::synth_fun(synth_name)
            .ret(bv_sort.clone())
            .grammar(grammar)
            .build(),
    );

    for rule in &program.rules {
        if rule.head.pred == "Goal" {
            for atom in &rule.body_atoms {
                let Some((state, polarity)) = pred_bit(program, &atom.pred) else {
                    continue;
                };
                if !state_index.contains_key(&state) {
                    continue;
                }
                let value = match polarity {
                    BitPolarity::One => 1,
                    BitPolarity::Zero => 0,
                };
                cmds.push(
                    Cmd::constraint()
                        .body(Bool::eq(
                            state_bit(&states, &state, Term::var(synth_name)),
                            BitVec::const_(value, 1),
                        ))
                        .build(),
                );
            }
        }
    }

    cmds.push(SyGuSCmd::CheckSynth);
    cmds
}
