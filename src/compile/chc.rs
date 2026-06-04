use crate::{
    backends::smt2::{
        ast::{HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm, PredicatePolarity},
        pred::{
            formula_and, formula_atom, formula_false, formula_or, formula_true, Pred, PredAtom,
            PredFormula,
        },
    },
    compile::{
        context::CompileContext,
        eval_ctx::{EnvView, SemanticContext},
        semantics::{interped_bool, interped_value_matches},
        syntax::{state_name, TermConstructor, TermSignature},
        types::{
            AspectKey, AspectValue, FieldDecl, FieldDomain, SemanticDomain, SemanticKind,
            SemanticProgram,
        },
    },
    parser::DelisProg,
};
use sygus_parser::SExpr as Value;
use rustc_hash::FxHashMap as HashMap;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use sygus_parser::Sort;

#[derive(Clone)]
struct ChildBinding {
    term: HornTerm,
    sort: String,
}

#[derive(Clone)]
struct TypedTerm {
    term: HornTerm,
    sort: String,
}

struct BoolPair {
    pos: PredFormula,
    neg: PredFormula,
}

#[derive(Default, Clone, Debug)]
struct QuantifierTiming {
    calls: usize,
    iterations: usize,
    domain_time: Duration,
    clone_time: Duration,
    body_time: Duration,
    combine_time: Duration,
    total_time: Duration,
}

#[derive(Default, Clone, Debug)]
struct QuantifierProfile {
    forall_pos: QuantifierTiming,
    forall_neg: QuantifierTiming,
    exists_pos: QuantifierTiming,
    exists_neg: QuantifierTiming,
}

impl QuantifierProfile {
    fn bucket_mut(&mut self, forall: bool, positive: bool) -> &mut QuantifierTiming {
        match (forall, positive) {
            (true, true) => &mut self.forall_pos,
            (true, false) => &mut self.forall_neg,
            (false, true) => &mut self.exists_pos,
            (false, false) => &mut self.exists_neg,
        }
    }
}

fn log_quantifier_bucket(cons: &str, label: &str, timing: &QuantifierTiming) {
    if timing.calls == 0 {
        return;
    }
    log::debug!(
        "CHC quantifier {} {}: calls={}, iterations={}, domain={:?}, clone={:?}, body={:?}, combine={:?}, total={:?}",
        cons,
        label,
        timing.calls,
        timing.iterations,
        timing.domain_time,
        timing.clone_time,
        timing.body_time,
        timing.combine_time,
        timing.total_time,
    );
}

#[derive(Default, Clone, Debug)]
struct StageTiming {
    calls: usize,
    total_time: Duration,
}

#[derive(Default, Clone, Debug)]
struct LoweringProfile {
    eval_bool_pair: StageTiming,
    semantic_bool_call: StageTiming,
    semantic_finite_call: StageTiming,
    aspect_variants: StageTiming,
    child_terms: StageTiming,
    pred_for: StageTiming,
}

fn log_stage_timing(cons: &str, label: &str, timing: &StageTiming) {
    if timing.calls == 0 {
        return;
    }
    log::debug!(
        "CHC stage {} {}: calls={}, total={:?}, avg={:?}",
        cons,
        label,
        timing.calls,
        timing.total_time,
        timing.total_time / timing.calls as u32,
    );
}

#[derive(Default, Clone)]
struct Env {
    aspect: Rc<HashMap<String, AspectKey>>,
    values: HashMap<String, AspectValue>,
    children: Rc<HashMap<String, ChildBinding>>,
    current_sort: Option<Rc<String>>,
    current_var: Option<Rc<String>>,
    current_term: Option<Rc<HornTerm>>,
}

impl EnvView for Env {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        self.aspect.as_ref()
    }

    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
}

impl<'a> CompileContext<'a> {
    pub fn compile_chc(self) -> Result<HornProg, String> {
        ChcCompiler::new(self.problem, self.semantic_program, self.sig, self.semantic_cases)?
            .compile()
    }
}

struct ChcCompiler<'a> {
    program: SemanticProgram,
    sig: TermSignature,
    ctx: SemanticContext<'a>,
    relation_sorts: HashMap<String, Vec<String>>,
    predicates: HashMap<(String, String, AspectKey, Option<AspectValue>), Pred>,
    next_helper_id: usize,
    domain_values_cache: RefCell<HashMap<(String, String), Vec<AspectValue>>>,
    quantifier_profile: RefCell<QuantifierProfile>,
    lowering_profile: RefCell<LoweringProfile>,
}

impl<'a> ChcCompiler<'a> {
    fn new(
        problem: &'a DelisProg,
        program: SemanticProgram,
        sig: TermSignature,
        semantic_cases: HashMap<String, HashMap<String, crate::compile::semantics::Case>>,
    ) -> Result<Self, String> {
        let ctx = SemanticContext::with_cases(problem, &program, &semantic_cases)?;
        let constructor_sorts = sig
            .constructors
            .iter()
            .map(|cons| (cons.name.clone(), cons.result_sort.clone()))
            .collect::<HashMap<_, _>>();
        let mut relation_sorts = HashMap::default();
        for spec in &program.semantics {
            let mut sorts = vec![spec.term_sort.clone()];
            let cases = ctx
                .cases()
                .get(&spec.relation)
                .ok_or_else(|| format!("missing parsed cases for relation {}", spec.relation))?;
            for cons in cases.keys() {
                let sort = constructor_sorts
                    .get(cons)
                    .ok_or_else(|| format!("unknown constructor {} in {}", cons, spec.relation))?;
                if !sorts.contains(sort) {
                    sorts.push(sort.clone());
                }
            }
            relation_sorts.insert(spec.relation.clone(), sorts);
        }
        Ok(Self {
            program,
            sig,
            ctx,
            relation_sorts,
            predicates: HashMap::default(),
            next_helper_id: 0,
            domain_values_cache: RefCell::new(HashMap::default()),
            quantifier_profile: RefCell::new(QuantifierProfile::default()),
            lowering_profile: RefCell::new(LoweringProfile::default()),
        }
        .init_predicates())
    }

    fn init_predicates(mut self) -> Self {
        for domain in &self.program.domains {
            for bit in &domain.bits {
                let state = domain_state_name(domain, &bit.aspect, bit.value.as_ref());
                for sort in self
                    .relation_sorts
                    .get(&domain.relation)
                    .into_iter()
                    .flatten()
                {
                    self.predicates.insert(
                        (
                            domain.relation.clone(),
                            sort.clone(),
                            bit.aspect.clone(),
                            bit.value.clone(),
                        ),
                        Pred::new(
                            format!("q{}.{}", state, sort),
                            format!("nq{}.{}", state, sort),
                        ),
                    );
                }
            }
        }
        self
    }

    fn compile(mut self) -> Result<HornProg, String> {
        let mut out = HornProg::default();
        out.decls.push(HornDecl::SetLogic("HORN".to_string()));
        let (sorts, decls) = self.sig.datatype_decls();
        out.decls.push(HornDecl::DeclareDatatypes(sorts, decls));

        for domain in &self.program.domains {
            for bit in &domain.bits {
                let state = domain_state_name(domain, &bit.aspect, bit.value.as_ref());
                for sort in self.relation_term_sorts(&domain.relation)? {
                    let pred =
                        self.pred_for(&domain.relation, sort, &bit.aspect, bit.value.as_ref())?;
                    for (name, polarity) in [
                        (pred_name(&pred, false), PredicatePolarity::Positive),
                        (pred_name(&pred, true), PredicatePolarity::Negative),
                    ] {
                        let declared_sort = Sort::from_name(sort);
                        out.decls
                            .push(HornDecl::DeclareRel(name.clone(), vec![declared_sort]));
                        out.register_predicate(name, state.clone(), polarity, sort.clone());
                    }
                }
            }
        }
        out.decls.push(HornDecl::DeclareRel(
            "Goal".to_string(),
            vec![Sort::from_name(self.sig.start_sort())],
        ));

        let constructors = self.sig.constructors.clone();
        for cons in &constructors {
            self.emit_constructor_rules(&mut out, cons)?;
        }
        self.emit_goal(&mut out)?;
        Ok(out)
    }

    fn emit_constructor_rules(
        &mut self,
        out: &mut HornProg,
        cons: &TermConstructor,
    ) -> Result<(), String> {
        let start = Instant::now();
        *self.quantifier_profile.borrow_mut() = QuantifierProfile::default();
        *self.lowering_profile.borrow_mut() = LoweringProfile::default();
        let mut matched = false;
        let domains = self.program.domains.clone();
        for domain in &domains {
            let relation = domain.relation.clone();
            let spec = self.ctx.spec_for_relation(&relation)?.clone();
            let Some(case) = self
                .ctx
                .cases()
                .get(&spec.relation)
                .and_then(|cases| cases.get(&cons.name))
                .cloned()
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
            env.current_sort = Some(Rc::new(cons.result_sort.clone()));
            env.current_var = Some(Rc::new(case.self_var.clone()));
            env.current_term = Some(Rc::new(head_term.clone()));
            for (idx, var) in case.vars.iter().enumerate() {
                Rc::make_mut(&mut env.children).insert(
                    var.clone(),
                    ChildBinding {
                        term: HornTerm::var(var),
                        sort: cons.arg_sorts[idx].clone(),
                    },
                );
            }
            for bit in &domain.bits {
                Rc::make_mut(&mut env.aspect).insert(case.aspect_var.clone(), bit.aspect.clone());
                let (formula, neg_formula) = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) => {
                        let pair = self.eval_bool_pair(&case.body, &env)?;
                        (pair.pos, pair.neg)
                    }
                    (SemanticKind::FiniteValue { .. }, Some(value)) => (
                        self.eval_finite_eq(&case.body, value, &env)?,
                        self.eval_finite_neq(&case.body, value, &env)?,
                    ),
                    _ => {
                        return Err(format!(
                            "malformed semantic domain bit {} for {}",
                            bit.bit, domain.relation
                        ));
                    }
                };
                let head = PredAtom::new(
                    self.pred_for(
                        &domain.relation,
                        &cons.result_sort,
                        &bit.aspect,
                        bit.value.as_ref(),
                    )?,
                    head_term.clone(),
                );
                self.emit_formula_rules(out, head.clone(), formula, &cons.result_sort, &head_term)?;
                self.emit_formula_rules(
                    out,
                    head.dual(),
                    neg_formula,
                    &cons.result_sort,
                    &head_term,
                )?;
            }
        }
        if !matched {
            return Err(format!(
                "missing semantic case for constructor {}",
                cons.name
            ));
        }
        let profile = self.quantifier_profile.borrow().clone();
        log_quantifier_bucket(&cons.name, "forall_pos", &profile.forall_pos);
        log_quantifier_bucket(&cons.name, "forall_neg", &profile.forall_neg);
        log_quantifier_bucket(&cons.name, "exists_pos", &profile.exists_pos);
        log_quantifier_bucket(&cons.name, "exists_neg", &profile.exists_neg);
        let lowering = self.lowering_profile.borrow().clone();
        log_stage_timing(&cons.name, "eval_bool_pair", &lowering.eval_bool_pair);
        log_stage_timing(&cons.name, "semantic_bool_call", &lowering.semantic_bool_call);
        log_stage_timing(
            &cons.name,
            "semantic_finite_call",
            &lowering.semantic_finite_call,
        );
        log_stage_timing(&cons.name, "aspect_variants", &lowering.aspect_variants);
        log_stage_timing(&cons.name, "child_terms", &lowering.child_terms);
        log_stage_timing(&cons.name, "pred_for", &lowering.pred_for);
        log::debug!(
            "CHC constructor {} compiled in {:?}",
            cons.name,
            start.elapsed(),
        );
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
            if domain.term_sort != self.sig.start_sort() {
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
            let pred = self.pred_for(&constraint.relation, self.sig.start_sort(), &aspect, None)?;
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

    fn emit_formula_rules(
        &mut self,
        out: &mut HornProg,
        head: PredAtom,
        formula: PredFormula,
        sort: &str,
        current_term: &HornTerm,
    ) -> Result<(), String> {
        match formula {
            PredFormula::True => {
                out.rules.push(HornRule {
                    body_atoms: Vec::new(),
                    head: head.to_horn(),
                });
            }
            PredFormula::False => {}
            PredFormula::Atom(atom) => {
                out.rules.push(HornRule {
                    body_atoms: vec![atom.to_horn()],
                    head: head.to_horn(),
                });
            }
            PredFormula::Or(parts) => {
                for part in parts {
                    self.emit_formula_rules(out, head.clone(), part, sort, current_term)?;
                }
            }
            PredFormula::And(parts) => {
                let mut body_atoms = Vec::new();
                for part in parts {
                    match part {
                        PredFormula::True => {}
                        PredFormula::False => return Ok(()),
                        PredFormula::Atom(atom) => body_atoms.push(atom.to_horn()),
                        other => body_atoms.push(self.emit_helper_atom(
                            out,
                            other,
                            sort,
                            current_term,
                        )?),
                    }
                }
                out.rules.push(HornRule {
                    body_atoms,
                    head: head.to_horn(),
                });
            }
        }
        Ok(())
    }

    fn emit_helper_atom(
        &mut self,
        out: &mut HornProg,
        formula: PredFormula,
        sort: &str,
        current_term: &HornTerm,
    ) -> Result<HornAtom, String> {
        let helper_name = format!("__chc.{}.{}", sort, self.next_helper_id);
        self.next_helper_id += 1;
        let helper_pred = Pred::new(helper_name.clone(), format!("n{helper_name}"));
        out.decls.push(HornDecl::DeclareRel(
            helper_name.clone(),
            vec![Sort::from_name(sort)],
        ));
        out.register_predicate(
            helper_name.clone(),
            helper_name.clone(),
            PredicatePolarity::Positive,
            sort.to_string(),
        );
        let helper_head = PredAtom::new(helper_pred, current_term.clone());
        self.emit_formula_rules(out, helper_head.clone(), formula, sort, current_term)?;
        Ok(helper_head.to_horn())
    }

    fn eval_bool(&self, expr: &Value, env: &Env) -> Result<PredFormula, String> {
        Ok(self.eval_bool_pair(expr, env)?.pos)
    }

    fn eval_bool_neg(&self, expr: &Value, env: &Env) -> Result<PredFormula, String> {
        Ok(self.eval_bool_pair(expr, env)?.neg)
    }

    fn eval_bool_pair(&self, expr: &Value, env: &Env) -> Result<BoolPair, String> {
        let start = Instant::now();
        let result = (|| {
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(BoolPair {
                    pos: formula_true(),
                    neg: formula_false(),
                }),
                "false" => Ok(BoolPair {
                    pos: formula_false(),
                    neg: formula_true(),
                }),
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
            "not" if items.len() == 2 => {
                let pair = self.eval_bool_pair(items[1], env)?;
                Ok(BoolPair {
                    pos: pair.neg,
                    neg: pair.pos,
                })
            }
            "and" => {
                let mut pos = formula_true();
                let mut neg = formula_false();
                for item in &items[1..] {
                    let pair = self.eval_bool_pair(item, env)?;
                    pos = formula_and(pos, pair.pos);
                    neg = formula_or(neg, pair.neg);
                    if matches!(pos, PredFormula::False) && matches!(neg, PredFormula::True) {
                        break;
                    }
                }
                Ok(BoolPair { pos, neg })
            }
            "or" => {
                let mut pos = formula_false();
                let mut neg = formula_true();
                for item in &items[1..] {
                    let pair = self.eval_bool_pair(item, env)?;
                    pos = formula_or(pos, pair.pos);
                    neg = formula_and(neg, pair.neg);
                    if matches!(pos, PredFormula::True) && matches!(neg, PredFormula::False) {
                        break;
                    }
                }
                Ok(BoolPair { pos, neg })
            }
            "ite" if items.len() == 4 => {
                let cond = self.eval_bool_pair(items[1], env)?;
                let then_pair = self.eval_bool_pair(items[2], env)?;
                let else_pair = self.eval_bool_pair(items[3], env)?;
                Ok(BoolPair {
                    pos: formula_or(
                        formula_and(cond.pos.clone(), then_pair.pos),
                        formula_and(cond.neg.clone(), else_pair.pos),
                    ),
                    neg: formula_or(
                        formula_and(cond.pos, then_pair.neg),
                        formula_and(cond.neg, else_pair.neg),
                    ),
                })
            }
            "=" if items.len() == 3 => Ok(BoolPair {
                pos: self.eval_equality(items[1], items[2], env, true)?,
                neg: self.eval_equality(items[1], items[2], env, false)?,
            }),
            "<" if items.len() == 3 => Ok(BoolPair {
                pos: self.eval_int_compare(items[1], items[2], env, IntCmp::Lt)?,
                neg: self.eval_int_compare(items[1], items[2], env, IntCmp::Ge)?,
            }),
            "<=" if items.len() == 3 => Ok(BoolPair {
                pos: self.eval_int_compare(items[1], items[2], env, IntCmp::Le)?,
                neg: self.eval_int_compare(items[1], items[2], env, IntCmp::Gt)?,
            }),
            "forall" | "exists" if items.len() == 3 => {
                self.eval_quantifier_pair(op == "forall", items[1], items[2], env)
            }
            relation if self.ctx.has_relation(relation) => {
                self.eval_semantic_bool_call_pair(relation, &items[1..], env)
            }
            func if self.ctx.has_func(func) => {
                self.eval_model_bool_call_pair(func, &items[1..], env)
            }
            _ => Err(format!("unsupported Bool expression: {expr}")),
        }
        })();
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.eval_bool_pair.calls += 1;
        profile.eval_bool_pair.total_time += elapsed;
        result
    }

    fn eval_quantifier_pair(
        &self,
        forall: bool,
        bindings: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<BoolPair, String> {
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
        let total_start = Instant::now();
        let domain_start = Instant::now();
        let values = self.quantifier_values(domain_expr, env)?;
        let domain_time = domain_start.elapsed();
        let mut pos = if forall {
            formula_true()
        } else {
            formula_false()
        };
        let mut neg = if forall {
            formula_false()
        } else {
            formula_true()
        };
        let mut clone_time = Duration::ZERO;
        let mut body_time = Duration::ZERO;
        let mut combine_time = Duration::ZERO;
        let mut iterations = 0usize;
        for value in values {
            iterations += 1;
            let step_start = Instant::now();
            let mut next = env.clone();
            next.values.insert(var.to_string(), value);
            clone_time += step_start.elapsed();
            let step_start = Instant::now();
            let pair = self.eval_bool_pair(body, &next)?;
            body_time += step_start.elapsed();
            let step_start = Instant::now();
            if forall {
                pos = formula_and(pos, pair.pos);
                neg = formula_or(neg, pair.neg);
            } else {
                pos = formula_or(pos, pair.pos);
                neg = formula_and(neg, pair.neg);
            }
            combine_time += step_start.elapsed();
            if forall && matches!(pos, PredFormula::False) && matches!(neg, PredFormula::True) {
                break;
            }
            if !forall && matches!(pos, PredFormula::True) && matches!(neg, PredFormula::False) {
                break;
            }
        }
        let mut profile = self.quantifier_profile.borrow_mut();
        for positive in [true, false] {
            let bucket = profile.bucket_mut(forall, positive);
            bucket.calls += 1;
            bucket.iterations += iterations;
            bucket.domain_time += domain_time;
            bucket.clone_time += clone_time;
            bucket.body_time += body_time;
            bucket.combine_time += combine_time;
            bucket.total_time += total_start.elapsed();
        }
        Ok(BoolPair { pos, neg })
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
                let (model, sort) = self.ctx.parse_univ_expr(domain_expr, env)?;
                self.cached_domain_values(&model, &sort)
            }
        }
    }

    fn eval_equality(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
        expected: bool,
    ) -> Result<PredFormula, String> {
        if self.ctx.infer_finite_sort(lhs, env).is_some()
            || self.ctx.infer_finite_sort(rhs, env).is_some()
        {
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
                    Ok(bool_formula((lhs_value == rhs_value) == expected))
                }
                _ => Ok(bool_formula((lhs_value == rhs_value) == expected)),
            }
        } else {
            let lhs = self.eval_int(lhs, env)?;
            let rhs = self.eval_int(rhs, env)?;
            Ok(bool_formula((lhs == rhs) == expected))
        }
    }

    fn eval_int_compare(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
        cmp: IntCmp,
    ) -> Result<PredFormula, String> {
        let lhs = self.eval_int(lhs, env)?;
        let rhs = self.eval_int(rhs, env)?;
        let value = match cmp {
            IntCmp::Lt => lhs < rhs,
            IntCmp::Le => lhs <= rhs,
            IntCmp::Gt => lhs > rhs,
            IntCmp::Ge => lhs >= rhs,
        };
        Ok(bool_formula(value))
    }

    fn eval_semantic_bool_call_pair(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolPair, String> {
        let start = Instant::now();
        let domain = self.ctx.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool {
            return Err(format!("{relation} does not return Bool"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let terms = self.child_terms(args[0], env)?;
        let allowed_sorts = self.relation_term_sorts(relation)?;
        let mut pos = formula_false();
        let mut neg = formula_false();
        for (guard, aspect) in self.aspect_variants(args[1], relation, env)? {
            for term in &terms {
                if !allowed_sorts.iter().any(|sort| sort == &term.sort) {
                    continue;
                }
                let pred = self.pred_for(relation, &term.sort, &aspect, None)?;
                pos = formula_or(
                    pos,
                    formula_and(
                        guard.clone(),
                        formula_atom(PredAtom::new(pred.clone(), term.term.clone())),
                    ),
                );
                neg = formula_or(
                    neg,
                    formula_and(
                        guard.clone(),
                        formula_atom(PredAtom::new(pred.dual(), term.term.clone())),
                    ),
                );
            }
        }
        let result = Ok(BoolPair { pos, neg });
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.semantic_bool_call.calls += 1;
        profile.semantic_bool_call.total_time += elapsed;
        result
    }

    fn eval_model_bool_call_pair(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolPair, String> {
        let sig = self
            .ctx
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        if sig.ret_sort.to_string() != "Bool" {
            return Err(format!("function {func} does not return Bool"));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        if let Some(arg_names) = self.ground_arg_names(sig, args, env) {
            let value = self.ctx.eval_model_symbols(model_ref, func, &arg_names);
            let matched = value.as_ref().is_some_and(interped_bool);
            return Ok(BoolPair {
                pos: bool_formula(matched),
                neg: bool_formula(!matched),
            });
        }
        Ok(BoolPair {
            pos: self.eval_model_call_matches(func, args, env, |value| interped_bool(value))?,
            neg: self.eval_model_call_matches(func, args, env, |value| !interped_bool(value))?,
        })
    }

    fn eval_model_call_matches<F>(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
        mut keep: F,
    ) -> Result<PredFormula, String>
    where
        F: FnMut(&crate::lang::model::InterpedValue) -> bool,
    {
        let mut out = formula_false();
        for tuple in self
            .ctx
            .matching_model_call_tuples(func, args, env, |value| keep(value))?
        {
            let mut conj = formula_true();
            for (arg_expr, value) in args.iter().zip(tuple.iter()) {
                conj = formula_and(conj, self.eval_finite_eq(arg_expr, value, env)?);
            }
            out = formula_or(out, conj);
        }
        Ok(out)
    }

    fn eval_finite_terms_equal(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
    ) -> Result<PredFormula, String> {
        let sort = self
            .ctx
            .infer_finite_sort(lhs, env)
            .or_else(|| self.ctx.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for equality (= {lhs} {rhs})"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = model_ref.univ_for_sort(&sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let sort_arc: Arc<str> = Arc::from(sort.as_str());
        let mut out = formula_false();
        for name in values {
            let value = AspectValue::Univ {
                sort: sort_arc.clone(),
                name: Arc::from(name),
            };
            out = formula_or(
                out,
                formula_and(
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
    ) -> Result<PredFormula, String> {
        let sort = self
            .ctx
            .infer_finite_sort(lhs, env)
            .or_else(|| self.ctx.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for inequality (= {lhs} {rhs})"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = model_ref.univ_for_sort(&sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let sort_arc: Arc<str> = Arc::from(sort.as_str());
        let mut out = formula_false();
        for lhs_name in &values {
            for rhs_name in &values {
                if lhs_name == rhs_name {
                    continue;
                }
                let lhs_value = AspectValue::Univ {
                    sort: sort_arc.clone(),
                    name: Arc::from(lhs_name.as_str()),
                };
                let rhs_value = AspectValue::Univ {
                    sort: sort_arc.clone(),
                    name: Arc::from(rhs_name.as_str()),
                };
                out = formula_or(
                    out,
                    formula_and(
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
    ) -> Result<PredFormula, String> {
        self.eval_finite_match(expr, target, env, true)
    }

    fn eval_finite_neq(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<PredFormula, String> {
        self.eval_finite_match(expr, target, env, false)
    }

    fn eval_finite_match(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
        positive: bool,
    ) -> Result<PredFormula, String> {
        if let Ok(value) = self.ctx.eval_value(expr, env) {
            let matches = self.ctx.value_matches_target(&value, target, env)?;
            return Ok(if matches == positive {
                formula_true()
            } else {
                formula_false()
            });
        }
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(if (*value == *target) == positive {
                    formula_true()
                } else {
                    formula_false()
                });
            }
            let model = self.ctx.current_model(env)?;
            let model_ref = self.ctx.model(&model)?;
            if let Some(value) = self.ctx.eval_model_symbols(model_ref, symbol, &[]) {
                return Ok(if interped_value_matches(&value, target) == positive {
                    formula_true()
                } else {
                    formula_false()
                });
            }
            return Ok(if (symbol == target.display()) == positive {
                formula_true()
            } else {
                formula_false()
            });
        }
        let items = expr
            .to_ref_vec()
            .ok_or_else(|| format!("expected finite expression, got {expr}"))?;
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("finite expression operator must be a symbol: {expr}"))?;
        if op == "ite" && items.len() == 4 {
            return Ok(formula_or(
                formula_and(
                    self.eval_bool(items[1], env)?,
                    self.eval_finite_match(items[2], target, env, positive)?,
                ),
                formula_and(
                    self.eval_bool_neg(items[1], env)?,
                    self.eval_finite_match(items[3], target, env, positive)?,
                ),
            ));
        }
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspect.get(aspect_arg) {
                    if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                        return Ok(if (*value == *target) == positive {
                            formula_true()
                        } else {
                            formula_false()
                        });
                    }
                }
            }
        }
        if self.ctx.has_relation(op) {
            return self.eval_semantic_finite_call(op, &items[1..], target, env, positive);
        }
        if self.ctx.has_func(op) {
            return self.eval_model_finite_call(op, &items[1..], target, env, positive);
        }
        Err(format!("unsupported finite expression: {expr}"))
    }

    fn eval_semantic_finite_call(
        &self,
        relation: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
        positive: bool,
    ) -> Result<PredFormula, String> {
        let start = Instant::now();
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let terms = self.child_terms(args[0], env)?;
        let allowed_sorts = self.relation_term_sorts(relation)?;
        let mut out = formula_false();
        for (guard, aspect) in self.aspect_variants(args[1], relation, env)? {
            for term in &terms {
                if !allowed_sorts.iter().any(|sort| sort == &term.sort) {
                    continue;
                }
                let pred = self.pred_for(relation, &term.sort, &aspect, Some(target))?;
                let pred = if positive { pred } else { pred.dual() };
                out = formula_or(
                    out,
                    formula_and(
                        guard.clone(),
                        formula_atom(PredAtom::new(pred.clone(), term.term.clone())),
                    ),
                );
            }
        }
        let result = Ok(out);
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.semantic_finite_call.calls += 1;
        profile.semantic_finite_call.total_time += elapsed;
        result
    }

    fn eval_model_finite_call(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
        positive: bool,
    ) -> Result<PredFormula, String> {
        let sig = self
            .ctx
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        if let Some(arg_names) = self.ground_arg_names(sig, args, env) {
            let value = self.ctx.eval_model_symbols(model_ref, func, &arg_names);
            let matches = value
                .as_ref()
                .is_some_and(|value| interped_value_matches(value, target));
            return Ok(bool_formula(matches == positive));
        }
        self.eval_model_call_matches(func, args, env, |value| {
            interped_value_matches(value, target) == positive
        })
    }

    fn child_terms(&self, expr: &Value, env: &Env) -> Result<Vec<TypedTerm>, String> {
        let start = Instant::now();
        let result = (|| {
        if let Some(child) = expr.as_symbol() {
            if env.current_var.as_deref().map(String::as_str) == Some(child) {
                let term = env.current_term.clone().ok_or_else(|| {
                    "missing current term for recursive semantic call".to_string()
                })?;
                let sort = env.current_sort.clone().ok_or_else(|| {
                    "missing current sort for recursive semantic call".to_string()
                })?;
                return Ok(vec![TypedTerm {
                    term: term.as_ref().clone(),
                    sort: sort.as_ref().clone(),
                }]);
            }
            let child = env
                .children
                .get(child)
                .cloned()
                .ok_or_else(|| format!("unknown semantic child variable {child}"))?;
            return Ok(vec![TypedTerm {
                term: child.term,
                sort: child.sort,
            }]);
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
                self.sig
                    .parent_contexts(sort, term.as_ref().clone())
                    .into_iter()
                    .map(|term| {
                        Ok(TypedTerm {
                            sort: self.parent_result_sort(&term)?,
                            term,
                        })
                    })
                    .collect()
            }
            _ => Err(format!("unsupported semantic call term navigation: {expr}")),
        }
        })();
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.child_terms.calls += 1;
        profile.child_terms.total_time += elapsed;
        result
    }

    fn aspect_variants(
        &self,
        expr: &Value,
        relation: &str,
        env: &Env,
    ) -> Result<Vec<(PredFormula, AspectKey)>, String> {
        let start = Instant::now();
        let result = (|| {
        if let Some(symbol) = expr.as_symbol() {
            let aspect = env
                .aspect
                .get(symbol)
                .ok_or_else(|| format!("unknown aspect variable {symbol}"))?;
            let mut aspect = aspect.clone();
            aspect.relation = relation.to_string();
            return Ok(vec![(formula_true(), aspect)]);
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

        let mut partials = vec![(formula_true(), None, Vec::new())];
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
                    next.push((formula_and(guard0.clone(), guard1.clone()), model, fields));
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
        })();
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.aspect_variants.calls += 1;
        profile.aspect_variants.total_time += elapsed;
        result
    }

    fn aspect_field_variants(
        &self,
        field: &FieldDecl,
        arg: &Value,
        env: &Env,
    ) -> Result<Vec<(PredFormula, AspectValue)>, String> {
        match &field.domain {
            FieldDomain::Sort(sort) if sort == "Model" => Ok(vec![(
                formula_true(),
                AspectValue::Model(self.ctx.eval_model_name(arg, env)?),
            )]),
            FieldDomain::Sort(sort) => {
                if let Ok(value) = self.ctx.eval_value_for_sort(arg, sort, env) {
                    return Ok(vec![(formula_true(), value)]);
                }
                if sort == "Int" {
                    return Ok(Vec::new());
                }
                self.dynamic_value_variants(sort, arg, env)
            }
            FieldDomain::Enum(values) => {
                let value = self.ctx.eval_value(arg, env)?;
                if values.contains(&value.display()) {
                    Ok(vec![(formula_true(), value)])
                } else {
                    Err(format!(
                        "aspect field {} expected one of ({})",
                        field.name,
                        values.join(" ")
                    ))
                }
            }
        }
    }

    fn dynamic_value_variants(
        &self,
        sort: &str,
        expr: &Value,
        env: &Env,
    ) -> Result<Vec<(PredFormula, AspectValue)>, String> {
        if self.ctx.infer_finite_sort(expr, env).as_deref() != Some(sort) {
            return Err(format!("unsupported value expression: {expr}"));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = model_ref.univ_for_sort(sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let sort_arc: Arc<str> = Arc::from(sort);
        values
            .into_iter()
            .map(|name| {
                let value = AspectValue::Univ {
                    sort: sort_arc.clone(),
                    name: Arc::from(name),
                };
                Ok((self.eval_finite_eq(expr, &value, env)?, value))
            })
            .collect()
    }

    fn eval_int(&self, expr: &Value, env: &Env) -> Result<isize, String> {
        self.ctx.eval_int_expr(expr, env)
    }

    fn pred_for(
        &self,
        relation: &str,
        sort: &str,
        aspect: &AspectKey,
        value: Option<&AspectValue>,
    ) -> Result<Pred, String> {
        let start = Instant::now();
        let result = self.predicates
            .get(&(
                relation.to_string(),
                sort.to_string(),
                aspect.clone(),
                value.cloned(),
            ))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "no predicate for {}:{}({}; {}){}",
                    relation,
                    sort,
                    aspect.model,
                    aspect.display_fields(),
                    value
                        .map(|v| format!(" = {}", v.display()))
                        .unwrap_or_default()
                )
            });
        let elapsed = start.elapsed();
        let mut profile = self.lowering_profile.borrow_mut();
        profile.pred_for.calls += 1;
        profile.pred_for.total_time += elapsed;
        result
    }

    fn relation_term_sorts(&self, relation: &str) -> Result<&[String], String> {
        self.relation_sorts
            .get(relation)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("missing term sorts for relation {relation}"))
    }

    fn ground_arg_names(
        &self,
        sig: &crate::lang::signature::FuncSig,
        args: &[&Value],
        env: &Env,
    ) -> Option<Vec<String>> {
        args.iter()
            .zip(sig.arg_sorts.iter())
            .map(|arg| {
                self.ctx
                    .eval_value_for_sort(arg.0, &arg.1.to_string(), env)
                    .ok()
                    .map(|value| value.display())
            })
            .collect()
    }

    fn cached_domain_values(&self, model: &str, sort: &str) -> Result<Vec<AspectValue>, String> {
        let key = (model.to_string(), sort.to_string());
        if let Some(values) = self.domain_values_cache.borrow().get(&key) {
            return Ok(values.clone());
        }
        let model_ref = self.ctx.model(model)?;
        let sort_arc: Arc<str> = Arc::from(sort);
        let values = model_ref.univ_for_sort(sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?
            .into_iter()
            .map(|name| AspectValue::Univ {
                sort: sort_arc.clone(),
                name: Arc::from(name),
            })
            .collect::<Vec<_>>();
        self.domain_values_cache
            .borrow_mut()
            .insert(key, values.clone());
        Ok(values)
    }

    fn parent_result_sort(&self, term: &HornTerm) -> Result<String, String> {
        let HornTerm::App { fun, .. } = term else {
            return Err(format!(
                "parent context must be a constructor application: {term}"
            ));
        };
        self.sig
            .constructors
            .iter()
            .find(|cons| cons.name == *fun)
            .map(|cons| cons.result_sort.clone())
            .ok_or_else(|| format!("unknown parent constructor {fun}"))
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

fn bool_formula(value: bool) -> PredFormula {
    if value {
        formula_true()
    } else {
        formula_false()
    }
}
