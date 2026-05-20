use crate::backends::smt2::ast::{HornAtom, HornProg, HornRule, HornTerm, PredicatePolarity};
use crate::backends::sygus::optim::{self, BitwiseExpr};
use crate::compile::{TermConstructor, TermSignature};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};
use sygus_parser::{BitVec, Bool, Cmd, GrammarDef, Literal, Sort, SyGuSCmd, SyGuSTerm, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BitPolarity {
    One,
    Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Obligation {
    bit_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ChildRef {
    child_idx: usize,
    bit_idx: usize,
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

fn pred_bit<'a>(program: &'a HornProg, pred: &str) -> Option<(&'a str, &'a str, BitPolarity)> {
    let info = program.predicates.get(pred)?;
    let polarity = match info.polarity {
        PredicatePolarity::Positive => BitPolarity::One,
        PredicatePolarity::Negative => BitPolarity::Zero,
    };
    Some((&info.state, &info.sort, polarity))
}

#[derive(Clone, Debug)]
pub struct SortLayout {
    pub states: Vec<String>,
    pub index: HashMap<String, usize>,
}

pub fn sort_widths(program: &HornProg) -> HashMap<String, usize> {
    layouts_by_sort(program)
        .into_iter()
        .map(|(sort, layout)| (sort, layout.states.len().max(1)))
        .collect()
}

pub fn layouts_by_sort(program: &HornProg) -> HashMap<String, SortLayout> {
    let mut live_preds = HashSet::default();
    for rule in &program.rules {
        live_preds.insert(rule.head.pred.clone());
        for atom in &rule.body_atoms {
            live_preds.insert(atom.pred.clone());
        }
    }
    for query in &program.queries {
        for atom in &query.body_atoms {
            live_preds.insert(atom.pred.clone());
        }
    }
    let mut states_by_sort: HashMap<String, BTreeSet<String>> = HashMap::default();
    for (pred, info) in &program.predicates {
        if !live_preds.contains(pred) {
            continue;
        }
        states_by_sort
            .entry(info.sort.clone())
            .or_default()
            .insert(info.state.clone());
    }
    states_by_sort
        .into_iter()
        .map(|(sort, states)| {
            let states = states.into_iter().collect::<Vec<_>>();
            let index = states
                .iter()
                .enumerate()
                .map(|(idx, state)| (state.clone(), idx))
                .collect();
            (sort, SortLayout { states, index })
        })
        .collect()
}

fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
}

fn obligation_binding_name(cons_name: &str, bit_idx: usize) -> String {
    format!("__sygus_{}_b{}", cons_name.replace('.', "_"), bit_idx)
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
    head_vars: &HashMap<String, (usize, String)>,
    result_sort: &str,
    layouts: &HashMap<String, SortLayout>,
    program: &HornProg,
) -> Result<AtomRef, String> {
    let (state, sort, polarity) = pred_bit(program, &atom.pred)
        .ok_or_else(|| format!("missing predicate metadata for {}", atom.pred))?;
    match &atom.arg {
        HornTerm::Var(var) => {
            let (child_idx, child_sort) = head_vars
                .get(var)
                .ok_or_else(|| format!("body atom references non-child variable {var}"))?;
            let layout = layouts
                .get(child_sort)
                .ok_or_else(|| format!("missing local state layout for sort {child_sort}"))?;
            let bit_idx = *layout
                .index
                .get(state)
                .ok_or_else(|| format!("state {state} does not belong to sort {child_sort}"))?;
            Ok(AtomRef::Child(ChildRef {
                child_idx: *child_idx,
                bit_idx,
                polarity,
            }))
        }
        arg if arg == head_arg => {
            if sort != result_sort {
                return Err(format!(
                    "current-term atom for {} has sort {}, expected {}",
                    atom.pred, sort, result_sort
                ));
            }
            let layout = layouts
                .get(result_sort)
                .ok_or_else(|| format!("missing local state layout for sort {result_sort}"))?;
            let bit_idx = *layout
                .index
                .get(state)
                .ok_or_else(|| format!("state {state} does not belong to sort {result_sort}"))?;
            Ok(AtomRef::Current(Obligation { bit_idx }))
        }
        other => Err(format!(
            "unsupported recursive body atom target {}; expected a child variable or the current constructor term",
            other
        )),
    }
}

fn build_constructor_plan(
    program: &HornProg,
    layouts: &HashMap<String, SortLayout>,
    cons: &TermConstructor,
    rules: &[HornRule],
) -> Result<ConstructorPlan, String> {
    let mut plan = ConstructorPlan::default();
    for rule in rules {
        let (head_state, head_sort, head_polarity) = pred_bit(program, &rule.head.pred)
            .ok_or_else(|| format!("missing predicate metadata for {}", rule.head.pred))?;
        if head_polarity != BitPolarity::One {
            continue;
        }
        if head_sort != cons.result_sort
            && !compatible_layouts(layouts, head_sort, &cons.result_sort)
        {
            continue;
        }
        let layout = layouts
            .get(&cons.result_sort)
            .ok_or_else(|| format!("missing local state layout for sort {}", cons.result_sort))?;
        let head_bit_idx = *layout.index.get(head_state).ok_or_else(|| {
            format!(
                "unknown state {head_state} for constructor sort {}",
                cons.result_sort
            )
        })?;
        let head_args = match &rule.head.arg {
            HornTerm::App { fun, args } if fun == &cons.name => args.as_slice(),
            HornTerm::Var(name) if name == &cons.name && cons.arg_sorts.is_empty() => &[],
            _ => continue,
        };
        let mut head_vars = HashMap::default();
        for (idx, arg) in head_args.iter().enumerate() {
            let HornTerm::Var(name) = arg else {
                return Err(format!(
                    "constructor head for {} must use variables only: {}",
                    cons.name, rule.head.arg
                ));
            };
            let arg_sort = cons
                .arg_sorts
                .get(idx)
                .cloned()
                .ok_or_else(|| format!("missing sort for argument {idx} of {}", cons.name))?;
            head_vars.insert(name.clone(), (idx, arg_sort));
        }

        let mut clause = Clause {
            child_refs: Vec::new(),
            self_refs: Vec::new(),
        };
        for atom in &rule.body_atoms {
            match classify_atom(
                atom,
                &rule.head.arg,
                &head_vars,
                &cons.result_sort,
                layouts,
                program,
            )? {
                AtomRef::Child(child) => clause.child_refs.push(child),
                AtomRef::Current(obligation) => clause.self_refs.push(obligation),
            }
        }
        clause.child_refs.sort();
        clause.self_refs.sort();
        plan.obligations
            .entry(Obligation {
                bit_idx: head_bit_idx,
            })
            .or_default()
            .push(clause);
    }

    if let Some(layout) = layouts.get(&cons.result_sort) {
        for bit_idx in 0..layout.states.len() {
            plan.obligations.entry(Obligation { bit_idx }).or_default();
        }
    }

    for clauses in plan.obligations.values_mut() {
        let mut seen = HashSet::default();
        clauses.retain(|clause| seen.insert(clause.clone()));
    }

    Ok(plan)
}

fn compatible_layouts(
    layouts: &HashMap<String, SortLayout>,
    lhs_sort: &str,
    rhs_sort: &str,
) -> bool {
    let Some(lhs) = layouts.get(lhs_sort) else {
        return false;
    };
    let Some(rhs) = layouts.get(rhs_sort) else {
        return false;
    };
    lhs.states == rhs.states
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
            obligation.bit_idx
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
    helper_names: &HashMap<Obligation, String>,
) -> Result<BoolExpr, String> {
    let mut conjuncts = Vec::new();
    for child in &clause.child_refs {
        let bit = BitVec::extract(
            child.bit_idx,
            child.bit_idx,
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
        } else if let Some(name) = helper_names.get(dep) {
            conjuncts.push(BoolExpr::Dynamic(Term::var(name)));
        } else {
            return Err(format!(
                "unresolved constructor obligation dependency for state {}",
                dep.bit_idx
            ));
        }
    }
    Ok(and_exprs(conjuncts))
}

fn obligation_expr_with_resolved_deps(
    clauses: &[Clause],
    const_helpers: &HashMap<Obligation, bool>,
    helper_names: &HashMap<Obligation, String>,
) -> Result<BoolExpr, String> {
    let disjuncts = clauses
        .iter()
        .map(|clause| clause_expr(clause, const_helpers, helper_names))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(or_exprs(disjuncts))
}

pub fn compile_one_way(
    program: &HornProg,
    synth_name: &str,
    grammar: GrammarDef,
    sig: &TermSignature,
    start_sort: &str,
    layouts: Option<HashMap<String, SortLayout>>,
    optimize: bool,
) -> Vec<SyGuSCmd> {
    let total_start = Instant::now();
    let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];

    let layouts = layouts.unwrap_or_else(|| layouts_by_sort(program));
    let constructors = sig.constructors.clone();

    let bucketing_start = Instant::now();
    let constructor_names = constructors
        .iter()
        .map(|cons| cons.name.clone())
        .collect::<BTreeSet<_>>();
    let mut rules_by_constructor: BTreeMap<String, Vec<HornRule>> = BTreeMap::new();
    for rule in &program.rules {
        let Some((head_state, _head_sort, head_polarity)) = pred_bit(program, &rule.head.pred)
        else {
            continue;
        };
        if head_polarity != BitPolarity::One || head_state.is_empty() {
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
    let bucketing_elapsed = bucketing_start.elapsed();

    let mut constructor_cmds = Vec::new();
    let mut plan_elapsed = Duration::ZERO;
    let mut order_elapsed = Duration::ZERO;
    let mut resolve_elapsed = Duration::ZERO;
    let mut assemble_elapsed = Duration::ZERO;

    for cons in &constructors {
        let rules = rules_by_constructor
            .get(&cons.name)
            .cloned()
            .unwrap_or_default();
        let plan_start = Instant::now();
        let plan = build_constructor_plan(program, &layouts, cons, &rules).unwrap_or_else(|err| {
            panic!("failed to build constructor plan for {}: {err}", cons.name)
        });
        plan_elapsed += plan_start.elapsed();

        let order_start = Instant::now();
        let mut order = Vec::new();
        let mut visiting = HashSet::default();
        let mut visited = HashSet::default();
        for obligation in plan.obligations.keys().copied() {
            visit_obligation(obligation, &plan, &mut visiting, &mut visited, &mut order)
                .unwrap_or_else(|err| panic!("{err} for constructor {}", cons.name));
        }
        order_elapsed += order_start.elapsed();

        let resolve_start = Instant::now();
        let mut const_helpers = HashMap::default();
        let mut helper_names = HashMap::default();
        let arg_names = (0..cons.arg_sorts.len()).map(arg_name).collect::<Vec<_>>();
        // Distinct obligation RHS values: identical SyGuS term reuses one `let` binding.
        let mut distinct_obligations: Vec<(SyGuSTerm, String)> = Vec::new();
        let mut let_bindings = Vec::new();
        for obligation in order {
            let clauses = plan
                .obligations
                .get(&obligation)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            match obligation_expr_with_resolved_deps(clauses, &const_helpers, &helper_names)
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
                    let binding_name =
                        if let Some((_, name)) = distinct_obligations.iter().find(|(t, _)| t == &body)
                        {
                            name.clone()
                        } else {
                            let name = obligation_binding_name(&cons.name, obligation.bit_idx);
                            distinct_obligations.push((body.clone(), name.clone()));
                            let_bindings.push((name.clone(), body));
                            name
                        };
                    helper_names.insert(obligation, binding_name);
                }
            }
        }
        resolve_elapsed += resolve_start.elapsed();
        if log::log_enabled!(log::Level::Debug) {
            let entries = layouts[&cons.result_sort]
                .states
                .iter()
                .enumerate()
                .filter_map(|(bit_idx, state): (usize, &String)| {
                    const_helpers
                        .get(&Obligation { bit_idx })
                        .map(|value| (state.clone(), *value))
                })
                .collect::<Vec<_>>();
            log::info!(
                "sygus constant obligations for {}: {}/{}",
                cons.name,
                entries.len(),
                layouts[&cons.result_sort].states.len()
            );
        }

        let assemble_start = Instant::now();
        let mut builder = Cmd::define_fun(&cons.name);
        for (idx, arg_sort) in cons.arg_sorts.iter().enumerate() {
            builder = builder.arg(&arg_names[idx], Sort::from_name(arg_sort));
        }
        let bool_bits = layouts[&cons.result_sort]
            .states
            .iter()
            .enumerate()
            .map(|(bit_idx, state)| {
                if let Some(value) = const_helpers.get(&Obligation { bit_idx }) {
                    BoolExpr::Const(*value)
                } else if let Some(name) = helper_names.get(&Obligation { bit_idx }) {
                    BoolExpr::Dynamic(Term::var(name))
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
        let body = let_bindings
            .into_iter()
            .rev()
            .fold(body, |acc, (name, value)| Term::let_binding(vec![(name, value)], acc));
        constructor_cmds.push(builder.ret(Sort::from_name(&cons.result_sort)).body(body).build());
        let assemble_cons_elapsed = assemble_start.elapsed();
        assemble_elapsed += assemble_cons_elapsed;
        log::info!(
            "sygus BV constructor {}: rules={}, obligations={}, plan={:?}, order={:?}, resolve={:?}, assemble={:?}",
            cons.name,
            rules.len(),
            plan.obligations.len(),
            plan_start.elapsed(),
            order_start.elapsed(),
            resolve_start.elapsed(),
            assemble_cons_elapsed,
        );
    }

    cmds.extend(constructor_cmds);

    cmds.push(
        Cmd::synth_fun(synth_name)
            .ret(Sort::from_name(start_sort))
            .grammar(grammar)
            .build(),
    );

    log::info!(
        "sygus BV emission summary: constructors={}, rule_bucketing={:?}, plan={:?}, order={:?}, resolve={:?}, assemble={:?}, total={:?}",
        constructors.len(),
        bucketing_elapsed,
        plan_elapsed,
        order_elapsed,
        resolve_elapsed,
        assemble_elapsed,
        total_start.elapsed(),
    );

    for rule in &program.rules {
        if rule.head.pred == "Goal" {
            for atom in &rule.body_atoms {
                let Some((state, _sort, polarity)) = pred_bit(program, &atom.pred) else {
                    continue;
                };
                if atom.pred == "Goal" {
                    continue;
                }
                let Some(info) = program.predicates.get(&atom.pred) else {
                    continue;
                };
                if info.sort != start_sort && !compatible_layouts(&layouts, &info.sort, start_sort)
                {
                    continue;
                }
                let Some(bit_idx) = layouts
                    .get(start_sort)
                    .and_then(|layout| layout.index.get(state))
                    .copied()
                else {
                    continue;
                };
                let value = match polarity {
                    BitPolarity::One => 1,
                    BitPolarity::Zero => 0,
                };
                cmds.push(
                    Cmd::constraint()
                        .body(Bool::eq(
                            BitVec::extract(bit_idx, bit_idx, Term::var(synth_name)),
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
