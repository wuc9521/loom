use crate::{
    backends::smt2::{
        ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm, PredicatePolarity},
        pred::{dnf_and, dnf_false, dnf_or, dnf_true, emit_dnf_rules, Pred, PredAtom, PredDnf},
    },
    compile::{
        native::{EnvView, SemanticContext},
        semantic::{interped_bool, interped_value_matches},
        syntax::{state_name, term_sort, TermConstructor, TermSignature},
        types::{
            AspectKey, AspectValue, FieldDecl, FieldDomain, SemanticDomain, SemanticKind,
            SemanticProgram,
        },
    },
    lang::eval::{eval_symbols, get_domain_for_sort},
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;
use sygus_parser::{Sort, SortDecl};

#[derive(Clone)]
struct ChildBinding {
    term: HornTerm,
}

#[derive(Default, Clone)]
struct Env {
    aspect: HashMap<String, AspectKey>,
    values: HashMap<String, AspectValue>,
    children: HashMap<String, ChildBinding>,
    current_sort: Option<String>,
    current_var: Option<String>,
    current_term: Option<HornTerm>,
}

impl EnvView for Env {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        &self.aspect
    }

    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
}

pub fn compile(problem: &DelisProg, program: SemanticProgram) -> Result<HornProg, String> {
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("CHC compilation requires declare-term-types".to_string());
    };
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    ChcCompiler::new(problem, program, sig)?.compile()
}

struct ChcCompiler<'a> {
    program: SemanticProgram,
    sig: TermSignature,
    ctx: SemanticContext<'a>,
    predicates: HashMap<(String, AspectKey, Option<AspectValue>), Pred>,
    monomorphic_terms: bool,
}

impl<'a> ChcCompiler<'a> {
    fn new(
        problem: &'a DelisProg,
        program: SemanticProgram,
        sig: TermSignature,
    ) -> Result<Self, String> {
        let ctx = SemanticContext::new(problem, &program)?;
        let mut predicates = HashMap::new();
        for domain in &program.domains {
            for bit in &domain.bits {
                let state = domain_state_name(domain, &bit.aspect, bit.value.as_ref());
                predicates.insert(
                    (
                        domain.relation.clone(),
                        bit.aspect.clone(),
                        bit.value.clone(),
                    ),
                    Pred::new(format!("q{}", state), format!("nq{}", state)),
                );
            }
        }
        Ok(Self {
            program,
            sig,
            ctx,
            predicates,
            monomorphic_terms: false,
        })
    }

    fn compile(mut self) -> Result<HornProg, String> {
        self.monomorphic_terms = self.requires_monomorphic_terms()?;
        let mut out = HornProg::default();
        out.decls.push(HornDecl::SetLogic("HORN".to_string()));
        if self.monomorphic_terms {
            out.decls.push(HornDecl::DeclareDatatypes(
                vec![SortDecl {
                    symbol: "Term".to_string(),
                    arity: 0,
                }],
                vec![self.sig.datatype_constructors()],
            ));
        } else {
            let (sorts, decls) = self.sig.datatype_decls();
            out.decls.push(HornDecl::DeclareDatatypes(sorts, decls));
        }

        for domain in &self.program.domains {
            for bit in &domain.bits {
                let state = domain_state_name(domain, &bit.aspect, bit.value.as_ref());
                let pred = self.pred_for(&domain.relation, &bit.aspect, bit.value.as_ref())?;
                for (name, polarity) in [
                    (pred_name(&pred, false), PredicatePolarity::Positive),
                    (pred_name(&pred, true), PredicatePolarity::Negative),
                ] {
                    let declared_sort = if self.monomorphic_terms {
                        term_sort()
                    } else {
                        Sort::from_name(&domain.term_sort)
                    };
                    out.decls.push(HornDecl::DeclareRel(
                        name.clone(),
                        vec![declared_sort],
                    ));
                    out.register_predicate(
                        name,
                        state.clone(),
                        polarity,
                        if self.monomorphic_terms {
                            "Term".to_string()
                        } else {
                            domain.term_sort.clone()
                        },
                    );
                }
            }
        }
        out.decls.push(HornDecl::DeclareRel(
            "Goal".to_string(),
            vec![if self.monomorphic_terms {
                term_sort()
            } else {
                Sort::from_name(self.sig.start_sort())
            }],
        ));

        for cons in &self.sig.constructors {
            self.emit_constructor_rules(&mut out, cons)?;
        }
        self.emit_goal(&mut out)?;
        Ok(out)
    }

    fn emit_constructor_rules(
        &self,
        out: &mut HornProg,
        cons: &TermConstructor,
    ) -> Result<(), String> {
        let mut matched = false;
        for domain in &self.program.domains {
            let spec = self.ctx.spec_for_relation(&domain.relation)?;
            let Some(case) = self
                .ctx
                .cases()
                .get(&spec.relation)
                .and_then(|cases| cases.get(&cons.name))
            else {
                continue;
            };
            matched = true;
            if case.vars.len() != cons.arg_sorts.len() {
                return Err(format!(
                    "match case for constructor {} binds {} variables, expected {}",
                    cons.name,
                    case.vars.len(),
                    cons.arg_sorts.len()
                ));
            }
            let head_term = HornTerm::app(
                &cons.name,
                case.vars.iter().map(HornTerm::var).collect::<Vec<_>>(),
            );
            let mut env = Env::default();
            env.current_sort = Some(cons.result_sort.clone());
            env.current_var = Some(case.self_var.clone());
            env.current_term = Some(head_term.clone());
            for var in &case.vars {
                env.children.insert(
                    var.clone(),
                    ChildBinding {
                        term: HornTerm::var(var),
                    },
                );
            }
            for bit in &domain.bits {
                env.aspect.insert("asp".to_string(), bit.aspect.clone());
                let dnf = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) => self.eval_bool(&case.body, &env)?,
                    (SemanticKind::FiniteValue { .. }, Some(value)) => {
                        self.eval_finite_eq(&case.body, value, &env)?
                    }
                    _ => {
                        return Err(format!(
                            "malformed semantic domain bit {} for {}",
                            bit.bit, domain.relation
                        ));
                    }
                };
                let not_dnf = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) => self.eval_bool_neg(&case.body, &env)?,
                    (SemanticKind::FiniteValue { .. }, Some(value)) => {
                        self.eval_finite_neq(&case.body, value, &env)?
                    }
                    _ => unreachable!(),
                };
                let head = PredAtom::new(
                    self.pred_for(&domain.relation, &bit.aspect, bit.value.as_ref())?,
                    head_term.clone(),
                );
                emit_dnf_rules(&mut out.rules, head.clone(), dnf.clone());
                emit_dnf_rules(&mut out.rules, head.dual(), not_dnf);
            }
        }
        if !matched {
            return Err(format!("missing semantic case for constructor {}", cons.name));
        }
        Ok(())
    }

    fn emit_goal(&self, out: &mut HornProg) -> Result<(), String> {
        let mut body_atoms = Vec::new();
        for constraint in &self.program.constraints {
            let domain = self.ctx.domain_for_relation(&constraint.relation)?;
            if domain.kind != SemanticKind::Bool {
                return Err(format!(
                    "top-level constraint {} must target Bool semantics",
                    constraint.relation
                ));
            }
            if !self.monomorphic_terms && domain.term_sort != self.sig.start_sort() {
                return Err(format!(
                    "constraint relation {} has term sort {}, expected start sort {}",
                    constraint.relation,
                    domain.term_sort,
                    self.sig.start_sort()
                ));
            }
            let aspect = AspectKey {
                relation: constraint.relation.clone(),
                model: constraint.model.clone(),
                fields: constraint.fields.clone(),
            };
            let pred = self.pred_for(&constraint.relation, &aspect, None)?;
            body_atoms.push(HornAtom {
                pred: pred_name(&pred, !constraint.expected),
                arg: HornTerm::var("r"),
            });
        }
        out.rules.push(HornRule {
            body_atoms,
            head: HornAtom {
                pred: "Goal".to_string(),
                arg: HornTerm::var("r"),
            },
        });
        out.queries.push(HornQuery {
            body_atoms: vec![HornAtom {
                pred: "Goal".to_string(),
                arg: HornTerm::var("r"),
            }],
        });
        Ok(())
    }

    fn eval_bool(&self, expr: &Value, env: &Env) -> Result<PredDnf, String> {
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(dnf_true()),
                "false" => Ok(dnf_false()),
                _ => Err(format!("expected Bool expression, got {expr}")),
            };
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected Bool expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("Bool expression operator must be a symbol: {expr}"))?;
        match op {
            "not" if items.len() == 2 => self.eval_bool_neg(items[1], env),
            "and" => items[1..].iter().try_fold(dnf_true(), |acc, item| {
                Ok(dnf_and(acc, self.eval_bool(item, env)?))
            }),
            "or" => items[1..].iter().try_fold(dnf_false(), |acc, item| {
                Ok(dnf_or(acc, self.eval_bool(item, env)?))
            }),
            "=" if items.len() == 3 => self.eval_equality(items[1], items[2], env, true),
            "<" if items.len() == 3 => self.eval_int_compare(items[1], items[2], env, IntCmp::Lt),
            "<=" if items.len() == 3 => {
                self.eval_int_compare(items[1], items[2], env, IntCmp::Le)
            }
            "forall" | "exists" if items.len() == 3 => {
                self.eval_quantifier(op, items[1], items[2], env)
            }
            relation if self.ctx.has_relation(relation) => {
                self.eval_semantic_bool_call(relation, &items[1..], env)
            }
            func if self.ctx.has_func(func) => {
                self.eval_model_bool_call(func, &items[1..], env, true)
            }
            _ => Err(format!("unsupported Bool expression: {expr}")),
        }
    }

    fn eval_bool_neg(&self, expr: &Value, env: &Env) -> Result<PredDnf, String> {
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(dnf_false()),
                "false" => Ok(dnf_true()),
                _ => Err(format!("expected Bool expression, got {expr}")),
            };
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected Bool expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("Bool expression operator must be a symbol: {expr}"))?;
        match op {
            "not" if items.len() == 2 => self.eval_bool(items[1], env),
            "and" => items[1..].iter().try_fold(dnf_false(), |acc, item| {
                Ok(dnf_or(acc, self.eval_bool_neg(item, env)?))
            }),
            "or" => items[1..].iter().try_fold(dnf_true(), |acc, item| {
                Ok(dnf_and(acc, self.eval_bool_neg(item, env)?))
            }),
            "=" if items.len() == 3 => self.eval_equality(items[1], items[2], env, false),
            "<" if items.len() == 3 => self.eval_int_compare(items[1], items[2], env, IntCmp::Ge),
            "<=" if items.len() == 3 => {
                self.eval_int_compare(items[1], items[2], env, IntCmp::Gt)
            }
            "forall" if items.len() == 3 => {
                self.eval_quantifier("exists-neg", items[1], items[2], env)
            }
            "exists" if items.len() == 3 => {
                self.eval_quantifier("forall-neg", items[1], items[2], env)
            }
            relation if self.ctx.has_relation(relation) => {
                self.eval_semantic_bool_call_neg(relation, &items[1..], env)
            }
            func if self.ctx.has_func(func) => {
                self.eval_model_bool_call(func, &items[1..], env, false)
            }
            _ => Err(format!("unsupported Bool expression: {expr}")),
        }
    }

    fn eval_quantifier(
        &self,
        op: &str,
        bindings: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let binding_list = bindings
            .to_ref_vec()
            .ok_or_else(|| format!("quantifier binding list must be a list: {bindings}"))?;
        if binding_list.len() != 1 {
            return Err(format!(
                "native CHC lowering currently supports one binding per quantifier, got {bindings}"
            ));
        }
        let binding = binding_list[0]
            .to_ref_vec()
            .ok_or_else(|| format!("quantifier binding must be a list: {}", binding_list[0]))?;
        let [var, domain_expr] = binding.as_slice() else {
            return Err(format!(
                "quantifier binding must have shape (v domain): {}",
                binding_list[0]
            ));
        };
        let var = var
            .as_symbol()
            .ok_or_else(|| format!("quantifier variable must be a symbol: {var}"))?;
        let conjunctive = op == "forall" || op == "forall-neg";
        let neg_body = op == "forall-neg" || op == "exists-neg";
        let mut out = if conjunctive { dnf_true() } else { dnf_false() };
        for value in self.quantifier_values(domain_expr, env)? {
            let mut next = env.clone();
            next.values.insert(var.to_string(), value);
            let part = if neg_body {
                self.eval_bool_neg(body, &next)?
            } else {
                self.eval_bool(body, &next)?
            };
            out = if conjunctive {
                dnf_and(out, part)
            } else {
                dnf_or(out, part)
            };
        }
        Ok(out)
    }

    fn requires_monomorphic_terms(&self) -> Result<bool, String> {
        let cons_sort = self
            .sig
            .constructors
            .iter()
            .map(|cons| (cons.name.clone(), cons.result_sort.clone()))
            .collect::<HashMap<_, _>>();
        for domain in &self.program.domains {
            let spec = self.ctx.spec_for_relation(&domain.relation)?;
            let Some(cases) = self.ctx.cases().get(&spec.relation) else {
                continue;
            };
            for cons_name in cases.keys() {
                let Some(sort) = cons_sort.get(cons_name) else {
                    continue;
                };
                if sort != &domain.term_sort {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn quantifier_values(
        &self,
        domain_expr: &Value,
        env: &Env,
    ) -> Result<Vec<AspectValue>, String> {
        let items = domain_expr
            .to_ref_vec()
            .ok_or_else(|| format!("unsupported domain expression: {domain_expr}"))?;
        match items.first().and_then(|item| item.as_symbol()) {
            Some("int-range") if items.len() == 3 => {
                let lo = self.eval_int(items[1], env)?;
                let hi = self.eval_int(items[2], env)?;
                if lo < 0 || hi < lo {
                    return Ok(Vec::new());
                }
                Ok((lo as usize..=hi as usize).map(AspectValue::Int).collect())
            }
            _ => {
                let (model, sort) = self.ctx.parse_domain_expr(domain_expr, env)?;
                let model_ref = self.ctx.model(&model)?;
                let values = get_domain_for_sort(model_ref, &sort)
                    .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
                Ok(values
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort.clone(),
                        name,
                    })
                    .collect())
            }
        }
    }

    fn eval_equality(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
        expected: bool,
    ) -> Result<PredDnf, String> {
        if self.ctx.infer_finite_sort(lhs, env).is_some() || self.ctx.infer_finite_sort(rhs, env).is_some() {
            if expected {
                self.eval_finite_terms_equal(lhs, rhs, env)
            } else {
                self.eval_finite_terms_not_equal(lhs, rhs, env)
            }
        } else if let (Ok(lhs_value), Ok(rhs_value)) =
            (self.ctx.eval_value(lhs, env), self.ctx.eval_value(rhs, env))
        {
            match (&lhs_value, &rhs_value) {
                (AspectValue::Int(_), AspectValue::Int(_)) => {
                    Ok(bool_dnf((lhs_value == rhs_value) == expected))
                }
                _ => Ok(bool_dnf((lhs_value == rhs_value) == expected)),
            }
        } else {
            let lhs = self.eval_int(lhs, env)?;
            let rhs = self.eval_int(rhs, env)?;
            Ok(bool_dnf((lhs == rhs) == expected))
        }
    }

    fn eval_int_compare(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
        cmp: IntCmp,
    ) -> Result<PredDnf, String> {
        let lhs = self.eval_int(lhs, env)?;
        let rhs = self.eval_int(rhs, env)?;
        let value = match cmp {
            IntCmp::Lt => lhs < rhs,
            IntCmp::Le => lhs <= rhs,
            IntCmp::Gt => lhs > rhs,
            IntCmp::Ge => lhs >= rhs,
        };
        Ok(bool_dnf(value))
    }

    fn eval_semantic_bool_call(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<PredDnf, String> {
        let domain = self.ctx.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool {
            return Err(format!("{relation} does not return Bool"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let terms = self.child_terms(args[0], env)?;
        let mut out = dnf_false();
        for (guard, aspect) in self.aspect_variants(args[1], relation, env)? {
            let pred = self.pred_for(relation, &aspect, None)?;
            for term in &terms {
                out = dnf_or(
                    out,
                    dnf_and(
                        guard.clone(),
                        vec![vec![PredAtom::new(pred.clone(), term.clone())]],
                    ),
                );
            }
        }
        Ok(out)
    }

    fn eval_semantic_bool_call_neg(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<PredDnf, String> {
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let terms = self.child_terms(args[0], env)?;
        let mut out = dnf_false();
        for (guard, aspect) in self.aspect_variants(args[1], relation, env)? {
            let pred = self.pred_for(relation, &aspect, None)?.dual();
            for term in &terms {
                out = dnf_or(
                    out,
                    dnf_and(
                        guard.clone(),
                        vec![vec![PredAtom::new(pred.clone(), term.clone())]],
                    ),
                );
            }
        }
        Ok(out)
    }

    fn eval_model_bool_call(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
        expected: bool,
    ) -> Result<PredDnf, String> {
        let sig = self.ctx.func_sigs.get(func).unwrap();
        if sig.ret_sort.to_string() != "Bool" {
            return Err(format!("function {func} does not return Bool"));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let domains = self.ctx.argument_domains(sig, model_ref)?;
        let mut out = dnf_false();
        for tuple in domains
            .iter()
            .map(|values| values.iter())
            .multi_cartesian_product()
        {
            let arg_names = tuple
                .iter()
                .map(|value| value.display())
                .collect::<Vec<_>>();
            let Some(value) = eval_symbols(func, &arg_names, &model_ref.func_interps) else {
                continue;
            };
            if interped_bool(value) != expected {
                continue;
            }
            let mut conj = dnf_true();
            for (arg_expr, value) in args.iter().zip(tuple.iter()) {
                conj = dnf_and(conj, self.eval_finite_eq(arg_expr, value, env)?);
            }
            out = dnf_or(out, conj);
        }
        Ok(out)
    }

    fn eval_finite_terms_equal(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let sort = self
            .ctx
            .infer_finite_sort(lhs, env)
            .or_else(|| self.ctx.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for equality (= {lhs} {rhs})"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = get_domain_for_sort(model_ref, &sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let mut out = dnf_false();
        for name in values {
            let value = AspectValue::Univ {
                sort: sort.clone(),
                name,
            };
            out = dnf_or(
                out,
                dnf_and(
                    self.eval_finite_eq(lhs, &value, env)?,
                    self.eval_finite_eq(rhs, &value, env)?,
                ),
            );
        }
        Ok(out)
    }

    fn eval_finite_terms_not_equal(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let sort = self
            .ctx
            .infer_finite_sort(lhs, env)
            .or_else(|| self.ctx.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for inequality (= {lhs} {rhs})"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = get_domain_for_sort(model_ref, &sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let mut out = dnf_false();
        for lhs_name in &values {
            for rhs_name in &values {
                if lhs_name == rhs_name {
                    continue;
                }
                let lhs_value = AspectValue::Univ {
                    sort: sort.clone(),
                    name: lhs_name.clone(),
                };
                let rhs_value = AspectValue::Univ {
                    sort: sort.clone(),
                    name: rhs_name.clone(),
                };
                out = dnf_or(
                    out,
                    dnf_and(
                        self.eval_finite_eq(lhs, &lhs_value, env)?,
                        self.eval_finite_eq(rhs, &rhs_value, env)?,
                    ),
                );
            }
        }
        Ok(out)
    }

    fn eval_finite_eq(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        if let Ok(value) = self.ctx.eval_value(expr, env) {
            return Ok(if self.values_match_target(&value, target, env)? {
                dnf_true()
            } else {
                dnf_false()
            });
        }
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(if value == target {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            let model = self.ctx.current_model(env)?;
            let model_ref = self.ctx.model(&model)?;
            if let Some(value) = eval_symbols(symbol, &[], &model_ref.func_interps) {
                return Ok(if interped_value_matches(value, target) {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            return Ok(if symbol == target.display() {
                dnf_true()
            } else {
                dnf_false()
            });
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected finite expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("finite expression operator must be a symbol: {expr}"))?;
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspect.get(aspect_arg) {
                    if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                        return Ok(if value == target {
                            dnf_true()
                        } else {
                            dnf_false()
                        });
                    }
                }
            }
        }
        if self.ctx.has_relation(op) {
            return self.eval_semantic_finite_call_eq(op, &items[1..], target, env);
        }
        if self.ctx.has_func(op) {
            return self.eval_model_finite_call_eq(op, &items[1..], target, env);
        }
        Err(format!("unsupported finite expression: {expr}"))
    }

    fn eval_finite_neq(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        if let Ok(value) = self.ctx.eval_value(expr, env) {
            return Ok(if self.values_match_target(&value, target, env)? {
                dnf_false()
            } else {
                dnf_true()
            });
        }
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(if value != target {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            let model = self.ctx.current_model(env)?;
            let model_ref = self.ctx.model(&model)?;
            if let Some(value) = eval_symbols(symbol, &[], &model_ref.func_interps) {
                return Ok(if !interped_value_matches(value, target) {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            return Ok(if symbol != target.display() {
                dnf_true()
            } else {
                dnf_false()
            });
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected finite expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("finite expression operator must be a symbol: {expr}"))?;
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspect.get(aspect_arg) {
                    if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                        return Ok(if value != target {
                            dnf_true()
                        } else {
                            dnf_false()
                        });
                    }
                }
            }
        }
        if self.ctx.has_relation(op) {
            let terms = self.child_terms(items[1], env)?;
            let mut out = dnf_false();
            for (guard, aspect) in self.aspect_variants(items[2], op, env)? {
                let pred = self.pred_for(op, &aspect, Some(target))?.dual();
                for term in &terms {
                    out = dnf_or(
                        out,
                        dnf_and(
                            guard.clone(),
                            vec![vec![PredAtom::new(pred.clone(), term.clone())]],
                        ),
                    );
                }
            }
            return Ok(out);
        }
        if self.ctx.has_func(op) {
            return self.eval_model_finite_call_neq(op, &items[1..], target, env);
        }
        Err(format!("unsupported finite expression: {expr}"))
    }

    fn eval_semantic_finite_call_eq(
        &self,
        relation: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let terms = self.child_terms(args[0], env)?;
        let mut out = dnf_false();
        for (guard, aspect) in self.aspect_variants(args[1], relation, env)? {
            let pred = self.pred_for(relation, &aspect, Some(target))?;
            for term in &terms {
                out = dnf_or(
                    out,
                    dnf_and(
                        guard.clone(),
                        vec![vec![PredAtom::new(pred.clone(), term.clone())]],
                    ),
                );
            }
        }
        Ok(out)
    }

    fn eval_model_finite_call_eq(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let sig = self.ctx.func_sigs.get(func).unwrap();
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let domains = self.ctx.argument_domains(sig, model_ref)?;
        let mut out = dnf_false();
        for tuple in domains
            .iter()
            .map(|values| values.iter())
            .multi_cartesian_product()
        {
            let arg_names = tuple
                .iter()
                .map(|value| value.display())
                .collect::<Vec<_>>();
            let Some(value) = eval_symbols(func, &arg_names, &model_ref.func_interps) else {
                continue;
            };
            if !interped_value_matches(value, target) {
                continue;
            }
            let mut conj = dnf_true();
            for (arg_expr, value) in args.iter().zip(tuple.iter()) {
                conj = dnf_and(conj, self.eval_finite_eq(arg_expr, value, env)?);
            }
            out = dnf_or(out, conj);
        }
        Ok(out)
    }

    fn eval_model_finite_call_neq(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let sig = self.ctx.func_sigs.get(func).unwrap();
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let domains = self.ctx.argument_domains(sig, model_ref)?;
        let mut out = dnf_false();
        for tuple in domains
            .iter()
            .map(|values| values.iter())
            .multi_cartesian_product()
        {
            let arg_names = tuple
                .iter()
                .map(|value| value.display())
                .collect::<Vec<_>>();
            let Some(value) = eval_symbols(func, &arg_names, &model_ref.func_interps) else {
                continue;
            };
            if interped_value_matches(value, target) {
                continue;
            }
            let mut conj = dnf_true();
            for (arg_expr, value) in args.iter().zip(tuple.iter()) {
                conj = dnf_and(conj, self.eval_finite_eq(arg_expr, value, env)?);
            }
            out = dnf_or(out, conj);
        }
        Ok(out)
    }

    fn child_terms(&self, expr: &Value, env: &Env) -> Result<Vec<HornTerm>, String> {
        if let Some(child) = expr.as_symbol() {
            if env.current_var.as_deref() == Some(child) {
                let term = env
                    .current_term
                    .clone()
                    .ok_or_else(|| "missing current term for recursive semantic call".to_string())?;
                return Ok(vec![term]);
            }
            let term = env
                .children
                .get(child)
                .map(|binding| binding.term.clone())
                .ok_or_else(|| format!("unknown semantic child variable {child}"))?;
            return Ok(vec![term]);
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("semantic call term must be a child variable: {expr}"))?;
        match items.as_slice() {
            [op, inner] if op.as_symbol() == Some("parent") => {
                if inner.as_symbol().is_none() {
                    return Err(format!(
                        "parent navigation target must be a symbol naming the current term: {expr}"
                    ));
                }
                let sort = env
                    .current_sort
                    .as_deref()
                    .ok_or_else(|| "missing current sort for parent navigation".to_string())?;
                let term = env
                    .current_term
                    .clone()
                    .ok_or_else(|| "missing current term for parent navigation".to_string())?;
                Ok(self.sig.parent_contexts(sort, term))
            }
            _ => Err(format!(
                "unsupported semantic call term navigation: {expr}"
            )),
        }
    }

    fn aspect_variants(
        &self,
        expr: &Value,
        relation: &str,
        env: &Env,
    ) -> Result<Vec<(PredDnf, AspectKey)>, String> {
        if let Some(symbol) = expr.as_symbol() {
            let aspect = env
                .aspect
                .get(symbol)
                .ok_or_else(|| format!("unknown aspect variable {symbol}"))?;
            let mut aspect = aspect.clone();
            aspect.relation = relation.to_string();
            return Ok(vec![(dnf_true(), aspect)]);
        }
        let domain = self.ctx.domain_for_relation(relation)?;
        let space = self.ctx.aspect_space(&domain.aspect_space)?;
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("aspect expression must be a list: {expr}"))?;
        let cons = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("aspect constructor must be a symbol: {expr}"))?;
        if cons != space.name {
            return Err(format!("aspect constructor must be {}", space.name));
        }
        if items.len() - 1 != space.fields.len() {
            return Err(format!(
                "aspect constructor {} provides {} scalar fields, expected {}",
                cons,
                items.len().saturating_sub(2),
                space.fields.len().saturating_sub(1)
            ));
        }

        let mut partials = vec![(dnf_true(), None, Vec::new())];
        for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
            let variants = self.aspect_field_variants(field, arg, env)?;
            let mut next = Vec::new();
            for (guard0, model0, fields0) in &partials {
                for (guard1, value) in &variants {
                    let mut fields = fields0.clone();
                    let model = match (&field.domain, value) {
                        (FieldDomain::Sort(sort), AspectValue::Model(name)) if sort == "Model" => {
                            Some(name.clone())
                        }
                        (FieldDomain::Sort(sort), _) if sort == "Model" => {
                            return Err(format!(
                                "aspect field {} expected model value, got {}",
                                field.name,
                                value.display()
                            ));
                        }
                        _ => {
                            fields.push((field.name.clone(), value.clone()));
                            model0.clone()
                        }
                    };
                    next.push((dnf_and(guard0.clone(), guard1.clone()), model, fields));
                }
            }
            partials = next;
        }

        partials
            .into_iter()
            .map(|(guard, model, fields)| {
                Ok((
                    guard,
                    AspectKey {
                        relation: relation.to_string(),
                        model: model.ok_or_else(|| format!("aspect {cons} has no model field"))?,
                        fields,
                    },
                ))
            })
            .collect()
    }

    fn aspect_field_variants(
        &self,
        field: &FieldDecl,
        arg: &Value,
        env: &Env,
    ) -> Result<Vec<(PredDnf, AspectValue)>, String> {
        match &field.domain {
            FieldDomain::Sort(sort) if sort == "Model" => Ok(vec![(
                dnf_true(),
                AspectValue::Model(self.ctx.eval_model_name(arg, env)?),
            )]),
            FieldDomain::Sort(sort) => {
                if let Ok(value) = self.ctx.eval_value_for_sort(arg, sort, env) {
                    return Ok(vec![(dnf_true(), value)]);
                }
                if sort == "Int" {
                    return Ok(Vec::new());
                }
                self.dynamic_value_variants(sort, arg, env)
            }
            FieldDomain::Enum(values) => {
                let value = self.ctx.eval_value(arg, env)?;
                if values.contains(&value.display()) {
                    Ok(vec![(dnf_true(), value)])
                } else {
                    Err(format!(
                        "aspect field {} expected one of ({})",
                        field.name,
                        values.join(" ")
                    ))
                }
            }
            FieldDomain::Union { bot, sorts } => {
                if let Ok(value) = self.ctx.eval_value(arg, env) {
                    let ok = match &value {
                        AspectValue::Symbol(symbol) => symbol == bot,
                        AspectValue::Univ { sort, .. } => sorts.contains(sort),
                        _ => false,
                    };
                    if ok {
                        return Ok(vec![(dnf_true(), value)]);
                    }
                }
                let mut out = vec![(dnf_true(), AspectValue::Symbol(bot.clone()))];
                for sort in sorts {
                    out.extend(self.dynamic_value_variants(sort, arg, env)?);
                }
                Ok(out)
            }
        }
    }

    fn dynamic_value_variants(
        &self,
        sort: &str,
        expr: &Value,
        env: &Env,
    ) -> Result<Vec<(PredDnf, AspectValue)>, String> {
        if self.ctx.infer_finite_sort(expr, env).as_deref() != Some(sort) {
            return Err(format!("unsupported value expression: {expr}"));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = get_domain_for_sort(model_ref, sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        values
            .into_iter()
            .map(|name| {
                let value = AspectValue::Univ {
                    sort: sort.to_string(),
                    name,
                };
                Ok((self.eval_finite_eq(expr, &value, env)?, value))
            })
            .collect()
    }

    fn eval_int(&self, expr: &Value, env: &Env) -> Result<isize, String> {
        self.ctx.eval_int_expr(expr, env)
    }

    fn values_match_target(
        &self,
        value: &AspectValue,
        target: &AspectValue,
        env: &Env,
    ) -> Result<bool, String> {
        if value == target {
            return Ok(true);
        }
        match (value, target) {
            (AspectValue::Int(idx), AspectValue::Univ { sort, name }) => {
                let model = self.ctx.current_model(env)?;
                let model_ref = self.ctx.model(&model)?;
                let Some(domain) = get_domain_for_sort(model_ref, sort) else {
                    return Ok(false);
                };
                Ok(domain.get(*idx).is_some_and(|elem| elem == name))
            }
            _ => Ok(false),
        }
    }

    fn pred_for(
        &self,
        relation: &str,
        aspect: &AspectKey,
        value: Option<&AspectValue>,
    ) -> Result<Pred, String> {
        self.predicates
            .get(&(relation.to_string(), aspect.clone(), value.cloned()))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "no predicate for {}({}; {}){}",
                    relation,
                    aspect.model,
                    aspect.display_fields(),
                    value
                        .map(|v| format!(" = {}", v.display()))
                        .unwrap_or_default()
                )
            })
    }
}

fn domain_state_name(
    domain: &SemanticDomain,
    aspect: &AspectKey,
    value: Option<&AspectValue>,
) -> String {
    let mut state = state_name(aspect);
    if domain.kind != SemanticKind::Bool {
        if let Some(value) = value {
            state.push('.');
            state.push_str(&value.display());
        }
    }
    state
}

fn pred_name(pred: &Pred, neg: bool) -> String {
    if neg {
        pred.dual().name()
    } else {
        pred.name()
    }
}

#[derive(Clone, Copy)]
enum IntCmp {
    Lt,
    Le,
    Gt,
    Ge,
}

fn bool_dnf(value: bool) -> PredDnf {
    if value {
        dnf_true()
    } else {
        dnf_false()
    }
}
