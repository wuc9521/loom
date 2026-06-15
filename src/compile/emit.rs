//! Unified SyGuS-BV command emission.
//!
//! Pipeline: `compile_chc` → `saturate::saturate` →
//! `BitLayout::from_horn_prog` → `aggregate_constructor_bits` →
//! `lower_constraints` → `emit_sygus_cmds`.
//!
//! After saturation, every rule's body atoms reference only variables
//! bound by the rule's head term (child arg refs) or are intersection
//! atoms on the head's term itself (self-loops / helper preds). Pop
//! atoms with free `__upN` vars are eliminated by saturation. Rules that
//! still carry such residuals are skipped — saturation incompleteness is
//! the only source of incompleteness in the emitted program.

use crate::backends::smt2::ast::{HornAtom, HornProg, HornTerm};
use crate::compile::bexpr::{arg_name, self_bit_name, BoolExpr};
use crate::compile::context::CompileContext;
use crate::compile::layout::BitLayout;
use crate::compile::optim;
use crate::compile::saturate;
use crate::compile::syntax::{state_name, TermSignature};
use crate::compile::types::SemanticConstraint;
use rustc_hash::FxHashMap as HashMap;
use sygus_parser::{
    BitVec, Bool, Cmd, GrammarDef, Literal, Sort, SortedVar, SyGuSCmd, SyGuSTerm, Term,
};

pub struct ConstructorBits {
    pub name: String,
    pub arg_sorts: Vec<String>,
    pub result_sort: String,
    pub bits: Vec<BoolExpr>,
}

pub struct SynthSig {
    pub name: String,
    pub args: Vec<SortedVar>,
    pub ret_sort: String,
}

#[derive(Clone, Debug)]
pub struct EmitOptions {
    pub optimize: bool,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self { optimize: true }
    }
}

pub fn compile_via_horn(ctx: CompileContext<'_>) -> Result<Vec<SyGuSCmd>, String> {
    let sig = ctx.sig.clone();
    let term_types = ctx.term_types;
    let synth_name = ctx.synth_name.clone();
    let synth_args = ctx.synth_args.clone();
    let constraints = ctx.semantic_program.constraints.clone();

    let mut prog = ctx.compile_chc()?;
    if std::env::var("DELIS_DUMP_PREDS").is_ok() {
        let mut by_sort: HashMap<String, usize> = HashMap::default();
        for (_, info) in &prog.predicates {
            *by_sort.entry(info.sort.clone()).or_default() += 1;
        }
        eprintln!("[predicates after compile_chc, before saturate] {by_sort:?}");
    }
    // Bit-link triples must be derived from the PRE-saturation HornProg —
    // saturation consumes POP rules into derived push rules. Stash the
    // triples here so fwd_decls::apply can use them after saturation.
    let bit_link_triples = crate::compile::bit_link::derive_bit_links(&prog);
    saturate::saturate(&mut prog);
    if std::env::var("DELIS_DUMP_PREDS").is_ok() {
        let mut by_sort: HashMap<String, usize> = HashMap::default();
        for (_, info) in &prog.predicates {
            *by_sort.entry(info.sort.clone()).or_default() += 1;
        }
        eprintln!("[predicates after saturate] {by_sort:?}");
    }

    let layout = BitLayout::from_horn_prog(&prog);
    if std::env::var("DELIS_DUMP_BIT_LAYOUT").is_ok() {
        let mut entries: Vec<(usize, &str, &str)> = layout
            .bit_of_pred_for_debug()
            .iter()
            .filter_map(|(name, bit)| {
                prog.predicates
                    .get(name)
                    .map(|info| (*bit, name.as_str(), info.sort.as_str()))
            })
            .collect();
        entries.sort_by_key(|(bit, _, sort)| (*sort, *bit));
        for (bit, name, sort) in entries {
            println!("layout {sort} bit={bit} pred={name}");
        }
    }
    let cons_bits = aggregate_constructor_bits(&prog, &layout, &sig)?;

    let sort_map: HashMap<String, Sort> = sig
        .sort_order()
        .iter()
        .map(|s| (s.clone(), Sort::from_name(s)))
        .collect();
    let grammar = sig.bv_grammar_with_sorts(term_types, &sort_map)?;

    let lowered = lower_constraints(&layout, &prog, &sig, &synth_name, &constraints)?;
    let synth_sig = SynthSig {
        name: synth_name,
        args: synth_args,
        ret_sort: sig.start_sort().to_string(),
    };
    let cmds = emit_sygus_cmds(
        &layout,
        &sig,
        cons_bits,
        grammar,
        synth_sig,
        lowered,
        EmitOptions::default(),
    )?;
    // Lower parent-referencing nullaries to fwd-declared synth-funs plus
    // a fixpoint constraint. No-op for specs without `(parent n)` (bit_link
    // triples are empty there).
    crate::compile::fwd_decls::apply(cmds, &prog, &bit_link_triples, &layout, &sig)
}

pub fn emit_sygus_cmds(
    layout: &BitLayout,
    sig: &TermSignature,
    cons_bits: Vec<ConstructorBits>,
    grammar: GrammarDef,
    synth: SynthSig,
    constraints: Vec<SyGuSTerm>,
    options: EmitOptions,
) -> Result<Vec<SyGuSCmd>, String> {
    let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];

    let mut seen_sort: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
    for sort in sig.sort_order() {
        cmds.push(SyGuSCmd::DefineSort(
            sort.clone(),
            Vec::new(),
            Sort::bitvec(layout.width(sort)),
        ));
        seen_sort.insert(sort.clone());
    }
    let param_width = layout.width(sig.start_sort());
    for arg in &synth.args {
        let sort_name = arg.sort.to_string();
        if seen_sort.insert(sort_name.clone()) {
            cmds.push(SyGuSCmd::DefineSort(
                sort_name,
                Vec::new(),
                Sort::bitvec(param_width),
            ));
        }
    }

    for cr in cons_bits {
        let arg_names: Vec<String> = (0..cr.arg_sorts.len()).map(arg_name).collect();
        let mut builder = Cmd::define_fun(&cr.name);
        for (idx, sort) in cr.arg_sorts.iter().enumerate() {
            builder = builder.arg(&arg_names[idx], Sort::from_name(sort));
        }
        let body = if options.optimize {
            optim::try_lift_native_bv(&cr.bits, &arg_names)
        } else {
            None
        }
        .unwrap_or_else(|| BoolExpr::resolve_closure(&cr.bits));
        cmds.push(
            builder
                .ret(Sort::from_name(&cr.result_sort))
                .body(body)
                .build(),
        );
    }

    let mut synth_cmd = Cmd::synth_fun(&synth.name)
        .ret(Sort::from_name(&synth.ret_sort))
        .grammar(grammar);
    for arg in synth.args {
        synth_cmd = synth_cmd.arg(&arg.name, arg.sort);
    }
    cmds.push(synth_cmd.build());

    for term in constraints {
        cmds.push(Cmd::constraint().body(term).build());
    }

    cmds.push(SyGuSCmd::CheckSynth);
    Ok(cmds)
}

pub fn aggregate_constructor_bits(
    prog: &HornProg,
    layout: &BitLayout,
    sig: &TermSignature,
) -> Result<Vec<ConstructorBits>, String> {
    // Index helper rules so body atoms referencing a helper can be inlined
    // instead of consuming a BV bit per helper. A predicate is a helper iff
    // its `PredicateInfo.state == name` (the marker `chc.rs::emit_helper_atom`
    // sets). Helpers carry intermediate sub-formulae of one constructor's
    // lowering — inlining them keeps the BV width proportional to the
    // bisim-quotient classes only.
    let mut helper_rules: HashMap<&str, Vec<&crate::backends::smt2::ast::HornRule>> =
        HashMap::default();
    for rule in &prog.rules {
        let head_pred = rule.head.pred.as_str();
        let is_helper = prog
            .predicates
            .get(head_pred)
            .map(|info| info.state == *head_pred)
            .unwrap_or(false);
        if is_helper {
            helper_rules.entry(head_pred).or_default().push(rule);
        }
    }

    let mut out = Vec::with_capacity(sig.constructors.len());
    for cons in &sig.constructors {
        let width = layout.width(&cons.result_sort);
        let mut bits: Vec<BoolExpr> = vec![BoolExpr::Const(false); width];

        for rule in &prog.rules {
            let HornTerm::App { fun, args: head_args } = &rule.head.arg else {
                continue;
            };
            if fun != &cons.name {
                continue;
            }
            let Some(var_to_arg) = head_arg_index_map(head_args) else {
                continue;
            };
            // Skip rules whose head is a helper — they're inlined at use sites,
            // not OR-ed into a head bit.
            if helper_rules.contains_key(rule.head.pred.as_str()) {
                continue;
            }
            let Some(head_bit) = layout.bit_of(&rule.head.pred) else {
                continue;
            };
            if head_bit >= width {
                return Err(format!(
                    "head bit {} out of range for {} (width {})",
                    head_bit, cons.result_sort, width
                ));
            }

            let mut visiting: rustc_hash::FxHashSet<&str> = rustc_hash::FxHashSet::default();
            let body_terms: Result<Vec<BoolExpr>, String> = rule
                .body_atoms
                .iter()
                .map(|atom| {
                    body_atom_to_bool(
                        atom,
                        &rule.head.arg,
                        &var_to_arg,
                        layout,
                        prog,
                        &helper_rules,
                        &mut visiting,
                    )
                })
                .collect();
            let body_terms = match body_terms {
                Ok(t) => t,
                Err(_) => continue, // Residual existential — skip rule.
            };
            let body = BoolExpr::and(body_terms);
            bits[head_bit] = BoolExpr::or(vec![
                std::mem::replace(&mut bits[head_bit], BoolExpr::Const(false)),
                body,
            ]);
        }

        out.push(ConstructorBits {
            name: cons.name.clone(),
            arg_sorts: cons.arg_sorts.clone(),
            result_sort: cons.result_sort.clone(),
            bits,
        });
    }
    Ok(out)
}

fn body_atom_to_bool<'p>(
    atom: &'p HornAtom,
    head_term: &HornTerm,
    var_to_arg: &HashMap<&str, usize>,
    layout: &BitLayout,
    prog: &'p HornProg,
    helper_rules: &HashMap<&'p str, Vec<&'p crate::backends::smt2::ast::HornRule>>,
    visiting: &mut rustc_hash::FxHashSet<&'p str>,
) -> Result<BoolExpr, String> {
    // Helper atoms (no BV bit allocated) are expanded inline as
    // `OR_r (AND r.body)` over the helper's defining rules. Helpers are
    // always applied to the enclosing constructor's head term, so their
    // body atoms reference vars in the same scope.
    if let Some(rules) = helper_rules.get(atom.pred.as_str()) {
        let pred = atom.pred.as_str();
        if !visiting.insert(pred) {
            // Cycle: sound under monotone LFP to substitute false.
            return Ok(BoolExpr::Const(false));
        }
        let mut disjuncts = Vec::with_capacity(rules.len());
        for rule in rules {
            let mut conjuncts = Vec::with_capacity(rule.body_atoms.len());
            let mut residual = false;
            for body_atom in &rule.body_atoms {
                match body_atom_to_bool(
                    body_atom,
                    &rule.head.arg,
                    var_to_arg,
                    layout,
                    prog,
                    helper_rules,
                    visiting,
                ) {
                    Ok(e) => conjuncts.push(e),
                    Err(_) => {
                        residual = true;
                        break;
                    }
                }
            }
            if !residual {
                disjuncts.push(BoolExpr::and(conjuncts));
            }
        }
        visiting.remove(pred);
        return Ok(BoolExpr::or(disjuncts));
    }

    let Some(bit) = layout.bit_of(&atom.pred) else {
        return Err(format!("unknown body pred {}", atom.pred));
    };
    match &atom.arg {
        HornTerm::Var(name) => {
            let Some(&arg_idx) = var_to_arg.get(name.as_str()) else {
                return Err(format!(
                    "body atom var `{name}` is not a head-arg variable"
                ));
            };
            Ok(BoolExpr::Dynamic(Bool::eq(
                BitVec::extract(bit, bit, Term::var(&arg_name(arg_idx))),
                BitVec::const_(1, 1),
            )))
        }
        HornTerm::App { .. } => {
            if &atom.arg != head_term {
                return Err(format!(
                    "non-intersection App body atom survived saturation: \
                     atom={atom} head_term={head_term}"
                ));
            }
            Ok(BoolExpr::Dynamic(Term::var(&self_bit_name(bit))))
        }
    }
}

fn head_arg_index_map(head_args: &[HornTerm]) -> Option<HashMap<&str, usize>> {
    let mut map: HashMap<&str, usize> = HashMap::default();
    for (idx, arg) in head_args.iter().enumerate() {
        match arg {
            HornTerm::Var(name) => {
                map.insert(name.as_str(), idx);
            }
            HornTerm::App { .. } => return None,
        }
    }
    Some(map)
}

pub fn lower_constraints(
    layout: &BitLayout,
    prog: &HornProg,
    sig: &TermSignature,
    synth_name: &str,
    constraints: &[SemanticConstraint],
) -> Result<Vec<SyGuSTerm>, String> {
    let mut out = Vec::with_capacity(constraints.len());
    for c in constraints {
        let pred_name = q_pred_name_for(prog, sig.start_sort(), &c.relation, &c.model, &c.fields)?;
        let bit = layout
            .bit_of(&pred_name)
            .ok_or_else(|| format!("no bit assigned for constraint pred `{pred_name}`"))?;
        let expected_bit = if c.expected { "1" } else { "0" };
        out.push(Bool::eq(
            BitVec::extract(bit, bit, Term::var(synth_name)),
            Term::lit(Literal::BinConst(expected_bit.to_string())),
        ));
    }
    Ok(out)
}

fn q_pred_name_for(
    prog: &HornProg,
    sort: &str,
    relation: &str,
    model: &str,
    fields: &[(String, crate::compile::types::AspectValue)],
) -> Result<String, String> {
    use crate::backends::smt2::ast::PredicatePolarity;
    use crate::compile::types::AspectKey;
    let aspect = AspectKey {
        relation: relation.to_string(),
        model: model.to_string(),
        fields: fields.to_vec(),
    };
    let aspect_key = state_name(&aspect);
    // Bisim may have merged several aspects onto one canonical state. The
    // map populated in `chc::compile` is the source of truth for which.
    let want_state = prog
        .aspect_state
        .get(&aspect_key)
        .cloned()
        .unwrap_or(aspect_key.clone());
    for (name, info) in &prog.predicates {
        if info.sort == sort
            && info.polarity == PredicatePolarity::Positive
            && info.state == want_state
        {
            return Ok(name.clone());
        }
    }
    Err(format!(
        "no q-pred found for relation={relation} sort={sort} aspect={aspect_key} (canonical={want_state})"
    ))
}
