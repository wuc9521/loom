
use crate::backends::smt2::ast::{HornAtom, HornProg, HornRule, HornTerm};
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
    saturate::saturate(&mut prog);
    if std::env::var("DELIS_DUMP_SAT").is_ok() {
        eprintln!("==== saturated rules pre-discharge ({}) ====", prog.rules.len());
        for r in &prog.rules {
            eprintln!("{r}");
        }
        eprintln!("==== end ====");
    }
    discharge_guards(&mut prog);

    if std::env::var("DELIS_DUMP_ONEWAY").is_ok() {
        eprintln!("==== one-way rules ({}) ====", prog.rules.len());
        for r in &prog.rules {
            eprintln!("{r}");
        }
        eprintln!("==== end one-way rules ====");
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
    emit_sygus_cmds(
        &layout,
        &sig,
        cons_bits,
        grammar,
        synth_sig,
        lowered,
        EmitOptions::default(),
    )
}

fn discharge_guards(prog: &mut HornProg) {
    let productive = productive_predicates(&prog.rules);
    let mut kept = Vec::with_capacity(prog.rules.len());
    'rule: for rule in std::mem::take(&mut prog.rules) {
        if is_guard_pred(&rule.head.pred) {
            continue;
        }
        let mut body = Vec::with_capacity(rule.body_atoms.len());
        for atom in rule.body_atoms {
            if is_guard_pred(&atom.pred) {
                if productive.contains(&atom.pred) {
                    continue;
                }
                continue 'rule;
            }
            body.push(atom);
        }
        kept.push(HornRule {
            body_atoms: body,
            head: rule.head,
        });
    }
    prog.rules = kept;
}

fn productive_predicates(rules: &[HornRule]) -> rustc_hash::FxHashSet<String> {
    let mut productive: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
    loop {
        let mut changed = false;
        for rule in rules {
            if productive.contains(&rule.head.pred) {
                continue;
            }
            if rule
                .body_atoms
                .iter()
                .all(|atom| productive.contains(&atom.pred))
            {
                productive.insert(rule.head.pred.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    productive
}

fn is_guard_pred(pred: &str) -> bool {
    pred.starts_with("__guard")
}

fn is_shape_pred(pred: &str) -> bool {
    pred.starts_with("__shape.")
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
    let mut out = Vec::with_capacity(sig.constructors.len());
    for cons in &sig.constructors {
        let width = layout.width(&cons.result_sort);
        let mut bits: Vec<BoolExpr> = vec![BoolExpr::Const(false); width];

        for rule in &prog.rules {
            let Some(var_to_arg) = rule_builds_cons(rule, cons) else {
                continue;
            };
            if prog.predicates.get(&rule.head.pred).map(|i| i.sort.as_str())
                != Some(cons.result_sort.as_str())
            {
                continue;
            }
            let Some(head_bit) = layout.bit_of(&rule.head.pred) else {
                continue;
            };
            if head_bit >= width {
                return Err(format!(
                    "head bit {head_bit} out of range for {} (width {width})",
                    cons.result_sort
                ));
            }

            let conditions: Result<Vec<BoolExpr>, String> = rule
                .body_atoms
                .iter()
                .filter(|atom| !is_shape_pred(&atom.pred))
                .map(|atom| body_atom_to_bool(atom, &rule.head.arg, &var_to_arg, layout))
                .collect();
            let body = BoolExpr::and(conditions?);
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

fn rule_builds_cons<'r>(
    rule: &'r HornRule,
    cons: &crate::compile::syntax::TermConstructor,
) -> Option<HashMap<&'r str, usize>> {
    match &rule.head.arg {
        HornTerm::App { fun, args } if fun.as_str() == cons.name.as_str() => {
            let mut map: HashMap<&str, usize> = HashMap::default();
            for (idx, arg) in args.iter().enumerate() {
                match arg {
                    HornTerm::Var(name) => {
                        map.insert(name.as_str(), idx);
                    }
                    HornTerm::App { .. } => return None,
                }
            }
            Some(map)
        }
        HornTerm::App { .. } => None,
        HornTerm::Var(_) => {
            let shape = rule
                .body_atoms
                .iter()
                .find(|atom| is_shape_pred(&atom.pred));
            match shape {
                Some(sel) if sel.pred == shape_pred_of(cons) => Some(HashMap::default()),
                Some(_) => None,
                None => Some(HashMap::default()),
            }
        }
    }
}

fn body_atom_to_bool(
    atom: &HornAtom,
    head_term: &HornTerm,
    var_to_arg: &HashMap<&str, usize>,
    layout: &BitLayout,
) -> Result<BoolExpr, String> {
    let bit = layout
        .bit_of(&atom.pred)
        .ok_or_else(|| format!("unknown body pred {}", atom.pred))?;
    if &atom.arg == head_term {
        return Ok(BoolExpr::Dynamic(Term::var(&self_bit_name(bit))));
    }
    match &atom.arg {
        HornTerm::Var(name) => {
            let &arg_idx = var_to_arg
                .get(name.as_str())
                .ok_or_else(|| format!("body var `{name}` is neither a child nor the node"))?;
            Ok(BoolExpr::Dynamic(Bool::eq(
                BitVec::extract(bit, bit, Term::var(&arg_name(arg_idx))),
                BitVec::const_(1, 1),
            )))
        }
        HornTerm::App { .. } => Err(format!(
            "non-self App body atom survived saturation: atom={atom} head={head_term}"
        )),
    }
}

fn shape_pred_of(cons: &crate::compile::syntax::TermConstructor) -> String {
    format!("__shape.{}.{}", cons.name, cons.arg_sorts.len())
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
