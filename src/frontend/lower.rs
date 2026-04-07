use crate::chc::ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm};
use crate::chc::pred::{dualize_dnf, emit_dnf_rules, Pred, PredAtom, PredDnf};
use crate::chc::saturate::saturate;
use crate::parser::{DelisCmd, DelisProg};
use crate::theory::utils::{collect_loaded_models, get_universe_domain_and_map};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;
use sygus_parser::ast::{GroupedRuleList, SyGuSGTerm, SyGuSTerm};
use sygus_parser::{DTConsDecl, GrammarDef, Identifier, Sort, SortDecl, SortedVar, SyGuSCmd};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AspectKey {
    pub relation: String,
    pub model: String,
    pub fields: Vec<(String, String)>,
}

impl AspectKey {
    fn display_fields(&self) -> String {
        self.fields
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Debug, Clone)]
pub struct BitMapping {
    pub aspect: AspectKey,
    pub bit: usize,
}

#[derive(Debug, Clone)]
pub struct LoweringSummary {
    pub finite_spaces: Vec<String>,
    pub semantic_relations: Vec<String>,
    pub bit_mappings: Vec<BitMapping>,
    pub constraint_bits: Vec<ConstraintBit>,
    pub needs_saturation: bool,
}

#[derive(Debug, Clone)]
pub struct ConstraintBit {
    pub relation: String,
    pub model: String,
    pub expected: bool,
    pub bit: usize,
}

#[derive(Debug, Clone)]
struct FieldDecl {
    name: String,
    sort: String,
}

#[derive(Debug, Clone)]
struct FiniteSpace {
    name: String,
    fields: Vec<FieldDecl>,
    where_expr: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AspectValue {
    Model(String),
    Int(usize),
    Symbol(String),
}

#[derive(Debug, Clone)]
struct SemanticConstraint {
    relation: String,
    model: String,
    fields: Vec<(String, String)>,
    expected: bool,
}

impl AspectValue {
    fn display(&self) -> String {
        match self {
            AspectValue::Model(value) | AspectValue::Symbol(value) => value.clone(),
            AspectValue::Int(value) => value.to_string(),
        }
    }
}

impl LoweringSummary {
    pub fn to_comment_text(&self, target: &str) -> String {
        let mut lines = Vec::new();
        lines.push(format!("; generic frontend {target} lowering summary"));
        lines.push(format!(
            "; finite-spaces: {}",
            self.finite_spaces.join(", ")
        ));
        lines.push(format!(
            "; semantic-relations: {}",
            self.semantic_relations.join(", ")
        ));
        lines.push(format!("; needs-saturation: {}", self.needs_saturation));
        lines.push(format!("; bv-len: {}", self.bit_mappings.len()));
        for mapping in &self.bit_mappings {
            lines.push(format!(
                "; bit {} <-> {}({}; {})",
                mapping.bit,
                mapping.aspect.relation,
                mapping.aspect.model,
                mapping.aspect.display_fields()
            ));
        }
        for constraint in &self.constraint_bits {
            lines.push(format!(
                "; constraint {}{} -> bit {} == {}",
                if constraint.expected { "+" } else { "-" },
                format!("{}:{}", constraint.relation, constraint.model),
                constraint.bit,
                if constraint.expected { 1 } else { 0 }
            ));
        }
        if self.bit_mappings.is_empty() {
            lines.push("; no semantic aspects were enumerated; add semantic constraints to create BV state bits".to_string());
        }
        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
struct SemCase {
    self_var: String,
    vars: Vec<String>,
    body: Value,
}

pub fn to_sygus(problem: &DelisProg) -> Result<Vec<SyGuSCmd>, String> {
    let summary = lower(problem)?;
    if summary.bit_mappings.is_empty() {
        return Err("generic SyGuS emission requires at least one semantic aspect bit".to_string());
    }

    let bv_sort = Sort::bitvec(summary.bit_mappings.len());
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::DeclareTermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("generic SyGuS emission requires declare-term-types".to_string());
    };
    let grammar = declare_term_types_to_bv_grammar(declare_term_types, &bv_sort)?;
    let mut program = to_chc_ast(problem)?;
    if summary.needs_saturation {
        program.rules = saturate(&program.rules);
    }
    crate::chc::bv::compile_one_way(&program, &synth_name(problem)?, grammar)
        .into_iter()
        .map(Ok)
        .collect()
}

pub fn to_chc_ast(problem: &DelisProg) -> Result<HornProg, String> {
    let summary = lower(problem)?;
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::DeclareTermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("generic CHC emission requires declare-term-types".to_string());
    };
    let cases = parse_semantic_cases(problem)?;
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    let mut program = HornProg::default();
    program.decls.push(HornDecl::SetLogic("HORN".to_string()));
    program.decls.push(HornDecl::DeclareDatatypes(
        vec![SortDecl {
            symbol: "Term".to_string(),
            arity: 0,
        }],
        vec![sig.datatype_constructors()],
    ));
    program
        .decls
        .push(HornDecl::DeclareVar("r".to_string(), term_sort()));
    for var in sig.all_match_vars(&cases) {
        program.decls.push(HornDecl::DeclareVar(var, term_sort()));
    }
    for mapping in &summary.bit_mappings {
        let pred = state_name(&mapping.aspect);
        rel_decl(&mut program, &pred);
        rel_decl(&mut program, format!("${pred}"));
    }
    rel_decl(&mut program, "Goal");

    for cons in &sig.constructors {
        let Some(case) = cases.get(&cons.name) else {
            return Err(format!(
                "missing define-semantics match case for constructor {}",
                cons.name
            ));
        };
        if case.vars.len() != cons.arg_sorts.len() {
            return Err(format!(
                "match case for constructor {} binds {} variables, expected {}",
                cons.name,
                case.vars.len(),
                cons.arg_sorts.len()
            ));
        }
        let head_arg = HornTerm::app(
            &cons.name,
            case.vars.iter().map(HornTerm::var).collect::<Vec<_>>(),
        );
        let child_vars = case
            .vars
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        for mapping in &summary.bit_mappings {
            let model_len = model_len(problem, &mapping.aspect.model)?;
            let scalar_env = aspect_scalar_env(&mapping.aspect, model_len);
            let dnf = eval_semantic_dnf(
                problem,
                &case.body,
                &mapping.aspect,
                &scalar_env,
                &summary.semantic_relations,
                &child_vars,
                &sig,
                cons,
                &head_arg,
                &case.self_var,
            )?;
            let head = PredAtom::new(Pred::new(state_name(&mapping.aspect)), head_arg.clone());
            emit_dnf_rules(&mut program.rules, head.clone(), dnf.clone());
            if let Some((dual_head, dual_dnf)) = dualize_dnf(head, &dnf) {
                emit_dnf_rules(&mut program.rules, dual_head, dual_dnf);
            }
        }
    }

    let goal_body = summary
        .constraint_bits
        .iter()
        .map(|constraint| {
            let aspect = summary
                .bit_mappings
                .iter()
                .find(|mapping| mapping.bit == constraint.bit)
                .map(|mapping| &mapping.aspect)
                .ok_or_else(|| format!("constraint references unknown bit {}", constraint.bit))?;
            let base = state_name(aspect);
            Ok(HornAtom {
                pred: if constraint.expected {
                    base
                } else {
                    format!("${base}")
                },
                arg: HornTerm::var("r"),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    program.rules.push(HornRule {
        body_atoms: goal_body,
        head: HornAtom {
            pred: "Goal".to_string(),
            arg: HornTerm::var("r"),
        },
    });
    program.queries.push(HornQuery {
        body_atoms: vec![HornAtom {
            pred: "Goal".to_string(),
            arg: HornTerm::var("r"),
        }],
    });
    Ok(program)
}

fn synth_name(problem: &DelisProg) -> Result<String, String> {
    problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::As(SyGuSCmd::SynthFun(name, _, _, _)) => Some(name.clone()),
            _ => None,
        })
        .ok_or_else(|| "generic SyGuS emission requires synth-fun".to_string())
}

fn term_sort() -> Sort {
    Sort::Simple(Identifier::Symbol("Term".to_string()))
}

fn rel_decl(program: &mut HornProg, name: impl Into<String>) {
    program
        .decls
        .push(HornDecl::DeclareRel(name.into(), vec![term_sort()]));
}

fn sanitize_symbol(symbol: &str) -> String {
    symbol
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn state_name(aspect: &AspectKey) -> String {
    let mut parts = vec![
        sanitize_symbol(&aspect.relation),
        sanitize_symbol(&aspect.model),
    ];
    for (field, value) in &aspect.fields {
        parts.push(sanitize_symbol(field));
        parts.push(sanitize_symbol(value));
    }
    parts.join("_")
}

#[derive(Debug, Clone)]
struct TermConstructor {
    name: String,
    result_sort: String,
    arg_sorts: Vec<String>,
}

#[derive(Debug, Clone)]
struct TermSignature {
    constructors: Vec<TermConstructor>,
}

impl TermSignature {
    fn from_declare_term_types(
        decl: &crate::parser::ast::DeclareTermTypesCmd,
    ) -> Result<Self, String> {
        let mut constructors = Vec::new();
        for (sort_decl, term_type_decl) in decl.sort_decls.iter().zip(&decl.term_type_decls) {
            for cons in &term_type_decl.constructors {
                constructors.push(TermConstructor {
                    name: cons.symbol.to_string(),
                    result_sort: sort_decl.symbol.clone(),
                    arg_sorts: cons.sorts.iter().map(|sort| sort.to_string()).collect(),
                });
            }
        }
        Ok(Self { constructors })
    }

    fn datatype_constructors(&self) -> Vec<DTConsDecl> {
        self.constructors
            .iter()
            .map(|cons| DTConsDecl {
                name: cons.name.clone(),
                args: cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(idx, _sort)| SortedVar {
                        name: format!("arg{idx}"),
                        sort: term_sort(),
                    })
                    .collect(),
            })
            .collect()
    }

    fn all_match_vars(&self, cases: &HashMap<String, SemCase>) -> Vec<String> {
        let mut vars = std::collections::BTreeSet::new();
        for case in cases.values() {
            vars.extend(case.vars.iter().cloned());
        }
        for cons in &self.constructors {
            for idx in 0..cons.arg_sorts.len() {
                vars.insert(format!("__up{idx}"));
            }
        }
        vars.into_iter().collect()
    }

    fn parent_contexts(&self, child_sort: &str, child: HornTerm) -> Vec<HornTerm> {
        let mut out = Vec::new();
        for cons in &self.constructors {
            for (idx, arg_sort) in cons.arg_sorts.iter().enumerate() {
                if arg_sort != child_sort {
                    continue;
                }
                let args = cons
                    .arg_sorts
                    .iter()
                    .enumerate()
                    .map(|(arg_idx, _)| {
                        if arg_idx == idx {
                            child.clone()
                        } else {
                            HornTerm::var(format!("__up{arg_idx}"))
                        }
                    })
                    .collect();
                out.push(HornTerm::app(&cons.name, args));
            }
        }
        out
    }
}

fn model_len(problem: &DelisProg, model_name: &str) -> Result<usize, String> {
    let model = problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) if model.name == model_name => Some(model),
            _ => None,
        })
        .ok_or_else(|| format!("unknown model {model_name}"))?;
    let (domain, _id2sym, _sort) = get_universe_domain_and_map(model);
    Ok(domain.len())
}

fn aspect_scalar_env(aspect: &AspectKey, model_len: usize) -> Vec<(String, AspectValue)> {
    let mut env = vec![(
        "model".to_string(),
        AspectValue::Model(aspect.model.clone()),
    )];
    for (name, value) in &aspect.fields {
        let value = value
            .parse::<usize>()
            .map(AspectValue::Int)
            .unwrap_or_else(|_| AspectValue::Symbol(value.clone()));
        env.push((name.clone(), value));
    }
    env.push(("__len".to_string(), AspectValue::Int(model_len)));
    env
}

fn id_gterm(name: &str) -> Result<SyGuSGTerm, String> {
    Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Identifier(
        Identifier::from_str(name)
            .map_err(|e| format!("invalid grammar identifier {name}: {e}"))?,
    )))
}

fn app_gterm(name: &str, args: Vec<String>) -> Result<SyGuSGTerm, String> {
    Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(
        Identifier::from_str(name)
            .map_err(|e| format!("invalid grammar identifier {name}: {e}"))?,
        args.into_iter()
            .map(|arg| {
                Identifier::from_str(&arg)
                    .map(SyGuSTerm::Identifier)
                    .map_err(|e| format!("invalid grammar argument {arg}: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    )))
}

fn grammar_symbol_for_sort(sort_name: &str, ret_sort_name: &str) -> String {
    if sort_name == ret_sort_name {
        "Start".to_string()
    } else {
        sort_name.to_string()
    }
}

fn declare_term_types_to_bv_grammar(
    decl: &crate::parser::ast::DeclareTermTypesCmd,
    bv_sort: &Sort,
) -> Result<GrammarDef, String> {
    let ret_sort_name = decl
        .sort_decls
        .first()
        .map(|decl| decl.symbol.clone())
        .ok_or_else(|| "declare-term-types is empty".to_string())?;
    let sorted_vars = decl
        .sort_decls
        .iter()
        .map(|sort_decl| SortedVar {
            name: grammar_symbol_for_sort(&sort_decl.symbol, &ret_sort_name),
            sort: bv_sort.clone(),
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
                .map(|cons| {
                    if cons.sorts.is_empty() {
                        id_gterm(&cons.symbol.to_string())
                    } else {
                        let args = cons
                            .sorts
                            .iter()
                            .map(|sort| grammar_symbol_for_sort(&sort.to_string(), &ret_sort_name))
                            .collect::<Vec<_>>();
                        app_gterm(&cons.symbol.to_string(), args)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(GroupedRuleList {
                symbol,
                sort: bv_sort.clone(),
                terms,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(GrammarDef {
        sorted_vars,
        grouped_rule_lists,
    })
}

fn parse_semantic_cases(problem: &DelisProg) -> Result<HashMap<String, SemCase>, String> {
    let semantics = problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::DefineSemantics(cmd) => Some(cmd),
            _ => None,
        })
        .ok_or_else(|| "generic SyGuS emission requires define-semantics".to_string())?;
    let body_src = semantics
        .body
        .get(1)
        .ok_or_else(|| "define-semantics expects a match body".to_string())?;
    let body = lexpr::from_str(body_src)
        .map_err(|e| format!("failed to parse define-semantics body: {e}"))?;
    let outer = body
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must be a list: {body}"))?;
    let match_expr = match outer.as_slice() {
        [single] => single,
        _ => &body,
    };
    let items = match_expr
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics body must contain a match expression: {body}"))?;
    if items.first().and_then(|v| v.as_symbol()) != Some("match") {
        return Err(format!(
            "define-semantics body must start with match: {body}"
        ));
    }
    let self_var = items
        .get(1)
        .and_then(|v| v.as_symbol())
        .ok_or_else(|| format!("match must bind a scrutinee variable: {body}"))?
        .to_string();
    let mut cases = HashMap::new();
    for case in items.iter().skip(2) {
        let case_items = case
            .to_ref_vec()
            .ok_or_else(|| format!("match case must be a list: {case}"))?;
        if case_items.len() != 2 {
            return Err(format!("match case must have pattern and body: {case}"));
        }
        let pattern = case_items[0]
            .to_ref_vec()
            .ok_or_else(|| format!("match pattern must be a list: {}", case_items[0]))?;
        let cons = pattern
            .first()
            .and_then(|v| v.as_symbol())
            .ok_or_else(|| {
                format!(
                    "match pattern constructor must be a symbol: {}",
                    case_items[0]
                )
            })?
            .to_string();
        let vars = pattern
            .iter()
            .skip(1)
            .map(|v| {
                v.as_symbol()
                    .map(str::to_string)
                    .ok_or_else(|| format!("match pattern variable must be a symbol: {v}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        cases.insert(
            cons.clone(),
            SemCase {
                self_var: self_var.clone(),
                vars,
                body: case_items[1].clone(),
            },
        );
    }
    Ok(cases)
}

fn dnf_true() -> PredDnf {
    vec![vec![]]
}

fn dnf_false() -> PredDnf {
    Vec::new()
}

fn dnf_and(lhs: PredDnf, rhs: PredDnf) -> PredDnf {
    let mut out = Vec::new();
    for left in &lhs {
        for right in &rhs {
            let mut conj = left.clone();
            conj.extend(right.clone());
            out.push(conj);
        }
    }
    out
}

fn dnf_or(mut lhs: PredDnf, rhs: PredDnf) -> PredDnf {
    lhs.extend(rhs);
    lhs
}

fn negate_dnf(dnf: &PredDnf) -> PredDnf {
    if dnf.is_empty() {
        return dnf_true();
    }
    let mut clauses: Vec<Vec<PredAtom>> = Vec::new();
    for conj in dnf {
        if conj.is_empty() {
            return dnf_false();
        }
        clauses.push(conj.iter().map(PredAtom::dual).collect());
    }
    let mut out = dnf_true();
    for clause in clauses {
        let mut next = Vec::new();
        for base in &out {
            for atom in &clause {
                let mut conj = base.clone();
                conj.push(atom.clone());
                next.push(conj);
            }
        }
        out = next;
    }
    out
}

fn eval_semantic_dnf(
    problem: &DelisProg,
    expr: &Value,
    current_aspect: &AspectKey,
    scalar_env: &[(String, AspectValue)],
    semantic_relations: &[String],
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<PredDnf, String> {
    if let Some(symbol) = expr.as_symbol() {
        return match symbol {
            "true" => Ok(dnf_true()),
            "false" => Ok(dnf_false()),
            _ => Err(format!("unsupported semantic symbol {symbol}")),
        };
    }
    let items = expr
        .to_ref_vec()
        .ok_or_else(|| format!("semantic expression must be a list: {expr}"))?;
    let op = items
        .first()
        .and_then(|v| v.as_symbol())
        .ok_or_else(|| format!("semantic expression must start with a symbol: {expr}"))?;
    match op {
        "and" => items[1..].iter().try_fold(dnf_true(), |acc, arg| {
            Ok(dnf_and(
                acc,
                eval_semantic_dnf(
                    problem,
                    arg,
                    current_aspect,
                    scalar_env,
                    semantic_relations,
                    child_vars,
                    sig,
                    current_constructor,
                    current_term,
                    self_var,
                )?,
            ))
        }),
        "or" => items[1..].iter().try_fold(dnf_false(), |acc, arg| {
            Ok(dnf_or(
                acc,
                eval_semantic_dnf(
                    problem,
                    arg,
                    current_aspect,
                    scalar_env,
                    semantic_relations,
                    child_vars,
                    sig,
                    current_constructor,
                    current_term,
                    self_var,
                )?,
            ))
        }),
        "not" | "dual" if items.len() == 2 => Ok(negate_dnf(&eval_semantic_dnf(
            problem,
            items[1],
            current_aspect,
            scalar_env,
            semantic_relations,
            child_vars,
            sig,
            current_constructor,
            current_term,
            self_var,
        )?)),
        "exists" if items.len() == 3 => {
            let mut out = dnf_false();
            for extra_env in enumerate_exists_bindings(problem, items[1], scalar_env)? {
                let mut next_env = scalar_env.to_vec();
                next_env.extend(extra_env);
                out = dnf_or(
                    out,
                    eval_semantic_dnf(
                        problem,
                        items[2],
                        current_aspect,
                        &next_env,
                        semantic_relations,
                        child_vars,
                        sig,
                        current_constructor,
                        current_term,
                        self_var,
                    )?,
                );
            }
            Ok(out)
        }
        "forall" if items.len() == 3 => {
            let mut out = dnf_true();
            for extra_env in enumerate_exists_bindings(problem, items[1], scalar_env)? {
                let mut next_env = scalar_env.to_vec();
                next_env.extend(extra_env);
                out = dnf_and(
                    out,
                    eval_semantic_dnf(
                        problem,
                        items[2],
                        current_aspect,
                        &next_env,
                        semantic_relations,
                        child_vars,
                        sig,
                        current_constructor,
                        current_term,
                        self_var,
                    )?,
                );
            }
            Ok(out)
        }
        "=" | "<" | "<=" => Ok(
            if eval_semantic_bool(problem, &current_aspect.model, expr, scalar_env)? {
                dnf_true()
            } else {
                dnf_false()
            },
        ),
        _ if semantic_relations.iter().any(|rel| rel == op) => {
            if items.len() < 3 {
                return Err(format!(
                    "semantic call {expr} expects term and aspect arguments"
                ));
            }
            let aspect = semantic_call_aspect(items[2], op, current_aspect, scalar_env)?;
            let pred = Pred::new(state_name(&aspect));
            let terms = semantic_call_terms(
                items[1],
                child_vars,
                sig,
                current_constructor,
                current_term,
                self_var,
            )?;
            Ok(terms
                .into_iter()
                .map(|term| vec![PredAtom::new(pred.clone(), term)])
                .collect())
        }
        _ => {
            let args = items[1..]
                .iter()
                .map(|arg| value_expr(arg, scalar_env, current_model_len(scalar_env)?))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(
                if eval_model_bool_values(problem, &current_aspect.model, op, &args)? {
                    dnf_true()
                } else {
                    dnf_false()
                },
            )
        }
    }
}

fn semantic_call_terms(
    target: &Value,
    child_vars: &std::collections::BTreeSet<String>,
    sig: &TermSignature,
    current_constructor: &TermConstructor,
    current_term: &HornTerm,
    self_var: &str,
) -> Result<Vec<HornTerm>, String> {
    if let Some(symbol) = target.as_symbol() {
        if child_vars.contains(symbol) {
            return Ok(vec![HornTerm::var(symbol)]);
        }
        if symbol == self_var {
            return Ok(vec![current_term.clone()]);
        }
        return Err(format!(
            "semantic call target {symbol} is not a pattern child; use an explicit navigation form"
        ));
    }
    let items = target.to_ref_vec().ok_or_else(|| {
        format!("semantic call target must be a child or navigation form: {target}")
    })?;
    match items.as_slice() {
        [op, inner] if op.as_symbol() == Some("up") => {
            if inner.as_symbol().is_none() {
                return Err(format!(
                    "up target must name the current term variable: {target}"
                ));
            }
            let contexts =
                sig.parent_contexts(&current_constructor.result_sort, current_term.clone());
            if contexts.is_empty() {
                return Err(format!(
                    "no parent constructor context can contain sort {}",
                    current_constructor.result_sort
                ));
            }
            Ok(contexts)
        }
        _ => Err(format!(
            "unsupported semantic call navigation target {target}"
        )),
    }
}

fn current_model_len(env: &[(String, AspectValue)]) -> Result<usize, String> {
    match lookup(env, "__len") {
        Some(AspectValue::Int(value)) => Ok(*value),
        _ => Err("internal error: missing __len in semantic environment".to_string()),
    }
}

fn model_by_name<'a>(
    problem: &'a DelisProg,
    model_name: &str,
) -> Result<&'a crate::lang::model::Model, String> {
    problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) if model.name == model_name => Some(model),
            _ => None,
        })
        .ok_or_else(|| format!("unknown model {model_name}"))
}

fn aspect_value_to_index(value: &AspectValue, id2sym: &[String]) -> Result<usize, String> {
    match value {
        AspectValue::Int(idx) => Ok(*idx),
        AspectValue::Symbol(sym) => id2sym
            .iter()
            .position(|candidate| candidate == sym)
            .ok_or_else(|| format!("unknown universe element {sym}")),
        AspectValue::Model(model) => Err(format!("expected universe element, got model {model}")),
    }
}

fn aspect_projection(
    expr: &Value,
    env: &[(String, AspectValue)],
) -> Option<Result<AspectValue, String>> {
    let items = expr.to_ref_vec()?;
    let [field, aspect] = items.as_slice() else {
        return None;
    };
    if aspect.as_symbol() != Some("asp") {
        return None;
    }
    let field = match field.as_symbol() {
        Some(field) => field,
        None => {
            return Some(Err(format!(
                "aspect projection field must be a symbol: {expr}"
            )));
        }
    };
    Some(Ok(lookup(env, field)
        .cloned()
        .unwrap_or_else(|| AspectValue::Symbol(field.to_string()))))
}

fn model_name_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<String, String> {
    match value_expr(expr, env, model_len)? {
        AspectValue::Model(model) => Ok(model),
        AspectValue::Symbol(symbol)
            if env.iter().any(|(_, value)| {
                matches!(value, AspectValue::Model(model_name) if model_name == &symbol)
            }) =>
        {
            Ok(symbol)
        }
        other => Err(format!("expected Model expression, got {other:?}")),
    }
}

fn value_expr(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<AspectValue, String> {
    if let Some(value) = expr.as_i64() {
        return if value >= 0 {
            Ok(AspectValue::Int(value as usize))
        } else {
            Err(format!(
                "negative integer value {value} is not a finite aspect value"
            ))
        };
    }
    if let Some(symbol) = expr.as_symbol() {
        return Ok(lookup(env, symbol)
            .cloned()
            .unwrap_or_else(|| AspectValue::Symbol(symbol.to_string())));
    }
    if let Some(value) = aspect_projection(expr, env) {
        return value;
    }
    eval_int(expr, env, model_len).and_then(|value| {
        if value >= 0 {
            Ok(AspectValue::Int(value as usize))
        } else {
            Err(format!(
                "negative integer value {value} is not a finite aspect value"
            ))
        }
    })
}

fn eval_model_bool_values(
    problem: &DelisProg,
    model_name: &str,
    prop: &str,
    args: &[AspectValue],
) -> Result<bool, String> {
    let model = model_by_name(problem, model_name)?;
    let (_domain, id2sym, _sort) = get_universe_domain_and_map(model);
    let resolved_args = args
        .iter()
        .map(|arg| resolve_model_value(problem, model_name, arg))
        .collect::<Result<Vec<_>, _>>()?;
    let indices = resolved_args
        .iter()
        .map(|arg| aspect_value_to_index(arg, &id2sym))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(matches!(
        crate::theory::utils::eval(prop, &indices, &model.func_interps, &id2sym),
        Some(crate::lang::model::InterpedValue::Literal(lit))
            if lit.to_string() == "true"
    ))
}

fn resolve_model_value(
    problem: &DelisProg,
    model_name: &str,
    value: &AspectValue,
) -> Result<AspectValue, String> {
    let AspectValue::Symbol(symbol) = value else {
        return Ok(value.clone());
    };
    let model = model_by_name(problem, model_name)?;
    let (_domain, id2sym, _sort) = get_universe_domain_and_map(model);
    if id2sym.iter().any(|elem| elem == symbol) {
        return Ok(value.clone());
    }
    let Some(value) = crate::theory::utils::eval(symbol, &[], &model.func_interps, &id2sym) else {
        return Ok(AspectValue::Symbol(symbol.clone()));
    };
    match value {
        crate::lang::model::InterpedValue::UnivElem(elem) => {
            Ok(AspectValue::Symbol(elem.name.clone()))
        }
        other => Err(format!(
            "expected zero-arity model symbol {symbol} to evaluate to a universe element, got {other}"
        )),
    }
}

fn enumerate_exists_bindings(
    problem: &DelisProg,
    bindings: &Value,
    scalar_env: &[(String, AspectValue)],
) -> Result<Vec<Vec<(String, AspectValue)>>, String> {
    let binding_items = bindings
        .to_ref_vec()
        .ok_or_else(|| format!("exists bindings must be a list: {bindings}"))?;
    let mut domains = Vec::new();
    for binding in binding_items {
        let item = binding
            .to_ref_vec()
            .ok_or_else(|| format!("exists binding must be a list: {binding}"))?;
        let [name, domain] = item.as_slice() else {
            return Err(format!(
                "exists binding must have shape (name domain): {binding}"
            ));
        };
        let name = name
            .as_symbol()
            .ok_or_else(|| format!("exists binding name must be a symbol: {name}"))?
            .to_string();
        let domain_items = domain
            .to_ref_vec()
            .ok_or_else(|| format!("exists domain must be a list: {domain}"))?;
        let model_len = current_model_len(scalar_env)?;
        match domain_items.first().and_then(|v| v.as_symbol()) {
            Some("int-range") if domain_items.len() == 3 => {
                let lo = eval_int(domain_items[1], scalar_env, model_len)?;
                let hi = eval_int(domain_items[2], scalar_env, model_len)?;
                if lo < 0 || hi < lo {
                    domains.push((name, Vec::new()));
                } else {
                    domains.push((
                        name,
                        (lo as usize..=hi as usize)
                            .map(AspectValue::Int)
                            .collect::<Vec<_>>(),
                    ));
                }
            }
            Some("domain") if domain_items.len() >= 2 => {
                let model_name = model_name_expr(domain_items[1], scalar_env, model_len)?;
                let model = model_by_name(problem, &model_name)?;
                let (domain_syms, _id2sym, _sort) = get_universe_domain_and_map(model);
                domains.push((
                    name,
                    domain_syms.into_iter().map(AspectValue::Symbol).collect(),
                ));
            }
            _ => {
                return Err(format!(
                    "supported quantified domains are (int-range lo hi) and (domain model-expr): {domain}"
                ));
            }
        }
    }

    let mut out = Vec::new();
    for values in domains
        .iter()
        .map(|(_, values)| values)
        .multi_cartesian_product()
    {
        out.push(
            domains
                .iter()
                .zip(values)
                .map(|((name, _), value)| (name.clone(), value.clone()))
                .collect(),
        );
    }
    Ok(out)
}

fn semantic_call_aspect(
    aspect_expr: &Value,
    relation: &str,
    current_aspect: &AspectKey,
    scalar_env: &[(String, AspectValue)],
) -> Result<AspectKey, String> {
    if aspect_expr.as_symbol() == Some("asp") {
        let mut aspect = current_aspect.clone();
        aspect.relation = relation.to_string();
        return Ok(aspect);
    }
    let items = aspect_expr.to_ref_vec().ok_or_else(|| {
        format!("semantic call aspect must be asp or an aspect application: {aspect_expr}")
    })?;
    if items.is_empty() {
        return Err(format!("empty aspect expression {aspect_expr}"));
    }
    let mut model = None;
    let mut fields = Vec::new();
    let model_len = current_model_len(scalar_env)?;
    let mut field_idx = 0usize;
    for arg in items.iter().skip(1) {
        let value = value_expr(arg, scalar_env, model_len)?;
        if let AspectValue::Model(model_name) = value {
            model = Some(model_name);
            continue;
        }
        let Some((field_name, _)) = current_aspect.fields.get(field_idx) else {
            return Err(format!(
                "too many scalar fields in aspect expression {aspect_expr}"
            ));
        };
        fields.push((field_name.clone(), value.display()));
        field_idx += 1;
    }
    Ok(AspectKey {
        relation: relation.to_string(),
        model: model.unwrap_or_else(|| current_aspect.model.clone()),
        fields,
    })
}

pub fn lower(problem: &DelisProg) -> Result<LoweringSummary, String> {
    let finite_cmds = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineFinite(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();
    let finite_spaces = finite_cmds
        .iter()
        .map(|cmd| parse_finite_space(cmd))
        .collect::<Result<Vec<_>, _>>()?;
    let semantics = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineSemantics(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();

    if finite_cmds.is_empty() {
        return Err("generic frontend lowering requires at least one define-finite".to_string());
    }
    if semantics.is_empty() {
        return Err("generic frontend lowering requires at least one define-semantics".to_string());
    }

    let semantic_relations = semantics
        .iter()
        .map(|cmd| semantic_relation_name(&cmd.body))
        .collect::<Result<Vec<_>, _>>()?;

    let models_by_name = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) => Some((model.name.clone(), model)),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let aspect_space = finite_spaces
        .first()
        .ok_or_else(|| "generic frontend lowering requires a finite aspect space".to_string())?;
    let semantic_constraints =
        collect_semantic_constraints(problem, &semantic_relations, aspect_space, &models_by_name)?;
    let mut model_names = Vec::new();
    for constraint in &semantic_constraints {
        if models_by_name.contains_key(&constraint.model)
            && !model_names.contains(&constraint.model)
        {
            model_names.push(constraint.model.clone());
        }
    }
    let loaded_models = collect_loaded_models(&model_names, &models_by_name);

    let mut bit_mappings = Vec::new();
    let mut bit_by_aspect = HashMap::new();
    for relation in &semantic_relations {
        for model in &loaded_models {
            for assignment in enumerate_aspects(aspect_space, model.name.as_str(), model)? {
                let bit = bit_mappings.len();
                let fields = assignment
                    .iter()
                    .filter(|(_, value)| !matches!(value, AspectValue::Model(_)))
                    .map(|(name, value)| (name.clone(), value.display()))
                    .collect::<Vec<_>>();
                let aspect = AspectKey {
                    relation: relation.clone(),
                    model: model.name.clone(),
                    fields,
                };
                bit_by_aspect.insert(aspect.clone(), bit);
                bit_mappings.push(BitMapping { bit, aspect });
            }
        }
    }
    let constraint_bits = semantic_constraints
        .iter()
        .map(|constraint| {
            let key = AspectKey {
                relation: constraint.relation.clone(),
                model: constraint.model.clone(),
                fields: constraint.fields.clone(),
            };
            let bit = bit_by_aspect.get(&key).copied().ok_or_else(|| {
                format!(
                    "constraint aspect does not belong to finite space: {}({}; {})",
                    key.relation,
                    key.model,
                    key.display_fields()
                )
            })?;
            Ok(ConstraintBit {
                relation: constraint.relation.clone(),
                model: constraint.model.clone(),
                expected: constraint.expected,
                bit,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let needs_saturation = semantics.iter().any(|cmd| {
        cmd.body.iter().any(|fragment| {
            fragment.contains("(up")
                || fragment.contains(".up")
                || fragment.contains("(parent")
                || fragment.contains("(child")
        })
    });

    Ok(LoweringSummary {
        finite_spaces: finite_spaces
            .iter()
            .map(|space| space.name.clone())
            .collect(),
        semantic_relations,
        bit_mappings,
        constraint_bits,
        needs_saturation,
    })
}

fn collect_semantic_constraints(
    problem: &DelisProg,
    semantic_relations: &[String],
    aspect_space: &FiniteSpace,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
) -> Result<Vec<SemanticConstraint>, String> {
    let mut constraints = Vec::new();
    for cmd in &problem.cmds {
        let DelisCmd::As(sygus_parser::SyGuSCmd::Constraint(term)) = cmd else {
            continue;
        };
        if let Some(constraint) =
            parse_semantic_constraint(term, semantic_relations, aspect_space, models_by_name)?
        {
            constraints.push(constraint);
        }
    }
    Ok(constraints)
}

fn parse_semantic_constraint(
    term: &SyGuSTerm,
    semantic_relations: &[String],
    aspect_space: &FiniteSpace,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
) -> Result<Option<SemanticConstraint>, String> {
    let (expected, term) = match term {
        SyGuSTerm::Application(sym, args) if sym.to_string() == "not" && args.len() == 1 => {
            (false, &args[0])
        }
        _ => (true, term),
    };
    let SyGuSTerm::Application(rel, args) = term else {
        return Ok(None);
    };
    let relation = rel.to_string();
    if !semantic_relations.contains(&relation) {
        return Ok(None);
    }
    let Some(aspect_term) = args.last() else {
        return Err(format!(
            "semantic constraint {relation} is missing an aspect argument"
        ));
    };
    let (model, fields) = parse_aspect_term(aspect_term, aspect_space, models_by_name)?;
    Ok(Some(SemanticConstraint {
        relation,
        model,
        fields,
        expected,
    }))
}

fn parse_aspect_term(
    term: &SyGuSTerm,
    aspect_space: &FiniteSpace,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
) -> Result<(String, Vec<(String, String)>), String> {
    let SyGuSTerm::Application(cons, args) = term else {
        return Err(format!(
            "semantic constraint aspect must be an application, got {term}"
        ));
    };
    if cons.to_string() != aspect_space.name {
        return Err(format!(
            "semantic constraint uses aspect constructor {}, expected {}",
            cons, aspect_space.name
        ));
    }
    if args.len() != aspect_space.fields.len() {
        return Err(format!(
            "aspect {} expects {} fields, got {}",
            aspect_space.name,
            aspect_space.fields.len(),
            args.len()
        ));
    }

    let mut model_name = None;
    for (field, arg) in aspect_space.fields.iter().zip(args.iter()) {
        if field.sort == "Model" {
            model_name = Some(arg.to_string());
            break;
        }
    }
    let model_name =
        model_name.ok_or_else(|| format!("aspect {} has no Model field", aspect_space.name))?;
    let model = models_by_name
        .get(&model_name)
        .ok_or_else(|| format!("constraint references unknown model {model_name}"))?;
    let (domain, _id2sym, _sort) = get_universe_domain_and_map(model);
    let model_len = domain.len();

    let mut assignment = Vec::new();
    for (field, arg) in aspect_space.fields.iter().zip(args.iter()) {
        let value = match field.sort.as_str() {
            "Model" => AspectValue::Model(arg.to_string()),
            "Int" => {
                let value = eval_sygus_int(arg, &assignment, model_len)?;
                if value < 0 {
                    return Err(format!(
                        "aspect field {} must be non-negative, got {value}",
                        field.name
                    ));
                }
                AspectValue::Int(value as usize)
            }
            other => AspectValue::Symbol(if arg.to_string() == other {
                other.to_string()
            } else {
                arg.to_string()
            }),
        };
        assignment.push((field.name.clone(), value));
    }
    if let Some(where_expr) = &aspect_space.where_expr {
        if !eval_bool(where_expr, &assignment, model_len)? {
            return Err(format!(
                "constraint aspect is outside finite space {}: {}",
                aspect_space.name, term
            ));
        }
    }
    let fields = assignment
        .iter()
        .filter(|(_, value)| !matches!(value, AspectValue::Model(_)))
        .map(|(name, value)| (name.clone(), value.display()))
        .collect();
    Ok((model_name, fields))
}

fn eval_sygus_int(
    term: &SyGuSTerm,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<isize, String> {
    let expr = lexpr::from_str(&term.to_string())
        .map_err(|e| format!("failed to parse integer aspect expression {term}: {e}"))?;
    eval_int(&expr, env, model_len)
}

fn parse_finite_space(cmd: &crate::parser::ast::DefineFiniteCmd) -> Result<FiniteSpace, String> {
    let record_src = cmd
        .body
        .first()
        .ok_or_else(|| format!("define-finite {} expects a record body", cmd.name))?;
    let record = lexpr::from_str(record_src)
        .map_err(|e| format!("failed to parse define-finite {} record: {e}", cmd.name))?;
    let fields = parse_record_fields(&record)?;
    let where_expr = cmd
        .body
        .iter()
        .position(|item| item == ":where")
        .and_then(|idx| cmd.body.get(idx + 1))
        .map(|src| {
            lexpr::from_str(src)
                .map_err(|e| format!("failed to parse define-finite {} :where: {e}", cmd.name))
        })
        .transpose()?;
    Ok(FiniteSpace {
        name: cmd.name.clone(),
        fields,
        where_expr,
    })
}

fn parse_record_fields(record: &Value) -> Result<Vec<FieldDecl>, String> {
    let list = record
        .to_ref_vec()
        .ok_or_else(|| "define-finite record must be a list".to_string())?;
    if list.first().and_then(|v| v.as_symbol()) != Some("record") {
        return Err("define-finite body must start with (record ...)".to_string());
    }
    list.iter()
        .skip(1)
        .map(|field| {
            let pair = field
                .to_ref_vec()
                .ok_or_else(|| format!("record field must be a list, got {field}"))?;
            match pair.as_slice() {
                [name, sort] => Ok(FieldDecl {
                    name: name
                        .as_symbol()
                        .ok_or_else(|| format!("record field name must be a symbol, got {name}"))?
                        .to_string(),
                    sort: sort
                        .as_symbol()
                        .ok_or_else(|| format!("record field sort must be a symbol, got {sort}"))?
                        .to_string(),
                }),
                _ => Err(format!(
                    "record field must have shape (name sort), got {field}"
                )),
            }
        })
        .collect()
}

fn enumerate_aspects(
    space: &FiniteSpace,
    model_name: &str,
    model: &crate::lang::model::Model,
) -> Result<Vec<Vec<(String, AspectValue)>>, String> {
    let (domain_syms, _id2sym, universe_sort) = get_universe_domain_and_map(model);
    let model_len = domain_syms.len();
    let domains = space
        .fields
        .iter()
        .map(|field| {
            let values = match field.sort.as_str() {
                "Model" => vec![AspectValue::Model(model_name.to_string())],
                "Int" => (0..=model_len).map(AspectValue::Int).collect(),
                other if other == universe_sort => domain_syms
                    .iter()
                    .cloned()
                    .map(AspectValue::Symbol)
                    .collect(),
                other => vec![AspectValue::Symbol(other.to_string())],
            };
            Ok((field.name.clone(), values))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut assignments = Vec::new();
    for values in domains
        .iter()
        .map(|(_, values)| values)
        .multi_cartesian_product()
    {
        let assignment = domains
            .iter()
            .zip(values)
            .map(|((name, _), value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();
        let keep = match &space.where_expr {
            Some(expr) => eval_bool(expr, &assignment, model_len)?,
            None => true,
        };
        if keep {
            assignments.push(assignment);
        }
    }
    Ok(assignments)
}

fn lookup<'a>(env: &'a [(String, AspectValue)], name: &str) -> Option<&'a AspectValue> {
    env.iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

fn eval_bool(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<bool, String> {
    if let Some(symbol) = expr.as_symbol() {
        return match symbol {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(format!("unknown boolean symbol {symbol}")),
        };
    }
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected boolean expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "boolean expression must start with an operator: {expr}"
        ));
    };
    match op {
        "and" => {
            list[1..].iter().try_fold(
                true,
                |acc, expr| Ok(acc && eval_bool(expr, env, model_len)?),
            )
        }
        "or" => list[1..].iter().try_fold(false, |acc, expr| {
            Ok(acc || eval_bool(expr, env, model_len)?)
        }),
        "not" if list.len() == 2 => Ok(!eval_bool(list[1], env, model_len)?),
        "=" | "<" | "<=" => {
            if list.len() != 3 {
                return Err(format!("{op} expects two arguments, got {expr}"));
            }
            let lhs = eval_int(list[1], env, model_len)?;
            let rhs = eval_int(list[2], env, model_len)?;
            match op {
                "=" => Ok(lhs == rhs),
                "<" => Ok(lhs < rhs),
                "<=" => Ok(lhs <= rhs),
                _ => unreachable!(),
            }
        }
        _ => Err(format!("unsupported boolean operator {op} in {expr}")),
    }
}

fn eval_semantic_bool(
    problem: &DelisProg,
    model_name: &str,
    expr: &Value,
    env: &[(String, AspectValue)],
) -> Result<bool, String> {
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected semantic boolean expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "semantic boolean expression must start with an operator: {expr}"
        ));
    };
    if list.len() != 3 {
        return Err(format!("{op} expects two arguments, got {expr}"));
    }
    let model_len = current_model_len(env)?;
    match op {
        "=" => {
            let lhs =
                resolve_model_value(problem, model_name, &value_expr(list[1], env, model_len)?)?;
            let rhs =
                resolve_model_value(problem, model_name, &value_expr(list[2], env, model_len)?)?;
            Ok(lhs == rhs)
        }
        "<" | "<=" => {
            let lhs = eval_int(list[1], env, model_len)?;
            let rhs = eval_int(list[2], env, model_len)?;
            match op {
                "<" => Ok(lhs < rhs),
                "<=" => Ok(lhs <= rhs),
                _ => unreachable!(),
            }
        }
        _ => Err(format!(
            "unsupported semantic boolean operator {op} in {expr}"
        )),
    }
}

fn eval_int(
    expr: &Value,
    env: &[(String, AspectValue)],
    model_len: usize,
) -> Result<isize, String> {
    if let Some(value) = expr.as_i64() {
        return Ok(value as isize);
    }
    if let Some(symbol) = expr.as_symbol() {
        return match lookup(env, symbol) {
            Some(AspectValue::Int(value)) => Ok(*value as isize),
            Some(value) => Err(format!("expected Int field {symbol}, got {value:?}")),
            None => Err(format!("unknown Int symbol {symbol}")),
        };
    }
    if let Some(value) = aspect_projection(expr, env) {
        return match value? {
            AspectValue::Int(value) => Ok(value as isize),
            other => Err(format!("expected Int aspect projection, got {other:?}")),
        };
    }
    let list = expr
        .to_ref_vec()
        .ok_or_else(|| format!("expected integer expression, got {expr}"))?;
    let Some(op) = list.first().and_then(|v| v.as_symbol()) else {
        return Err(format!(
            "integer expression must start with an operator: {expr}"
        ));
    };
    match op {
        "+" => list[1..]
            .iter()
            .try_fold(0, |acc, expr| Ok(acc + eval_int(expr, env, model_len)?)),
        "-" if list.len() == 2 => Ok(-eval_int(list[1], env, model_len)?),
        "-" => {
            let Some((first, rest)) = list[1..].split_first() else {
                return Err(format!("- expects at least one argument, got {expr}"));
            };
            let init = eval_int(first, env, model_len)?;
            rest.iter()
                .try_fold(init, |acc, expr| Ok(acc - eval_int(expr, env, model_len)?))
        }
        "len" if list.len() == 2 => {
            model_name_expr(list[1], env, model_len)?;
            Ok(model_len as isize)
        }
        _ => Err(format!("unsupported integer operator {op} in {expr}")),
    }
}

fn semantic_relation_name(body: &[String]) -> Result<String, String> {
    let Some(header) = body.first() else {
        return Err("define-semantics expects a relation header".to_string());
    };
    let text = header.trim();
    if let Some(name) = text
        .strip_prefix("((")
        .and_then(|rest| rest.split_whitespace().next())
    {
        return Ok(name.trim_end_matches(')').to_string());
    }
    Err(format!(
        "could not extract semantic relation name from define-semantics header: {text}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> DelisProg {
        crate::parser::ast::parse(input, None).expect("frontend smoke program should parse")
    }

    #[test]
    fn enumerates_subword_aspects_and_constraint_polarities() {
        let problem = parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
(check-synth)
"#,
        );

        let summary = lower(&problem).expect("frontend lowering should summarize");

        assert_eq!(summary.bit_mappings.len(), 16);
        assert_eq!(summary.semantic_relations, vec!["Regex.Sem"]);
        assert_eq!(summary.constraint_bits.len(), 2);
        assert_eq!(summary.constraint_bits[0].model, "w1");
        assert!(summary.constraint_bits[0].expected);
        assert_eq!(summary.constraint_bits[0].bit, 2);
        assert_eq!(summary.constraint_bits[1].model, "w5");
        assert!(!summary.constraint_bits[1].expected);
        assert_eq!(summary.constraint_bits[1].bit, 9);
    }

    #[test]
    fn finite_aspect_fields_are_not_named_primitives() {
        let problem = parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite Span
  (record (m Model) (left Int) (right Int))
  :where (and (<= 0 left) (<= left right) (<= right (len m))))
(define-semantics
  ((Regex.Sem (r Regex) (asp Span)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (Span w 0 (len w))))
(check-synth)
"#,
        );

        let summary = lower(&problem).expect("frontend lowering should summarize");

        assert_eq!(summary.bit_mappings.len(), 6);
        assert_eq!(summary.constraint_bits.len(), 1);
        assert_eq!(summary.constraint_bits[0].bit, 2);
        assert_eq!(
            summary.bit_mappings[2].aspect.fields,
            vec![
                ("left".to_string(), "0".to_string()),
                ("right".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn parses_direct_semantic_constraints() {
        let problem = parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
(check-synth)
"#,
        );

        let summary = lower(&problem).expect("frontend lowering should summarize");

        assert_eq!(summary.bit_mappings.len(), 16);
        assert_eq!(summary.constraint_bits.len(), 2);
        assert_eq!(summary.constraint_bits[0].relation, "Regex.Sem");
        assert_eq!(summary.constraint_bits[0].model, "w1");
        assert!(summary.constraint_bits[0].expected);
        assert_eq!(summary.constraint_bits[0].bit, 2);
        assert_eq!(summary.constraint_bits[1].relation, "Regex.Sem");
        assert_eq!(summary.constraint_bits[1].model, "w5");
        assert!(!summary.constraint_bits[1].expected);
        assert_eq!(summary.constraint_bits[1].bit, 9);
    }

    #[test]
    fn semantic_predicates_use_aspect_projection() {
        let problem = parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match ((($0) true) ((_) false))))
  (interp-fun b (match ((($1) true) ((_) false)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun b (match (((_) false)))))
(declare-term-types ((Regex 0)) (((a) (b) (re.++ Regex Regex))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r
     ((a) (and (= (j asp) (+ (i asp) 1)) (a (i asp))))
     ((b) (and (= (j asp) (+ (i asp) 1)) (b (i asp))))
     ((re.++ x y)
      (exists ((k (int-range (i asp) (j asp))))
        (and (Regex.Sem x (ReAsp (model asp) (i asp) k))
             (Regex.Sem y (ReAsp (model asp) k (j asp)))))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
(check-synth)
"#,
        );

        let sygus = to_sygus(&problem).expect("projected semantic predicates should lower");

        assert!(!sygus.is_empty());
    }

    #[test]
    fn detects_two_way_navigation() {
        let problem = parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Root 0)) (((cfg.top.S Root Root))))
(define-finite CfgAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r ((cfg.top.S x y) (Cfg.Sem (up r) asp)))))
(synth-fun r () Root)
(constraint (Cfg.Sem r (CfgAsp w1 0 (len w1))))
(check-synth)
"#,
        );

        let summary = lower(&problem).expect("frontend lowering should summarize");

        assert!(summary.needs_saturation);
    }
}
