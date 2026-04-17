use crate::{
    backends::smt2::{
        ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm, PredicatePolarity},
        pred::{dnf_and, dnf_false, dnf_or, dnf_true, emit_dnf_rules, Pred, PredAtom, PredDnf},
    },
    compile::{
        semantic::{interped_bool, interped_value_matches, parse_cases, Case},
        syntax::{state_name, term_sort, TermConstructor, TermSignature},
        types::{
            AspectKey, AspectValue, FieldDomain, FiniteSpace, SemanticDomain, SemanticKind,
            SemanticProgram, SemanticSpec,
        },
    },
    lang::{
        eval::{eval_symbols, get_domain_for_sort},
        model::Model,
        signature::FuncSig,
    },
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use lexpr::Value;
use std::collections::{BTreeSet, HashMap};
use sygus_parser::SortDecl;

#[derive(Clone)]
struct ChildBinding {
    term: HornTerm,
}

#[derive(Default, Clone)]
struct Env {
    aspect: HashMap<String, AspectKey>,
    values: HashMap<String, AspectValue>,
    children: HashMap<String, ChildBinding>,
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
    NativeCompiler::new(problem, program, sig)?.compile()
}

struct NativeCompiler<'a> {
    program: SemanticProgram,
    sig: TermSignature,
    models: HashMap<String, &'a Model>,
    func_sigs: HashMap<String, FuncSig>,
    spaces: HashMap<String, FiniteSpace>,
    specs: HashMap<String, SemanticSpec>,
    domains_by_relation: HashMap<String, SemanticDomain>,
    domain_by_sort: HashMap<String, SemanticDomain>,
    cases: HashMap<String, HashMap<String, Case>>,
    predicates: HashMap<(String, AspectKey, Option<AspectValue>), Pred>,
}

impl<'a> NativeCompiler<'a> {
    fn new(
        problem: &'a DelisProg,
        program: SemanticProgram,
        sig: TermSignature,
    ) -> Result<Self, String> {
        let models = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DelisCmd::DefineModel(model) => Some((model.name.clone(), model)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let func_sigs = problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DelisCmd::DeclareFunc(sig) => Some((sig.name.clone(), sig.clone())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let spaces = program
            .finite_spaces
            .iter()
            .map(|space| (space.name.clone(), space.clone()))
            .collect::<HashMap<_, _>>();
        let specs = program
            .semantics
            .iter()
            .map(|spec| (spec.relation.clone(), spec.clone()))
            .collect::<HashMap<_, _>>();
        let domains_by_relation = program
            .domains
            .iter()
            .map(|domain| (domain.relation.clone(), domain.clone()))
            .collect::<HashMap<_, _>>();
        let domain_by_sort = program
            .domains
            .iter()
            .map(|domain| (domain.term_sort.clone(), domain.clone()))
            .collect::<HashMap<_, _>>();
        let mut cases = HashMap::new();
        for spec in &program.semantics {
            cases.insert(spec.relation.clone(), parse_cases(&spec.body)?);
        }
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
            models,
            func_sigs,
            spaces,
            specs,
            domains_by_relation,
            domain_by_sort,
            cases,
            predicates,
        })
    }

    fn compile(&self) -> Result<HornProg, String> {
        let mut out = HornProg::default();
        out.decls.push(HornDecl::SetLogic("HORN".to_string()));
        out.decls.push(HornDecl::DeclareDatatypes(
            vec![SortDecl {
                symbol: "Term".to_string(),
                arity: 0,
            }],
            vec![self.sig.datatype_constructors()],
        ));
        out.decls
            .push(HornDecl::DeclareVar("r".to_string(), term_sort()));
        for var in self.all_case_vars() {
            out.decls.push(HornDecl::DeclareVar(var, term_sort()));
        }

        for domain in &self.program.domains {
            for bit in &domain.bits {
                let state = domain_state_name(domain, &bit.aspect, bit.value.as_ref());
                let pred = self.pred_for(&domain.relation, &bit.aspect, bit.value.as_ref())?;
                for (name, polarity) in [
                    (pred_name(&pred, false), PredicatePolarity::Positive),
                    (pred_name(&pred, true), PredicatePolarity::Negative),
                ] {
                    out.decls
                        .push(HornDecl::DeclareRel(name.clone(), vec![term_sort()]));
                    out.register_predicate(name, state.clone(), polarity);
                }
            }
        }
        out.decls
            .push(HornDecl::DeclareRel("Goal".to_string(), vec![term_sort()]));

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
        let domain = self.domain_for_sort(&cons.result_sort)?;
        let spec = self.spec_for_relation(&domain.relation)?;
        let case = self
            .cases
            .get(&spec.relation)
            .and_then(|cases| cases.get(&cons.name))
            .ok_or_else(|| format!("missing semantic case for constructor {}", cons.name))?;
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
        Ok(())
    }

    fn emit_goal(&self, out: &mut HornProg) -> Result<(), String> {
        let mut body_atoms = Vec::new();
        for constraint in &self.program.constraints {
            let domain = self.domain_for_relation(&constraint.relation)?;
            if domain.kind != SemanticKind::Bool {
                return Err(format!(
                    "top-level constraint {} must target Bool semantics",
                    constraint.relation
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
            "=" if items.len() == 3 => self.eval_finite_terms_equal(items[1], items[2], env),
            "forall" | "exists" if items.len() == 3 => {
                self.eval_quantifier(op, items[1], items[2], env)
            }
            relation if self.domains_by_relation.contains_key(relation) => {
                self.eval_semantic_bool_call(relation, &items[1..], env)
            }
            func if self.func_sigs.contains_key(func) => {
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
            "=" if items.len() == 3 => self.eval_finite_terms_not_equal(items[1], items[2], env),
            "forall" if items.len() == 3 => {
                self.eval_quantifier("exists-neg", items[1], items[2], env)
            }
            "exists" if items.len() == 3 => {
                self.eval_quantifier("forall-neg", items[1], items[2], env)
            }
            relation if self.domains_by_relation.contains_key(relation) => {
                self.eval_semantic_bool_call_neg(relation, &items[1..], env)
            }
            func if self.func_sigs.contains_key(func) => {
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
        let (model, sort) = self.parse_domain_expr(domain_expr, env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let values = get_domain_for_sort(model_ref, &sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let conjunctive = op == "forall" || op == "forall-neg";
        let neg_body = op == "forall-neg" || op == "exists-neg";
        let mut out = if conjunctive { dnf_true() } else { dnf_false() };
        for value in values {
            let mut next = env.clone();
            next.values.insert(
                var.to_string(),
                AspectValue::Univ {
                    sort: sort.clone(),
                    name: value,
                },
            );
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

    fn eval_semantic_bool_call(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<PredDnf, String> {
        let domain = self.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool {
            return Err(format!("{relation} does not return Bool"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let term = self.child_term(args[0], env)?;
        let aspect = self.eval_aspect(args[1], relation, env)?;
        let pred = self.pred_for(relation, &aspect, None)?;
        Ok(vec![vec![PredAtom::new(pred, term)]])
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
        let term = self.child_term(args[0], env)?;
        let aspect = self.eval_aspect(args[1], relation, env)?;
        let pred = self.pred_for(relation, &aspect, None)?.dual();
        Ok(vec![vec![PredAtom::new(pred, term)]])
    }

    fn eval_model_bool_call(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
        expected: bool,
    ) -> Result<PredDnf, String> {
        let sig = self.func_sigs.get(func).unwrap();
        if sig.ret_sort.to_string() != "Bool" {
            return Err(format!("function {func} does not return Bool"));
        }
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let domains = self.argument_domains(sig, model_ref)?;
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
            .infer_finite_sort(lhs, env)
            .or_else(|| self.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for equality (= {lhs} {rhs})"))?;
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
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
            .infer_finite_sort(lhs, env)
            .or_else(|| self.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for inequality (= {lhs} {rhs})"))?;
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
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
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(if value == target {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            let model = self.current_model(env)?;
            let model_ref = self
                .models
                .get(&model)
                .ok_or_else(|| format!("unknown model {model}"))?;
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
        if self.domains_by_relation.contains_key(op) {
            return self.eval_semantic_finite_call_eq(op, &items[1..], target, env);
        }
        if self.func_sigs.contains_key(op) {
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
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(if value != target {
                    dnf_true()
                } else {
                    dnf_false()
                });
            }
            let model = self.current_model(env)?;
            let model_ref = self
                .models
                .get(&model)
                .ok_or_else(|| format!("unknown model {model}"))?;
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
        if self.domains_by_relation.contains_key(op) {
            let term = self.child_term(items[1], env)?;
            let aspect = self.eval_aspect(items[2], op, env)?;
            let pred = self.pred_for(op, &aspect, Some(target))?.dual();
            return Ok(vec![vec![PredAtom::new(pred, term)]]);
        }
        if self.func_sigs.contains_key(op) {
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
        let term = self.child_term(args[0], env)?;
        let aspect = self.eval_aspect(args[1], relation, env)?;
        let pred = self.pred_for(relation, &aspect, Some(target))?;
        Ok(vec![vec![PredAtom::new(pred, term)]])
    }

    fn eval_model_finite_call_eq(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredDnf, String> {
        let sig = self.func_sigs.get(func).unwrap();
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let domains = self.argument_domains(sig, model_ref)?;
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
        let sig = self.func_sigs.get(func).unwrap();
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let domains = self.argument_domains(sig, model_ref)?;
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

    fn argument_domains(
        &self,
        sig: &FuncSig,
        model: &Model,
    ) -> Result<Vec<Vec<AspectValue>>, String> {
        sig.arg_sorts
            .iter()
            .map(|sort| {
                let sort = sort.to_string();
                get_domain_for_sort(model, &sort)
                    .ok_or_else(|| format!("model {} has no universe for sort {sort}", model.name))
                    .map(|values| {
                        values
                            .into_iter()
                            .map(|name| AspectValue::Univ {
                                sort: sort.clone(),
                                name,
                            })
                            .collect()
                    })
            })
            .collect()
    }

    fn child_term(&self, expr: &Value, env: &Env) -> Result<HornTerm, String> {
        let child = expr
            .as_symbol()
            .ok_or_else(|| format!("semantic call term must be a child variable: {expr}"))?;
        env.children
            .get(child)
            .map(|binding| binding.term.clone())
            .ok_or_else(|| format!("unknown semantic child variable {child}"))
    }

    fn eval_aspect(&self, expr: &Value, relation: &str, env: &Env) -> Result<AspectKey, String> {
        if let Some(symbol) = expr.as_symbol() {
            let aspect = env
                .aspect
                .get(symbol)
                .ok_or_else(|| format!("unknown aspect variable {symbol}"))?;
            let mut aspect = aspect.clone();
            aspect.relation = relation.to_string();
            return Ok(aspect);
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("aspect expression must be a list: {expr}"))?;
        let cons = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("aspect constructor must be a symbol: {expr}"))?;
        let space = self
            .spaces
            .get(cons)
            .ok_or_else(|| format!("unknown aspect constructor {cons}"))?;
        let mut model = None;
        let mut fields = Vec::new();
        for (field, arg) in space.fields.iter().zip(items.iter().skip(1)) {
            match &field.domain {
                FieldDomain::Sort(sort) if sort == "Model" => {
                    model = Some(self.eval_model_name(arg, env)?);
                }
                FieldDomain::Sort(sort) => {
                    let value = self.eval_value(arg, env)?;
                    match &value {
                        AspectValue::Univ { sort: actual, .. } if actual == sort => {}
                        AspectValue::Int(_) if sort == "Int" => {}
                        _ => {
                            return Err(format!(
                                "aspect field {} expected sort {}, got {}",
                                field.name,
                                sort,
                                value.display()
                            ));
                        }
                    }
                    fields.push((field.name.clone(), value));
                }
                FieldDomain::Enum(values) => {
                    let value = self.eval_value(arg, env)?;
                    if !values.contains(&value.display()) {
                        return Err(format!(
                            "aspect field {} expected one of ({})",
                            field.name,
                            values.join(" ")
                        ));
                    }
                    fields.push((field.name.clone(), value));
                }
            }
        }
        Ok(AspectKey {
            relation: relation.to_string(),
            model: model.ok_or_else(|| format!("aspect {cons} has no model field"))?,
            fields,
        })
    }

    fn eval_value(&self, expr: &Value, env: &Env) -> Result<AspectValue, String> {
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(value.clone());
            }
            return Ok(AspectValue::Symbol(symbol.to_string()));
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("value expression must be a symbol or list: {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("value expression operator must be a symbol: {expr}"))?;
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                let aspect = env
                    .aspect
                    .get(aspect_arg)
                    .ok_or_else(|| format!("unknown aspect variable {aspect_arg}"))?;
                if op == "model" {
                    return Ok(AspectValue::Model(aspect.model.clone()));
                }
                if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                    return Ok(value.clone());
                }
            }
        }
        Err(format!("unsupported value expression: {expr}"))
    }

    fn eval_model_name(&self, expr: &Value, env: &Env) -> Result<String, String> {
        match self.eval_value(expr, env)? {
            AspectValue::Model(model) | AspectValue::Symbol(model) => Ok(model),
            other => Err(format!("expected model value, got {}", other.display())),
        }
    }

    fn parse_domain_expr(&self, expr: &Value, env: &Env) -> Result<(String, String), String> {
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("domain expression must be a list: {expr}"))?;
        match items.as_slice() {
            [domain, sort_app] if domain.as_symbol() == Some("domain") => {
                let sort_items = sort_app.to_ref_vec().ok_or_else(|| {
                    format!("domain argument must be a sort application: {sort_app}")
                })?;
                let [sort, model_expr] = sort_items.as_slice() else {
                    return Err(format!(
                        "domain argument must have shape (Sort model): {sort_app}"
                    ));
                };
                let sort = sort
                    .as_symbol()
                    .ok_or_else(|| format!("domain sort must be a symbol: {sort}"))?
                    .to_string();
                let model = self.eval_model_name(model_expr, env)?;
                Ok((model, sort))
            }
            _ => Err(format!("unsupported domain expression: {expr}")),
        }
    }

    fn current_model(&self, env: &Env) -> Result<String, String> {
        env.aspect
            .get("asp")
            .map(|aspect| aspect.model.clone())
            .ok_or_else(|| "no current aspect in semantic environment".to_string())
    }

    fn infer_finite_sort(&self, expr: &Value, env: &Env) -> Option<String> {
        if let Some(symbol) = expr.as_symbol() {
            if let Some(AspectValue::Univ { sort, .. }) = env.values.get(symbol) {
                return Some(sort.clone());
            }
            return self
                .func_sigs
                .get(symbol)
                .map(|sig| sig.ret_sort.to_string());
        }
        let items = expr.to_ref_vec()?;
        let op = items.first()?.as_symbol()?;
        if let Some(domain) = self.domains_by_relation.get(op) {
            if let SemanticKind::FiniteValue { sort } = &domain.kind {
                return Some(sort.clone());
            }
        }
        self.func_sigs.get(op).map(|sig| sig.ret_sort.to_string())
    }

    fn all_case_vars(&self) -> Vec<String> {
        let mut vars = BTreeSet::new();
        for cases in self.cases.values() {
            for case in cases.values() {
                vars.extend(case.vars.iter().cloned());
            }
        }
        vars.into_iter().collect()
    }

    fn domain_for_sort(&self, sort: &str) -> Result<&SemanticDomain, String> {
        self.domain_by_sort
            .get(sort)
            .ok_or_else(|| format!("no semantic domain for term sort {sort}"))
    }

    fn domain_for_relation(&self, relation: &str) -> Result<&SemanticDomain, String> {
        self.domains_by_relation
            .get(relation)
            .ok_or_else(|| format!("no semantic domain for relation {relation}"))
    }

    fn spec_for_relation(&self, relation: &str) -> Result<&SemanticSpec, String> {
        self.specs
            .get(relation)
            .ok_or_else(|| format!("no semantic spec for relation {relation}"))
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
    sanitize_pred_part(&state)
}

fn sanitize_pred_part(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn pred_name(pred: &Pred, neg: bool) -> String {
    if neg {
        pred.dual().name()
    } else {
        pred.name()
    }
}
