use crate::chc::ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule};
use crate::theory::Logic;
use std::collections::{HashMap, HashSet};
use sygus_parser::api::{BitVec, Cmd, Term};
use sygus_parser::{
    DTConsDecl, GrammarDef, GroupedRuleList, Identifier, Index, Literal, Sort, SortedVar, SyGuSCmd,
    SyGuSGTerm, SyGuSTerm,
};

pub fn to_sygus(
    program: &HornProg,
    synth_name: &str,
    optimize: bool,
) -> Result<Vec<SyGuSCmd>, String> {
    let ctors = collect_ctors(&program.decls)?;
    let ctor_names: HashSet<String> = ctors.iter().map(|c| c.name.clone()).collect();
    let preds = collect_preds(&program.decls);
    if preds.is_empty() {
        return Err("no unary Term predicates declared".to_string());
    }
    let pred_index = preds
        .iter()
        .enumerate()
        .map(|(i, p)| (p.clone(), i))
        .collect::<HashMap<_, _>>();
    let bv_sort = Sort::bitvec(preds.len());

    let mut cmds = Vec::new();
    cmds.push(SyGuSCmd::SetLogic(Logic::BV.to_string()));
    cmds.extend(emit_ctor_fns(
        program,
        &ctors,
        &preds,
        &pred_index,
        &bv_sort,
        optimize,
    )?);
    cmds.push(emit_synth_fun(&ctors, &bv_sort, synth_name)?);

    if let Some(constraint) = emit_goal_constraint(program, &pred_index, &ctor_names, synth_name)? {
        cmds.push(constraint);
    } else {
        cmds.extend(emit_query_constraints(
            &program.queries,
            &pred_index,
            &ctor_names,
            synth_name,
        )?);
    }

    cmds.push(SyGuSCmd::CheckSynth);
    Ok(cmds)
}

fn is_term_sort(sort: &Sort) -> bool {
    matches!(sort, Sort::Simple(id) if id.get_name() == "Term")
}

fn collect_ctors(decls: &[HornDecl]) -> Result<Vec<DTConsDecl>, String> {
    for decl in decls {
        if let HornDecl::DeclareDatatypes(sorts, decls) = decl {
            for (idx, sort) in sorts.iter().enumerate() {
                if sort.symbol == "Term" {
                    return Ok(decls
                        .get(idx)
                        .ok_or_else(|| "datatype decl missing for Term".to_string())?
                        .clone());
                }
            }
        }
    }
    Err("Term datatype not declared".to_string())
}

fn collect_preds(decls: &[HornDecl]) -> Vec<String> {
    let mut preds = Vec::new();
    for decl in decls {
        if let HornDecl::DeclareRel(name, args) = decl {
            if args.len() == 1 && is_term_sort(&args[0]) {
                preds.push(name.clone());
            }
        }
    }
    preds
}

fn emit_ctor_fns(
    program: &HornProg,
    ctors: &[DTConsDecl],
    preds: &[String],
    pred_index: &HashMap<String, usize>,
    bv_sort: &Sort,
    optimize: bool,
) -> Result<Vec<SyGuSCmd>, String> {
    let mut out = Vec::new();
    for ctor in ctors {
        let arg_names: Vec<String> = (0..ctor.args.len()).map(|i| format!("arg{}", i)).collect();
        let mut bits = Vec::new();
        for pred in preds {
            let mut rule_bits = Vec::new();
            for rule in &program.rules {
                if rule.head.pred != *pred {
                    continue;
                }
                let (head_ctor, head_vars) = parse_head_ctor(rule)?;
                if head_ctor != ctor.name {
                    continue;
                }
                if head_vars.len() != arg_names.len() {
                    return Err(format!("arity mismatch for constructor {}", ctor.name));
                }
                let var_map = build_var_map(&head_vars, &arg_names);
                rule_bits.push(rule_body_bit(rule, pred_index, &var_map)?);
            }
            bits.push(or_bits(rule_bits));
        }
        let mut body = concat_bits(&bits)?;
        if optimize {
            body = simplify_bv(&body, preds.len()).unwrap_or(body);
        }
        let mut builder = Cmd::define_fun(ctor.name.as_str());
        for name in &arg_names {
            builder = builder.arg(name, bv_sort.clone());
        }
        out.push(builder.ret(bv_sort.clone()).body(body).build());
    }
    Ok(out)
}

fn emit_synth_fun(
    ctors: &[DTConsDecl],
    bv_sort: &Sort,
    synth_name: &str,
) -> Result<SyGuSCmd, String> {
    let mut terms = Vec::new();
    for ctor in ctors {
        let args = (0..ctor.args.len())
            .map(|_| Term::var("Start"))
            .collect::<Vec<_>>();
        let term = if args.is_empty() {
            Term::var(&ctor.name)
        } else {
            Term::app(&ctor.name, args)
        };
        terms.push(SyGuSGTerm::SyGuSTerm(term));
    }
    if terms.is_empty() {
        return Err("no constructors found for Term datatype".to_string());
    }
    let grammar = GrammarDef {
        sorted_vars: vec![SortedVar {
            name: "Start".to_string(),
            sort: bv_sort.clone(),
        }],
        grouped_rule_lists: vec![GroupedRuleList {
            symbol: "Start".to_string(),
            sort: bv_sort.clone(),
            terms,
        }],
    };
    Ok(Cmd::synth_fun(synth_name)
        .ret(bv_sort.clone())
        .grammar(grammar)
        .build())
}

fn emit_goal_constraint(
    program: &HornProg,
    pred_index: &HashMap<String, usize>,
    ctor_names: &HashSet<String>,
    synth_name: &str,
) -> Result<Option<SyGuSCmd>, String> {
    let mut rule_bits = Vec::new();
    for rule in &program.rules {
        if rule.head.pred != "Goal" {
            continue;
        }
        let mut var_map = HashMap::new();
        match rule.vars.len() {
            0 => {}
            1 => {
                var_map.insert(rule.vars[0].name.clone(), synth_name.to_string());
            }
            _ => {
                return Err("Goal rules with multiple vars require saturation".to_string());
            }
        }
        rule_bits.push(rule_body_bit_term(
            &rule.body_atoms,
            pred_index,
            ctor_names,
            &var_map,
        )?);
    }
    if rule_bits.is_empty() {
        return Ok(None);
    }
    let bit = or_bits(rule_bits);
    Ok(Some(Cmd::constraint().body(eq_one(bit)).build()))
}

fn emit_query_constraints(
    queries: &[HornQuery],
    pred_index: &HashMap<String, usize>,
    ctor_names: &HashSet<String>,
    synth_name: &str,
) -> Result<Vec<SyGuSCmd>, String> {
    let mut out = Vec::new();
    for query in queries {
        let mut var_map = HashMap::new();
        match query.vars.len() {
            0 => {}
            1 => {
                var_map.insert(query.vars[0].name.clone(), synth_name.to_string());
            }
            _ => return Err("queries with multiple vars require saturation".to_string()),
        }
        let bit = rule_body_bit_term(&query.body_atoms, pred_index, ctor_names, &var_map)?;
        out.push(Cmd::constraint().body(eq_one(bit)).build());
    }
    Ok(out)
}

fn parse_head_ctor(rule: &HornRule) -> Result<(String, Vec<String>), String> {
    if rule.head.args.len() != 1 {
        return Err("head predicate must have exactly one argument".to_string());
    }
    let term = &rule.head.args[0];
    match term {
        SyGuSTerm::Identifier(id) => Ok((id.get_name().to_string(), Vec::new())),
        SyGuSTerm::Application(id, args) => {
            let mut vars = Vec::new();
            for arg in args {
                if let SyGuSTerm::Identifier(arg_id) = arg {
                    vars.push(arg_id.get_name().to_string());
                } else {
                    return Err("constructor args must be variables".to_string());
                }
            }
            Ok((id.get_name().to_string(), vars))
        }
        _ => Err("unsupported constructor term in head".to_string()),
    }
}

fn build_var_map(head_vars: &[String], arg_names: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (idx, name) in head_vars.iter().enumerate() {
        map.insert(name.clone(), arg_names[idx].clone());
    }
    map
}

fn rule_body_bit(
    rule: &HornRule,
    pred_index: &HashMap<String, usize>,
    var_map: &HashMap<String, String>,
) -> Result<SyGuSTerm, String> {
    if rule.body_atoms.is_empty() {
        return Ok(BitVec::const_(1, 1));
    }
    let mut bit = body_atom_bit(&rule.body_atoms[0], pred_index, var_map)?;
    for atom in rule.body_atoms.iter().skip(1) {
        bit = BitVec::bvand(bit, body_atom_bit(atom, pred_index, var_map)?);
    }
    Ok(bit)
}

fn rule_body_bit_term(
    body_atoms: &[HornAtom],
    pred_index: &HashMap<String, usize>,
    ctor_names: &HashSet<String>,
    var_map: &HashMap<String, String>,
) -> Result<SyGuSTerm, String> {
    if body_atoms.is_empty() {
        return Ok(BitVec::const_(1, 1));
    }
    let mut bit = body_atom_bit_term(&body_atoms[0], pred_index, ctor_names, var_map)?;
    for atom in body_atoms.iter().skip(1) {
        bit = BitVec::bvand(
            bit,
            body_atom_bit_term(atom, pred_index, ctor_names, var_map)?,
        );
    }
    Ok(bit)
}

fn body_atom_bit(
    atom: &HornAtom,
    pred_index: &HashMap<String, usize>,
    var_map: &HashMap<String, String>,
) -> Result<SyGuSTerm, String> {
    if atom.args.len() != 1 {
        return Err("body predicate must have one argument".to_string());
    }
    let idx = *pred_index
        .get(&atom.pred)
        .ok_or_else(|| format!("unknown predicate: {}", atom.pred))?;
    let var_name = match &atom.args[0] {
        SyGuSTerm::Identifier(id) => id.get_name(),
        _ => return Err("body atom argument must be a variable".to_string()),
    };
    let bv_arg = var_map
        .get(var_name)
        .ok_or_else(|| format!("unknown variable in body: {}", var_name))?;
    Ok(BitVec::extract(idx, idx, Term::var(bv_arg)))
}

fn body_atom_bit_term(
    atom: &HornAtom,
    pred_index: &HashMap<String, usize>,
    ctor_names: &HashSet<String>,
    var_map: &HashMap<String, String>,
) -> Result<SyGuSTerm, String> {
    if atom.args.len() != 1 {
        return Err("body predicate must have one argument".to_string());
    }
    let idx = *pred_index
        .get(&atom.pred)
        .ok_or_else(|| format!("unknown predicate: {}", atom.pred))?;
    let term = eval_term_to_bv(&atom.args[0], var_map, ctor_names)?;
    Ok(BitVec::extract(idx, idx, term))
}

fn eval_term_to_bv(
    term: &SyGuSTerm,
    var_map: &HashMap<String, String>,
    ctor_names: &HashSet<String>,
) -> Result<SyGuSTerm, String> {
    match term {
        SyGuSTerm::Identifier(id) => {
            let name = id.get_name();
            if let Some(mapped) = var_map.get(name) {
                Ok(Term::var(mapped))
            } else if ctor_names.contains(name) {
                Ok(Term::var(name))
            } else {
                Err(format!("unknown term symbol: {}", name))
            }
        }
        SyGuSTerm::Application(id, args) => {
            let name = id.get_name();
            if !ctor_names.contains(name) {
                return Err(format!("unknown constructor: {}", name));
            }
            let mut out_args = Vec::new();
            for arg in args {
                out_args.push(eval_term_to_bv(arg, var_map, ctor_names)?);
            }
            Ok(Term::app(name, out_args))
        }
        _ => Err("unsupported term in constraint".to_string()),
    }
}

fn or_bits(bits: Vec<SyGuSTerm>) -> SyGuSTerm {
    if bits.is_empty() {
        return BitVec::const_(0, 1);
    }
    let mut out = bits[0].clone();
    for bit in bits.iter().skip(1) {
        out = BitVec::bvor(bit.clone(), out);
    }
    out
}

fn concat_bits(bits: &[SyGuSTerm]) -> Result<SyGuSTerm, String> {
    if bits.is_empty() {
        return Err("no predicate bits to concatenate".to_string());
    }
    let mut out = bits[0].clone();
    for bit in bits.iter().skip(1) {
        out = BitVec::concat(bit.clone(), out);
    }
    Ok(out)
}

fn eq_one(bit: SyGuSTerm) -> SyGuSTerm {
    Term::eq(bit, BitVec::const_(1, 1))
}

#[derive(Clone, Debug)]
enum BitExpr {
    Const(bool),
    VarBit { var: String, idx: usize },
    Not(Box<BitExpr>),
    And(Box<BitExpr>, Box<BitExpr>),
    Or(Box<BitExpr>, Box<BitExpr>),
}

fn simplify_bv(term: &SyGuSTerm, width: usize) -> Option<SyGuSTerm> {
    let bits = build_bits(term, width)?;
    let bits: Vec<BitExpr> = bits.into_iter().map(simplify_bit).collect();
    rebuild_bv(&bits)
}

fn build_bits(term: &SyGuSTerm, width: usize) -> Option<Vec<BitExpr>> {
    match term {
        SyGuSTerm::Literal(Literal::BinConst(s)) => {
            let mut bits = Vec::new();
            for (idx, ch) in s.chars().rev().enumerate() {
                let bit = match ch {
                    '0' => BitExpr::Const(false),
                    '1' => BitExpr::Const(true),
                    _ => return None,
                };
                if idx >= width {
                    return None;
                }
                bits.push(bit);
            }
            if bits.len() != width {
                return None;
            }
            Some(bits)
        }
        SyGuSTerm::Identifier(id) => {
            let name = id.get_name().to_string();
            let bits = (0..width)
                .map(|idx| BitExpr::VarBit {
                    var: name.clone(),
                    idx,
                })
                .collect();
            Some(bits)
        }
        SyGuSTerm::Application(id, args) => match id {
            Identifier::Symbol(sym) if sym == "concat" && args.len() == 2 => {
                let left_len = bit_len(&args[0], Some(width))?;
                let right_len = bit_len(&args[1], Some(width))?;
                if left_len + right_len != width {
                    return None;
                }
                let left_bits = build_bits(&args[0], left_len)?;
                let right_bits = build_bits(&args[1], right_len)?;
                let mut out = Vec::with_capacity(width);
                out.extend(right_bits);
                out.extend(left_bits);
                Some(out)
            }
            Identifier::Symbol(sym) if sym == "bvand" && args.len() == 2 => {
                let len = bit_len(&args[0], Some(width))?;
                if len != bit_len(&args[1], Some(width))? || len != width {
                    return None;
                }
                let a = build_bits(&args[0], len)?;
                let b = build_bits(&args[1], len)?;
                Some(
                    a.into_iter()
                        .zip(b)
                        .map(|(x, y)| BitExpr::And(Box::new(x), Box::new(y)))
                        .collect(),
                )
            }
            Identifier::Symbol(sym) if sym == "bvor" && args.len() == 2 => {
                let len = bit_len(&args[0], Some(width))?;
                if len != bit_len(&args[1], Some(width))? || len != width {
                    return None;
                }
                let a = build_bits(&args[0], len)?;
                let b = build_bits(&args[1], len)?;
                Some(
                    a.into_iter()
                        .zip(b)
                        .map(|(x, y)| BitExpr::Or(Box::new(x), Box::new(y)))
                        .collect(),
                )
            }
            Identifier::Symbol(sym) if sym == "bvnot" && args.len() == 1 => {
                let len = bit_len(&args[0], Some(width))?;
                if len != width {
                    return None;
                }
                let a = build_bits(&args[0], len)?;
                Some(a.into_iter().map(|x| BitExpr::Not(Box::new(x))).collect())
            }
            Identifier::Indexed(sym, idxs) if sym == "_ extract" && args.len() == 1 => {
                let (high, low) = extract_indices(idxs)?;
                if high != low || width != 1 {
                    return None;
                }
                if let SyGuSTerm::Identifier(id) = &args[0] {
                    return Some(vec![BitExpr::VarBit {
                        var: id.get_name().to_string(),
                        idx: high,
                    }]);
                }
                let arg_len = bit_len(&args[0], None)?;
                let arg_bits = build_bits(&args[0], arg_len)?;
                arg_bits.get(high).cloned().map(|b| vec![b])
            }
            _ => None,
        },
        _ => None,
    }
}

fn bit_len(term: &SyGuSTerm, default_width: Option<usize>) -> Option<usize> {
    match term {
        SyGuSTerm::Literal(Literal::BinConst(s)) => Some(s.len()),
        SyGuSTerm::Identifier(_) => default_width,
        SyGuSTerm::Application(id, args) => match id {
            Identifier::Symbol(sym) if sym == "concat" && args.len() == 2 => {
                Some(bit_len(&args[0], default_width)? + bit_len(&args[1], default_width)?)
            }
            Identifier::Symbol(sym) if (sym == "bvand" || sym == "bvor") && args.len() == 2 => {
                let a = bit_len(&args[0], default_width)?;
                let b = bit_len(&args[1], default_width)?;
                if a == b {
                    Some(a)
                } else {
                    None
                }
            }
            Identifier::Symbol(sym) if sym == "bvnot" && args.len() == 1 => {
                bit_len(&args[0], default_width)
            }
            Identifier::Indexed(sym, idxs) if sym == "_ extract" && args.len() == 1 => {
                let (high, low) = extract_indices(idxs)?;
                if high == low {
                    Some(1)
                } else {
                    Some(high - low + 1)
                }
            }
            _ => None,
        },
        _ => None,
    }
}

fn extract_indices(idxs: &[Index]) -> Option<(usize, usize)> {
    match idxs {
        [Index::Numeral(high), Index::Numeral(low)] => Some((*high, *low)),
        _ => None,
    }
}

fn simplify_bit(bit: BitExpr) -> BitExpr {
    match bit {
        BitExpr::Not(inner) => match *inner {
            BitExpr::Const(v) => BitExpr::Const(!v),
            BitExpr::Not(x) => *x,
            x => BitExpr::Not(Box::new(x)),
        },
        BitExpr::And(a, b) => {
            let a = simplify_bit(*a);
            let b = simplify_bit(*b);
            match (&a, &b) {
                (BitExpr::Const(false), _) | (_, BitExpr::Const(false)) => BitExpr::Const(false),
                (BitExpr::Const(true), _) => b,
                (_, BitExpr::Const(true)) => a,
                _ if same_bitexpr(&a, &b) => a,
                _ => BitExpr::And(Box::new(a), Box::new(b)),
            }
        }
        BitExpr::Or(a, b) => {
            let a = simplify_bit(*a);
            let b = simplify_bit(*b);
            match (&a, &b) {
                (BitExpr::Const(true), _) | (_, BitExpr::Const(true)) => BitExpr::Const(true),
                (BitExpr::Const(false), _) => b,
                (_, BitExpr::Const(false)) => a,
                _ if same_bitexpr(&a, &b) => a,
                _ => BitExpr::Or(Box::new(a), Box::new(b)),
            }
        }
        other => other,
    }
}

fn same_bitexpr(a: &BitExpr, b: &BitExpr) -> bool {
    match (a, b) {
        (BitExpr::Const(x), BitExpr::Const(y)) => x == y,
        (BitExpr::VarBit { var: av, idx: ai }, BitExpr::VarBit { var: bv, idx: bi }) => {
            av == bv && ai == bi
        }
        _ => false,
    }
}

fn rebuild_bv(bits: &[BitExpr]) -> Option<SyGuSTerm> {
    if bits.is_empty() {
        return None;
    }
    if let Some(lit) = rebuild_const(bits) {
        return Some(lit);
    }
    if let Some(term) = rebuild_var(bits) {
        return Some(term);
    }
    if let Some(term) = rebuild_unary("bvnot", bits) {
        return Some(term);
    }
    if let Some(term) = rebuild_binary("bvand", bits) {
        return Some(term);
    }
    if let Some(term) = rebuild_binary("bvor", bits) {
        return Some(term);
    }
    let mut out = bit_to_term(&bits[0]);
    for bit in bits.iter().skip(1) {
        out = BitVec::concat(bit_to_term(bit), out);
    }
    Some(out)
}

fn rebuild_const(bits: &[BitExpr]) -> Option<SyGuSTerm> {
    let mut out = String::new();
    for bit in bits.iter().rev() {
        match bit {
            BitExpr::Const(true) => out.push('1'),
            BitExpr::Const(false) => out.push('0'),
            _ => return None,
        }
    }
    Some(Term::lit(Literal::BinConst(out)))
}

fn rebuild_var(bits: &[BitExpr]) -> Option<SyGuSTerm> {
    let mut var = None;
    for (idx, bit) in bits.iter().enumerate() {
        match bit {
            BitExpr::VarBit { var: v, idx: i } if *i == idx => {
                if let Some(ref prev) = var {
                    if prev != v {
                        return None;
                    }
                } else {
                    var = Some(v.clone());
                }
            }
            _ => return None,
        }
    }
    var.map(|v| Term::var(&v))
}

fn rebuild_unary(op: &str, bits: &[BitExpr]) -> Option<SyGuSTerm> {
    let mut var = None;
    for (idx, bit) in bits.iter().enumerate() {
        match bit {
            BitExpr::Not(inner) => match &**inner {
                BitExpr::VarBit { var: v, idx: i } if *i == idx => {
                    if let Some(ref prev) = var {
                        if prev != v {
                            return None;
                        }
                    } else {
                        var = Some(v.clone());
                    }
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    var.map(|v| Term::app(op, vec![Term::var(&v)]))
}

fn rebuild_binary(op: &str, bits: &[BitExpr]) -> Option<SyGuSTerm> {
    let mut left = None;
    let mut right = None;
    for (idx, bit) in bits.iter().enumerate() {
        match bit {
            BitExpr::And(a, b) if op == "bvand" => {
                if !collect_binary_vars(&mut left, &mut right, idx, a, b) {
                    return None;
                }
            }
            BitExpr::Or(a, b) if op == "bvor" => {
                if !collect_binary_vars(&mut left, &mut right, idx, a, b) {
                    return None;
                }
            }
            _ => return None,
        }
    }
    match (left, right) {
        (Some(l), Some(r)) => Some(Term::app(op, vec![Term::var(&l), Term::var(&r)])),
        _ => None,
    }
}

fn collect_binary_vars(
    left: &mut Option<String>,
    right: &mut Option<String>,
    idx: usize,
    a: &BitExpr,
    b: &BitExpr,
) -> bool {
    match (a, b) {
        (BitExpr::VarBit { var: l, idx: li }, BitExpr::VarBit { var: r, idx: ri })
            if *li == idx && *ri == idx =>
        {
            if let Some(ref prev) = left {
                if prev != l {
                    return false;
                }
            } else {
                *left = Some(l.clone());
            }
            if let Some(ref prev) = right {
                if prev != r {
                    return false;
                }
            } else {
                *right = Some(r.clone());
            }
            true
        }
        _ => false,
    }
}

fn bit_to_term(bit: &BitExpr) -> SyGuSTerm {
    match bit {
        BitExpr::Const(true) => BitVec::const_(1, 1),
        BitExpr::Const(false) => BitVec::const_(0, 1),
        BitExpr::VarBit { var, idx } => BitVec::extract(*idx, *idx, Term::var(var)),
        BitExpr::Not(inner) => BitVec::bvnot(bit_to_term(inner)),
        BitExpr::And(a, b) => BitVec::bvand(bit_to_term(a), bit_to_term(b)),
        BitExpr::Or(a, b) => BitVec::bvor(bit_to_term(a), bit_to_term(b)),
    }
}
