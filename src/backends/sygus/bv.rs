use crate::backends::smt2::ast::{
    HornAtom, HornDecl, HornProg, HornRule, HornTerm, PredicatePolarity,
};
use crate::backends::sygus::optim::{self, BitwiseExpr};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use sygus_parser::{
    BitVec, Bool, Cmd, DTConsDecl, GrammarDef, Literal, Sort, SyGuSCmd, SyGuSTerm, Term,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BitPolarity {
    One,
    Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Obligation {
    state_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ChildRef {
    child_idx: usize,
    state_idx: usize,
    polarity: BitPolarity,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Clause {
    child_refs: Vec<ChildRef>,
    self_refs: Vec<Obligation>,
}

#[derive(Clone, Debug, Default)]
struct ConstructorPlan {
    obligations: BTreeMap<Obligation, Vec<Clause>>,
}

enum AtomRef {
    Child(ChildRef),
    Current(Obligation),
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

fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
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

enum BoolExpr {
    Const(bool),
    Dynamic(SyGuSTerm),
}

fn compact_bitvec(bits: Vec<BitExpr>) -> SyGuSTerm {
    let mut chunks = Vec::new();
    let mut const_bits = Vec::new();

    for bit in bits {
        match bit {
            BitExpr::Const(value) => const_bits.push(value),
            BitExpr::Dynamic(term) => {
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
    body
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
        None => BitExpr::Dynamic(Term::ite(cond, BitVec::const_(1, 1), BitVec::const_(0, 1))),
    }
}

fn and_exprs(conjuncts: Vec<BoolExpr>) -> BoolExpr {
    let mut dynamic = Vec::new();
    for conjunct in conjuncts {
        match conjunct {
            BoolExpr::Const(false) => return BoolExpr::Const(false),
            BoolExpr::Const(true) => {}
            BoolExpr::Dynamic(term) => dynamic.push(term),
        }
    }
    match dynamic.len() {
        0 => BoolExpr::Const(true),
        1 => BoolExpr::Dynamic(dynamic.remove(0)),
        _ => BoolExpr::Dynamic(Bool::and(dynamic)),
    }
}

fn or_exprs(disjuncts: Vec<BoolExpr>) -> BoolExpr {
    let mut dynamic = Vec::new();
    for disjunct in disjuncts {
        match disjunct {
            BoolExpr::Const(true) => return BoolExpr::Const(true),
            BoolExpr::Const(false) => {}
            BoolExpr::Dynamic(term) => dynamic.push(term),
        }
    }
    match dynamic.len() {
        0 => BoolExpr::Const(false),
        1 => BoolExpr::Dynamic(dynamic.remove(0)),
        _ => BoolExpr::Dynamic(Bool::or(dynamic)),
    }
}

fn classify_atom(
    atom: &HornAtom,
    head_arg: &HornTerm,
    head_vars: &HashMap<String, usize>,
    state_index: &HashMap<String, usize>,
    program: &HornProg,
) -> Result<AtomRef, String> {
    let (state, polarity) = pred_bit(program, &atom.pred)
        .ok_or_else(|| format!("missing predicate metadata for {}", atom.pred))?;
    let state_idx = *state_index
        .get(&state)
        .ok_or_else(|| format!("unknown state {state}"))?;
    match &atom.arg {
        HornTerm::Var(var) => {
            let child_idx = *head_vars
                .get(var)
                .ok_or_else(|| format!("body atom references non-child variable {var}"))?;
            Ok(AtomRef::Child(ChildRef {
                child_idx,
                state_idx,
                polarity,
            }))
        }
        arg if arg == head_arg => Ok(AtomRef::Current(Obligation { state_idx })),
        other => Err(format!(
            "unsupported recursive body atom target {}; expected a child variable or the current constructor term",
            other
        )),
    }
}

fn build_constructor_plan(
    program: &HornProg,
    states: &[String],
    state_index: &HashMap<String, usize>,
    cons: &DTConsDecl,
    rules: &[HornRule],
) -> Result<ConstructorPlan, String> {
    let mut plan = ConstructorPlan::default();
    for rule in rules {
        let (head_state, head_polarity) = pred_bit(program, &rule.head.pred)
            .ok_or_else(|| format!("missing predicate metadata for {}", rule.head.pred))?;
        if head_polarity != BitPolarity::One {
            continue;
        }
        let head_state_idx = *state_index
            .get(&head_state)
            .ok_or_else(|| format!("unknown state {head_state}"))?;
        let head_args = match &rule.head.arg {
            HornTerm::App { fun, args } if fun == &cons.name => args.as_slice(),
            HornTerm::Var(name) if name == &cons.name && cons.args.is_empty() => &[],
            _ => continue,
        };
        let mut head_vars = HashMap::new();
        for (idx, arg) in head_args.iter().enumerate() {
            let HornTerm::Var(name) = arg else {
                return Err(format!(
                    "constructor head for {} must use variables only: {}",
                    cons.name, rule.head.arg
                ));
            };
            head_vars.insert(name.clone(), idx);
        }

        let mut clause = Clause {
            child_refs: Vec::new(),
            self_refs: Vec::new(),
        };
        for atom in &rule.body_atoms {
            match classify_atom(atom, &rule.head.arg, &head_vars, state_index, program)? {
                AtomRef::Child(child) => clause.child_refs.push(child),
                AtomRef::Current(obligation) => clause.self_refs.push(obligation),
            }
        }
        clause.child_refs.sort();
        clause.self_refs.sort();
        plan.obligations
            .entry(Obligation {
                state_idx: head_state_idx,
            })
            .or_default()
            .push(clause);
    }

    for state_idx in 0..states.len() {
        plan.obligations
            .entry(Obligation { state_idx })
            .or_default();
    }

    for clauses in plan.obligations.values_mut() {
        let mut seen = HashSet::new();
        clauses.retain(|clause| seen.insert(clause.clone()));
    }

    Ok(plan)
}

fn visit_obligation(
    obligation: Obligation,
    plan: &ConstructorPlan,
    visiting: &mut HashSet<Obligation>,
    visited: &mut HashSet<Obligation>,
    order: &mut Vec<Obligation>,
) -> Result<(), String> {
    if visited.contains(&obligation) {
        return Ok(());
    }
    if !visiting.insert(obligation) {
        return Err(format!(
            "recursive constructor obligations contain a cycle at state {}",
            obligation.state_idx
        ));
    }

    if let Some(clauses) = plan.obligations.get(&obligation) {
        for clause in clauses {
            for dep in &clause.self_refs {
                visit_obligation(*dep, plan, visiting, visited, order)?;
            }
        }
    }

    visiting.remove(&obligation);
    visited.insert(obligation);
    order.push(obligation);
    Ok(())
}

fn clause_expr(
    clause: &Clause,
    const_helpers: &HashMap<Obligation, bool>,
    helper_exprs: &HashMap<Obligation, SyGuSTerm>,
) -> Result<BoolExpr, String> {
    let mut conjuncts = Vec::new();
    for child in &clause.child_refs {
        let bit = BitVec::extract(
            child.state_idx,
            child.state_idx,
            Term::var(&arg_name(child.child_idx)),
        );
        let value = match child.polarity {
            BitPolarity::One => 1,
            BitPolarity::Zero => 0,
        };
        conjuncts.push(BoolExpr::Dynamic(Bool::eq(bit, BitVec::const_(value, 1))));
    }
    for dep in &clause.self_refs {
        if let Some(value) = const_helpers.get(dep) {
            conjuncts.push(BoolExpr::Const(*value));
        } else if let Some(term) = helper_exprs.get(dep) {
            conjuncts.push(BoolExpr::Dynamic(term.clone()));
        } else {
            return Err(format!(
                "unresolved constructor obligation dependency for state {}",
                dep.state_idx
            ));
        }
    }
    Ok(and_exprs(conjuncts))
}

fn obligation_expr_with_resolved_deps(
    clauses: &[Clause],
    const_helpers: &HashMap<Obligation, bool>,
    helper_exprs: &HashMap<Obligation, SyGuSTerm>,
) -> Result<BoolExpr, String> {
    let disjuncts = clauses
        .iter()
        .map(|clause| clause_expr(clause, const_helpers, helper_exprs))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(or_exprs(disjuncts))
}

pub fn compile_one_way(
    program: &HornProg,
    synth_name: &str,
    grammar: GrammarDef,
    optimize: bool,
) -> Vec<SyGuSCmd> {
    let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];

    let mut states = BTreeSet::new();
    let mut constructors = Vec::new();
    for info in program.predicates.values() {
        states.insert(info.state.clone());
    }
    for decl in &program.decls {
        if let HornDecl::DeclareDatatypes(_, decls) = decl {
            constructors.extend(decls.iter().flatten().cloned());
        }
    }
    let states = states.into_iter().collect::<Vec<_>>();
    let state_index = states
        .iter()
        .enumerate()
        .map(|(idx, state)| (state.clone(), idx))
        .collect::<HashMap<_, _>>();
    let bv_sort = Sort::bitvec(states.len().max(1));

    let constructor_names = constructors
        .iter()
        .map(|cons| cons.name.clone())
        .collect::<BTreeSet<_>>();
    let mut rules_by_constructor: BTreeMap<String, Vec<HornRule>> = BTreeMap::new();
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

    for cons in &constructors {
        let rules = rules_by_constructor
            .get(&cons.name)
            .cloned()
            .unwrap_or_default();
        let plan = build_constructor_plan(program, &states, &state_index, cons, &rules)
            .unwrap_or_else(|err| {
                panic!("failed to build constructor plan for {}: {err}", cons.name)
            });

        let mut order = Vec::new();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        for obligation in plan.obligations.keys().copied() {
            visit_obligation(obligation, &plan, &mut visiting, &mut visited, &mut order)
                .unwrap_or_else(|err| panic!("{err} for constructor {}", cons.name));
        }

        let mut const_helpers = HashMap::new();
        let mut helper_exprs = HashMap::new();
        for obligation in order {
            let clauses = plan
                .obligations
                .get(&obligation)
                .cloned()
                .unwrap_or_default();
            match obligation_expr_with_resolved_deps(&clauses, &const_helpers, &helper_exprs)
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to inline constructor obligation for {}: {err}",
                        cons.name
                    )
                }) {
                BoolExpr::Const(value) => {
                    const_helpers.insert(obligation, value);
                }
                BoolExpr::Dynamic(body) => {
                    helper_exprs.insert(obligation, body);
                }
            }
        }
        if log::log_enabled!(log::Level::Debug) {
            let entries = states
                .iter()
                .enumerate()
                .filter_map(|(state_idx, state)| {
                    const_helpers
                        .get(&Obligation { state_idx })
                        .map(|value| (state.clone(), *value))
                })
                .collect::<Vec<_>>();
            log::debug!(
                "sygus constant obligations for {}: {}/{}",
                cons.name,
                entries.len(),
                states.len()
            );
            entries
                .iter()
                .for_each(|(state, value)| log::debug!("  {} = {}\n", state, value))
        }

        let mut builder = Cmd::define_fun(&cons.name);
        let arg_names = (0..cons.args.len()).map(arg_name).collect::<Vec<_>>();
        for idx in 0..cons.args.len() {
            builder = builder.arg(&arg_names[idx], bv_sort.clone());
        }
        let bool_bits = states
            .iter()
            .enumerate()
            .map(|(state_idx, state)| {
                if let Some(value) = const_helpers.get(&Obligation { state_idx }) {
                    BoolExpr::Const(*value)
                } else if let Some(expr) = helper_exprs.get(&Obligation { state_idx }) {
                    BoolExpr::Dynamic(expr.clone())
                } else {
                    panic!(
                        "missing inlined constructor obligation for {} state {}",
                        cons.name, state
                    );
                }
            })
            .collect::<Vec<_>>();
        let lift_bits = bool_bits
            .iter()
            .map(|expr| match expr {
                BoolExpr::Const(value) => BitwiseExpr::Const(*value),
                BoolExpr::Dynamic(term) => BitwiseExpr::Expr(term.clone()),
            })
            .collect::<Vec<_>>();
        let body = (if optimize {
            optim::try_lift_native_bv(&lift_bits, &arg_names)
        } else {
            None
        })
        .unwrap_or_else(|| {
            let bits = bool_bits
                .into_iter()
                .map(|expr| match expr {
                    BoolExpr::Const(value) => BitExpr::Const(value),
                    BoolExpr::Dynamic(term) => bool_to_bit(term),
                })
                .collect::<Vec<_>>();
            compact_bitvec(bits)
        });
        constructor_cmds.push(builder.ret(bv_sort.clone()).body(body).build());
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
