use crate::{
    backends::sygus::optim::{self, BitwiseExpr},
    compile::{
        context::CompileContext,
        expr::sygus_term_to_value,
        native::{EnvView, SemanticContext},
        semantic::{interped_bool, interped_value_matches},
        syntax::{TermConstructor, TermSignature},
        types::{AspectKey, AspectValue, SemanticKind},
    },
    lang::eval::eval_symbols,
    parser::{ast::TermTypes, DelisCmd, DelisProg},
};
use lexpr::Value;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
use sygus_parser::{
    BitVec, Bool, Cmd, GrammarDef, GroupedRuleList, Identifier, Literal, Sort, SortedVar, SyGuSCmd,
    SyGuSGTerm, SyGuSTerm, Term,
};

#[derive(Clone)]
enum BitExpr {
    Const(bool),
    Dynamic(SyGuSTerm),
}

#[derive(Clone)]
enum BoolExpr {
    Const(bool),
    Dynamic(SyGuSTerm),
}

#[derive(Clone)]
struct ChildBinding {
    arg_idx: usize,
}

#[derive(Default, Clone)]
struct Env {
    aspect: HashMap<String, AspectKey>,
    values: HashMap<String, AspectValue>,
    children: HashMap<String, ChildBinding>,
}

impl EnvView for Env {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        &self.aspect
    }

    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
}

struct NativeCompiler<'a> {
    problem: &'a DelisProg,
    term_types: &'a TermTypes,
    sig: TermSignature,
    ctx: SemanticContext<'a>,
    synth_name: String,
    synth_args: Vec<sygus_parser::SortedVar>,
    bit_index: HashMap<(String, AspectKey, Option<AspectValue>), usize>,
    ground_model_eval:
        RefCell<HashMap<(String, String, Vec<String>), Option<crate::lang::model::InterpedValue>>>,
    ground_model_eval_hits: RefCell<usize>,
    ground_model_eval_misses: RefCell<usize>,
    bool_eval_cache: RefCell<HashMap<BoolMemoKey, BoolExpr>>,
    finite_eq_cache: RefCell<HashMap<FiniteEqMemoKey, BoolExpr>>,
    quantifier_values_cache: RefCell<HashMap<QuantifierValuesMemoKey, Vec<AspectValue>>>,
    guarded_projection_cache: RefCell<HashMap<GuardedProjectionMemoKey, Vec<usize>>>,
    value_eval_cache: RefCell<HashMap<ExprEnvMemoKey, AspectValue>>,
    int_eval_cache: RefCell<HashMap<ExprEnvMemoKey, isize>>,
    aspect_eval_cache: RefCell<HashMap<AspectMemoKey, AspectKey>>,
    value_for_sort_cache: RefCell<HashMap<ValueForSortMemoKey, AspectValue>>,
    expr_symbol_cache: RefCell<HashMap<usize, Vec<String>>>,
    optimize: bool,
}

#[derive(Default)]
struct ConstructorStats {
    bits: usize,
    eval_time: Duration,
    lift_time: Duration,
    fallback_time: Duration,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct EnvMemoKey {
    aspects: Vec<(String, AspectKey)>,
    values: Vec<(String, AspectValue)>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct BoolMemoKey {
    expr_ptr: usize,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct FiniteEqMemoKey {
    expr_ptr: usize,
    target: AspectValue,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct QuantifierValuesMemoKey {
    expr_ptr: usize,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct GuardedProjectionMemoKey {
    relation: String,
    domain_expr_ptr: usize,
    guard_expr_ptr: usize,
    aspect_expr_ptr: usize,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ExprEnvMemoKey {
    expr_ptr: usize,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct AspectMemoKey {
    relation: String,
    expr_ptr: usize,
    env: EnvMemoKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ValueForSortMemoKey {
    sort: String,
    expr_ptr: usize,
    env: EnvMemoKey,
}

pub fn compile(ctx: &CompileContext<'_>, optimize: bool) -> Result<Vec<SyGuSCmd>, String> {
    let mut compiler = NativeCompiler::from_context(ctx, optimize)?;
    compiler.compile()
}

impl<'a> NativeCompiler<'a> {
    fn semantic_term_bit(
        &self,
        term_expr: &Value,
        bit: usize,
        env: &Env,
    ) -> Result<SyGuSTerm, String> {
        let child_name = term_expr
            .as_symbol()
            .ok_or_else(|| format!("semantic call term must be a symbol: {term_expr}"))?;
        if let Some(child) = env.children.get(child_name) {
            return Ok(bit_is_one(extract_arg_bit(child.arg_idx, bit)));
        }
        if child_name == self.synth_name {
            return Ok(bit_is_one(BitVec::extract(
                bit,
                bit,
                Term::var(&self.synth_name),
            )));
        }
        Err(format!("unknown semantic term variable {child_name}"))
    }

    fn sort_alias_cmds(&self) -> Result<Vec<SyGuSCmd>, String> {
        self.sig
            .sort_order()
            .iter()
            .map(|sort| {
                Ok(SyGuSCmd::DefineSort(
                    sort.clone(),
                    Vec::new(),
                    Sort::bitvec(self.sort_width(sort)?),
                ))
            })
            .collect()
    }

    fn from_context(ctx_data: &'a CompileContext<'a>, optimize: bool) -> Result<Self, String> {
        let sig = ctx_data.sig.clone();
        let ctx = ctx_data.semantic_context()?;
        let mut bit_index = HashMap::default();
        let mut offsets = HashMap::default();
        for domain in &ctx_data.semantic_program.domains {
            let offset = offsets.entry(domain.term_sort.clone()).or_insert(0);
            for bit in &domain.bits {
                bit_index.insert(
                    (
                        domain.relation.clone(),
                        bit.aspect.clone(),
                        bit.value.clone(),
                    ),
                    *offset + bit.bit,
                );
            }
            *offset += domain.bits.len();
        }
        Ok(Self {
            problem: ctx_data.problem,
            term_types: ctx_data.term_types,
            sig,
            ctx,
            synth_name: ctx_data.synth_name.clone(),
            synth_args: ctx_data.synth_args.clone(),
            bit_index,
            ground_model_eval: RefCell::new(HashMap::default()),
            ground_model_eval_hits: RefCell::new(0),
            ground_model_eval_misses: RefCell::new(0),
            bool_eval_cache: RefCell::new(HashMap::default()),
            finite_eq_cache: RefCell::new(HashMap::default()),
            quantifier_values_cache: RefCell::new(HashMap::default()),
            guarded_projection_cache: RefCell::new(HashMap::default()),
            value_eval_cache: RefCell::new(HashMap::default()),
            int_eval_cache: RefCell::new(HashMap::default()),
            aspect_eval_cache: RefCell::new(HashMap::default()),
            value_for_sort_cache: RefCell::new(HashMap::default()),
            expr_symbol_cache: RefCell::new(HashMap::default()),
            optimize,
        })
    }

    fn compile(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];
        cmds.extend(self.sort_alias_cmds()?);
        let grammar_start = Instant::now();
        let grammar = self.grammar()?;
        log::info!(
            "native grammar compilation completed in {:?}",
            grammar_start.elapsed()
        );
        for cons in &self.sig.constructors {
            let cons_start = Instant::now();
            cmds.push(self.constructor_fun(cons)?);
            log::info!(
                "native constructor {} compiled in {:?}",
                cons.name,
                cons_start.elapsed()
            );
        }
        let mut synth_cmd = Cmd::synth_fun(&self.synth_name)
            .ret(Sort::from_name(self.sig.start_sort()))
            .grammar(grammar);
        for arg in self.synth_args.clone() {
            synth_cmd = synth_cmd.arg(&arg.name, arg.sort);
        }
        cmds.push(synth_cmd.build());
        let env = Env::default();
        for cmd in &self.problem.cmds {
            let term = match cmd {
                DelisCmd::As(SyGuSCmd::Constraint(term)) => term,
                DelisCmd::DeclareAxiom(_) => continue,
                _ => continue,
            };
            cmds.push(
                Cmd::constraint()
                    .body(self.lower_top_level_constraint(term, &env)?)
                    .build(),
            );
        }
        cmds.push(SyGuSCmd::CheckSynth);
        Ok(cmds)
    }

    fn lower_top_level_constraint(&self, term: &SyGuSTerm, env: &Env) -> Result<SyGuSTerm, String> {
        match self.eval_bool(&sygus_term_to_value(term)?, env)? {
            BoolExpr::Const(value) => Ok(SyGuSTerm::Literal(Literal::Bool(value))),
            BoolExpr::Dynamic(term) => Ok(term),
        }
    }

    fn grammar(&self) -> Result<GrammarDef, String> {
        let sorted_vars = self
            .term_types
            .sort_decls
            .iter()
            .map(|sort_decl| {
                Ok(SortedVar {
                    name: self.sig.grammar_symbol_for_sort(&sort_decl.symbol),
                    sort: Sort::from_name(&sort_decl.symbol),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let grouped_rule_lists = self
            .term_types
            .sort_decls
            .iter()
            .zip(self.term_types.term_type_decls.iter())
            .map(|(sort_decl, term_type_decl)| {
                let symbol = self.sig.grammar_symbol_for_sort(&sort_decl.symbol);
                let terms = term_type_decl
                    .constructors
                    .iter()
                    .map(|cons| {
                        let name = cons.symbol.to_string();
                        let args = cons
                            .sorts
                            .iter()
                            .map(|sort| self.sig.grammar_symbol_for_sort(&sort.to_string()))
                            .collect();
                        gterm(&name, args)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(GroupedRuleList {
                    symbol,
                    sort: Sort::from_name(&sort_decl.symbol),
                    terms,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(GrammarDef {
            sorted_vars,
            grouped_rule_lists,
        })
    }

    fn constructor_fun(&self, cons: &TermConstructor) -> Result<SyGuSCmd, String> {
        let mut stats = ConstructorStats::default();
        let mut bool_bits = Vec::new();
        for domain in self.ctx.domains_for_sort(&cons.result_sort)? {
            let spec = self.ctx.spec_for_relation(&domain.relation)?;
            let cases = self.ctx.cases().get(&spec.relation).ok_or_else(|| {
                format!(
                    "missing parsed semantic cases for relation {}",
                    spec.relation
                )
            })?;
            let case = cases
                .get(&cons.name)
                .ok_or_else(|| format!("missing semantic case for constructor {}", cons.name))?;
            if case.vars.len() != cons.arg_sorts.len() {
                return Err(format!(
                    "match case for constructor {} in {} binds {} variables, expected {}",
                    cons.name,
                    spec.relation,
                    case.vars.len(),
                    cons.arg_sorts.len()
                ));
            }
            let body_is_ground = !case
                .vars
                .iter()
                .any(|var| self.expr_symbols(&case.body).iter().any(|name| name == var));
            let mut env = Env::default();
            for (idx, var) in case.vars.iter().enumerate() {
                env.children
                    .insert(var.clone(), ChildBinding { arg_idx: idx });
            }
            for bit in &domain.bits {
                env.aspect.insert("asp".to_string(), bit.aspect.clone());
                let eval_start = Instant::now();
                let bool_expr = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) if body_is_ground => {
                        match self.eval_bool(&case.body, &env)? {
                            BoolExpr::Const(value) => BoolExpr::Const(value),
                            BoolExpr::Dynamic(_) => {
                                return Err(format!(
                                    "ground Bool body unexpectedly lowered to dynamic term: {}",
                                    case.body
                                ));
                            }
                        }
                    }
                    (SemanticKind::FiniteValue { sort: _ }, Some(value)) if body_is_ground => {
                        match self.eval_finite_eq(&case.body, value, &env)? {
                            BoolExpr::Const(value) => BoolExpr::Const(value),
                            BoolExpr::Dynamic(_) => {
                                return Err(format!(
                                    "ground finite body unexpectedly lowered to dynamic term: {}",
                                    case.body
                                ));
                            }
                        }
                    }
                    (SemanticKind::Bool, None) => self.eval_bool(&case.body, &env)?,
                    (SemanticKind::FiniteValue { sort: _ }, Some(value)) => {
                        self.eval_finite_eq(&case.body, value, &env)?
                    }
                    _ => {
                        return Err(format!(
                            "malformed semantic domain bit {} for {}",
                            bit.bit, domain.relation
                        ));
                    }
                };
                stats.eval_time += eval_start.elapsed();
                stats.bits += 1;
                bool_bits.push(bool_expr);
            }
        }

        let mut builder = Cmd::define_fun(&cons.name);
        let arg_names = (0..cons.arg_sorts.len()).map(arg_name).collect::<Vec<_>>();
        for (idx, sort) in cons.arg_sorts.iter().enumerate() {
            builder = builder.arg(&arg_names[idx], Sort::from_name(sort));
        }
        let lift_bits = bool_bits
            .iter()
            .map(|expr| match expr {
                BoolExpr::Const(value) => BitwiseExpr::Const(*value),
                BoolExpr::Dynamic(term) => BitwiseExpr::Expr(term.clone()),
            })
            .collect::<Vec<_>>();
        let lift_start = Instant::now();
        let lifted = if self.optimize {
            optim::try_lift_native_bv(&lift_bits, &arg_names)
        } else {
            None
        };
        stats.lift_time = lift_start.elapsed();
        let body = if let Some(body) = lifted {
            body
        } else {
            let fallback_start = Instant::now();
            let bits = bool_bits
                .into_iter()
                .map(|expr| match expr {
                    BoolExpr::Const(value) => BitExpr::Const(value),
                    BoolExpr::Dynamic(term) => bool_to_bit(term),
                })
                .collect::<Vec<_>>();
            let body = compact_bitvec(bits);
            stats.fallback_time = fallback_start.elapsed();
            body
        };
        let cache_hits = *self.ground_model_eval_hits.borrow();
        let cache_misses = *self.ground_model_eval_misses.borrow();
        log::info!(
            "native constructor {} stats: bits={}, eval={:?}, lift={:?}, fallback={:?}, ground_model_cache_hits={}, ground_model_cache_misses={}",
            cons.name,
            stats.bits,
            stats.eval_time,
            stats.lift_time,
            stats.fallback_time,
            cache_hits,
            cache_misses,
        );
        Ok(builder
            .ret(Sort::from_name(&cons.result_sort))
            .body(body)
            .build())
    }

    fn eval_bool(&self, expr: &Value, env: &Env) -> Result<BoolExpr, String> {
        let memo_key = BoolMemoKey {
            expr_ptr: expr as *const Value as usize,
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.bool_eval_cache.borrow().get(&memo_key) {
            return Ok(cached.clone());
        }
        let result = self.eval_bool_uncached(expr, env)?;
        self.bool_eval_cache
            .borrow_mut()
            .insert(memo_key, result.clone());
        Ok(result)
    }

    fn eval_bool_uncached(&self, expr: &Value, env: &Env) -> Result<BoolExpr, String> {
        if let Some(value) = expr.as_bool() {
            return Ok(BoolExpr::Const(value));
        }
        if let Some(symbol) = expr.as_symbol() {
            return match symbol {
                "true" => Ok(BoolExpr::Const(true)),
                "false" => Ok(BoolExpr::Const(false)),
                _ => Err(format!("expected Bool expression, got {expr}")),
            };
        }
        let Some(items) = expr.to_ref_vec() else {
            return Err(format!("expected Bool expression, got {expr}"));
        };
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("Bool expression operator must be a symbol: {expr}"))?;
        match op {
            "not" if items.len() == 2 => Ok(match self.eval_bool(items[1], env)? {
                BoolExpr::Const(value) => BoolExpr::Const(!value),
                BoolExpr::Dynamic(term) => BoolExpr::Dynamic(Bool::not(term)),
            }),
            "and" => {
                self.eval_bool_conjunction(&items[1..], env)
            }
            "or" => {
                self.eval_bool_disjunction(&items[1..], env)
            }
            "ite" if items.len() == 4 => match self.eval_bool(items[1], env)? {
                BoolExpr::Const(true) => self.eval_bool(items[2], env),
                BoolExpr::Const(false) => self.eval_bool(items[3], env),
                cond => Ok(or_exprs(vec![
                    and_exprs(vec![cond.clone(), self.eval_bool(items[2], env)?]),
                    and_exprs(vec![negate_expr(cond), self.eval_bool(items[3], env)?]),
                ])),
            },
            "=" if items.len() == 3 => self.eval_equality(items[1], items[2], env),
            "<" if items.len() == 3 => self.eval_int_compare(items[1], items[2], env, |l, r| l < r),
            "<=" if items.len() == 3 => {
                self.eval_int_compare(items[1], items[2], env, |l, r| l <= r)
            }
            "forall" | "exists" if items.len() == 3 => {
                self.eval_quantifier(op, items[1], items[2], env)
            }
            relation if self.ctx.has_relation(relation) => {
                self.eval_semantic_bool_call(relation, &items[1..], env)
            }
            func if self.ctx.has_func(func) => self.eval_model_bool_call(func, &items[1..], env),
            _ => Err(format!("unsupported Bool expression: {expr}")),
        }
    }

    fn eval_quantifier(
        &self,
        op: &str,
        bindings: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let binding_list = bindings
            .to_ref_vec()
            .ok_or_else(|| format!("quantifier binding list must be a list: {bindings}"))?;
        if binding_list.len() != 1 {
            return Err(format!(
                "native BV lowering currently supports one binding per quantifier, got {bindings}"
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
        if let Some(result) = self.eval_guarded_quantifier(op, var, domain_expr, body, env)? {
            return Ok(result);
        }
        let mut next = env.clone();
        let mut dynamic = Vec::new();
        for value in self.quantifier_values(domain_expr, env)? {
            next.values.insert(var.to_string(), value);
            match (op, self.eval_bool(body, &next)?) {
                ("exists", BoolExpr::Const(true)) => return Ok(BoolExpr::Const(true)),
                ("exists", BoolExpr::Const(false)) => {}
                ("exists", BoolExpr::Dynamic(term)) => dynamic.push(term),
                ("forall", BoolExpr::Const(false)) => return Ok(BoolExpr::Const(false)),
                ("forall", BoolExpr::Const(true)) => {}
                ("forall", BoolExpr::Dynamic(term)) => dynamic.push(term),
                _ => unreachable!(),
            }
        }
        match op {
            "forall" => Ok(match dynamic.len() {
                0 => BoolExpr::Const(true),
                1 => BoolExpr::Dynamic(dynamic.remove(0)),
                _ => BoolExpr::Dynamic(Bool::and(dynamic)),
            }),
            "exists" => Ok(match dynamic.len() {
                0 => BoolExpr::Const(false),
                1 => BoolExpr::Dynamic(dynamic.remove(0)),
                _ => BoolExpr::Dynamic(Bool::or(dynamic)),
            }),
            _ => unreachable!(),
        }
    }

    fn eval_guarded_quantifier(
        &self,
        op: &str,
        var: &str,
        domain_expr: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        match op {
            "exists" => self.eval_guarded_exists(var, domain_expr, body, env),
            "forall" => self.eval_guarded_forall(var, domain_expr, body, env),
            _ => Ok(None),
        }
    }

    fn eval_guarded_exists(
        &self,
        var: &str,
        domain_expr: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        let Some(items) = body.to_ref_vec() else {
            return Ok(None);
        };
        if items.first().and_then(|item| item.as_symbol()) != Some("and") || items.len() <= 2 {
            return Ok(None);
        }
        for (idx, guard) in items[1..].iter().enumerate() {
            let mut rest = Vec::with_capacity(items.len().saturating_sub(2));
            for (j, item) in items[1..].iter().enumerate() {
                if j != idx {
                    rest.push(*item);
                }
            }
            if rest.len() == 1 {
                if let Some(result) = self.lower_guarded_projection(
                    "exists",
                    var,
                    domain_expr,
                    guard,
                    rest[0],
                    env,
                )? {
                    return Ok(Some(result));
                }
            }
            let Some(values) = self.const_guard_values(var, domain_expr, guard, true, env)? else {
                continue;
            };
            let result = self.eval_quantifier_over_values("exists", var, &values, &rest, env)?;
            return Ok(Some(result));
        }
        Ok(None)
    }

    fn eval_guarded_forall(
        &self,
        var: &str,
        domain_expr: &Value,
        body: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        let Some(items) = body.to_ref_vec() else {
            return Ok(None);
        };
        if items.first().and_then(|item| item.as_symbol()) != Some("or") || items.len() <= 2 {
            return Ok(None);
        }
        for (idx, item) in items[1..].iter().enumerate() {
            let Some(guard) = negated_guard(item) else {
                continue;
            };
            let mut rest = Vec::with_capacity(items.len().saturating_sub(2));
            for (j, disjunct) in items[1..].iter().enumerate() {
                if j != idx {
                    rest.push(*disjunct);
                }
            }
            if rest.len() == 1 {
                if let Some(result) = self.lower_guarded_projection(
                    "forall",
                    var,
                    domain_expr,
                    guard,
                    rest[0],
                    env,
                )? {
                    return Ok(Some(result));
                }
            }
            let Some(values) = self.const_guard_values(var, domain_expr, guard, true, env)? else {
                continue;
            };
            let result = self.eval_quantifier_over_values("forall", var, &values, &rest, env)?;
            return Ok(Some(result));
        }
        Ok(None)
    }

    fn lower_guarded_projection(
        &self,
        op: &str,
        var: &str,
        domain_expr: &Value,
        guard_expr: &Value,
        rest_expr: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        let Some(items) = rest_expr.to_ref_vec() else {
            return Ok(None);
        };
        let Some(relation) = items.first().and_then(|item| item.as_symbol()) else {
            return Ok(None);
        };
        if !self.ctx.has_relation(relation) {
            return Ok(None);
        }
        let domain = self.ctx.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool || items.len() != 3 {
            return Ok(None);
        }
        let term_expr = items[1];
        let aspect_expr = items[2];
        let bits = self.guarded_projection_bits(
            var,
            domain_expr,
            guard_expr,
            relation,
            aspect_expr,
            env,
        )?;
        let mut terms = Vec::with_capacity(bits.len());
        for bit in bits {
            terms.push(self.semantic_term_bit(term_expr, bit, env)?);
        }
        Ok(Some(match op {
            "exists" => match terms.len() {
                0 => BoolExpr::Const(false),
                1 => BoolExpr::Dynamic(terms.remove(0)),
                _ => BoolExpr::Dynamic(Bool::or(terms)),
            },
            "forall" => match terms.len() {
                0 => BoolExpr::Const(true),
                1 => BoolExpr::Dynamic(terms.remove(0)),
                _ => BoolExpr::Dynamic(Bool::and(terms)),
            },
            _ => unreachable!(),
        }))
    }

    fn guarded_projection_bits(
        &self,
        var: &str,
        domain_expr: &Value,
        guard_expr: &Value,
        relation: &str,
        aspect_expr: &Value,
        env: &Env,
    ) -> Result<Vec<usize>, String> {
        let key = GuardedProjectionMemoKey {
            relation: relation.to_string(),
            domain_expr_ptr: domain_expr as *const Value as usize,
            guard_expr_ptr: guard_expr as *const Value as usize,
            aspect_expr_ptr: aspect_expr as *const Value as usize,
            env: self.memo_env_key(guard_expr, env),
        };
        if let Some(cached) = self.guarded_projection_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let Some(values) = self.const_guard_values(var, domain_expr, guard_expr, true, env)? else {
            return Err(format!(
                "guarded projection expected constant guard: {}",
                guard_expr
            ));
        };
        let mut next = env.clone();
        let mut bits = Vec::with_capacity(values.len());
        for value in values {
            next.values.insert(var.to_string(), value);
            let aspect = self.cached_eval_aspect(aspect_expr, relation, &next)?;
            bits.push(self.bit_for(relation, &aspect, None)?);
        }
        self.guarded_projection_cache
            .borrow_mut()
            .insert(key, bits.clone());
        Ok(bits)
    }

    fn const_guard_values(
        &self,
        var: &str,
        domain_expr: &Value,
        guard: &Value,
        want: bool,
        env: &Env,
    ) -> Result<Option<Vec<AspectValue>>, String> {
        let mut next = env.clone();
        let mut values = Vec::new();
        for value in self.quantifier_values(domain_expr, env)? {
            next.values.insert(var.to_string(), value.clone());
            match self.eval_bool(guard, &next)? {
                BoolExpr::Const(result) => {
                    if result == want {
                        values.push(value);
                    }
                }
                BoolExpr::Dynamic(_) => return Ok(None),
            }
        }
        Ok(Some(values))
    }

    fn eval_quantifier_over_values(
        &self,
        op: &str,
        var: &str,
        values: &[AspectValue],
        body_items: &[&Value],
        env: &Env,
    ) -> Result<BoolExpr, String> {
        if body_items.len() == 1 {
            if let Some(result) =
                self.lower_quantified_semantic_projection(op, var, values, body_items[0], env)?
            {
                return Ok(result);
            }
        }
        let mut next = env.clone();
        let mut dynamic = Vec::new();
        for value in values {
            next.values.insert(var.to_string(), value.clone());
            let expr = match op {
                "exists" => self.eval_bool_conjunction(body_items, &next)?,
                "forall" => self.eval_bool_disjunction(body_items, &next)?,
                _ => unreachable!(),
            };
            match (op, expr) {
                ("exists", BoolExpr::Const(true)) => return Ok(BoolExpr::Const(true)),
                ("exists", BoolExpr::Const(false)) => {}
                ("exists", BoolExpr::Dynamic(term)) => dynamic.push(term),
                ("forall", BoolExpr::Const(false)) => return Ok(BoolExpr::Const(false)),
                ("forall", BoolExpr::Const(true)) => {}
                ("forall", BoolExpr::Dynamic(term)) => dynamic.push(term),
                _ => unreachable!(),
            }
        }
        Ok(match op {
            "exists" => match dynamic.len() {
                0 => BoolExpr::Const(false),
                1 => BoolExpr::Dynamic(dynamic.remove(0)),
                _ => BoolExpr::Dynamic(Bool::or(dynamic)),
            },
            "forall" => match dynamic.len() {
                0 => BoolExpr::Const(true),
                1 => BoolExpr::Dynamic(dynamic.remove(0)),
                _ => BoolExpr::Dynamic(Bool::and(dynamic)),
            },
            _ => unreachable!(),
        })
    }

    fn lower_quantified_semantic_projection(
        &self,
        op: &str,
        var: &str,
        values: &[AspectValue],
        expr: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        let Some(items) = expr.to_ref_vec() else {
            return Ok(None);
        };
        let Some(relation) = items.first().and_then(|item| item.as_symbol()) else {
            return Ok(None);
        };
        if !self.ctx.has_relation(relation) {
            return Ok(None);
        }
        let domain = self.ctx.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool || items.len() != 3 {
            return Ok(None);
        }
        let term_expr = items[1];
        let aspect_expr = items[2];
        let mut next = env.clone();
        let mut terms = Vec::new();
        for value in values {
            next.values.insert(var.to_string(), value.clone());
            let aspect = self.cached_eval_aspect(aspect_expr, relation, &next)?;
            let bit = self.bit_for(relation, &aspect, None)?;
            terms.push(self.semantic_term_bit(term_expr, bit, &next)?);
        }
        Ok(Some(match op {
            "exists" => match terms.len() {
                0 => BoolExpr::Const(false),
                1 => BoolExpr::Dynamic(terms.remove(0)),
                _ => BoolExpr::Dynamic(Bool::or(terms)),
            },
            "forall" => match terms.len() {
                0 => BoolExpr::Const(true),
                1 => BoolExpr::Dynamic(terms.remove(0)),
                _ => BoolExpr::Dynamic(Bool::and(terms)),
            },
            _ => unreachable!(),
        }))
    }

    fn eval_bool_conjunction(&self, items: &[&Value], env: &Env) -> Result<BoolExpr, String> {
        let mut dynamic = Vec::new();
        for item in items {
            match self.eval_bool(item, env)? {
                BoolExpr::Const(false) => return Ok(BoolExpr::Const(false)),
                BoolExpr::Const(true) => {}
                BoolExpr::Dynamic(term) => dynamic.push(term),
            }
        }
        Ok(match dynamic.len() {
            0 => BoolExpr::Const(true),
            1 => BoolExpr::Dynamic(dynamic.remove(0)),
            _ => BoolExpr::Dynamic(Bool::and(dynamic)),
        })
    }

    fn eval_bool_disjunction(&self, items: &[&Value], env: &Env) -> Result<BoolExpr, String> {
        let mut dynamic = Vec::new();
        for item in items {
            match self.eval_bool(item, env)? {
                BoolExpr::Const(true) => return Ok(BoolExpr::Const(true)),
                BoolExpr::Const(false) => {}
                BoolExpr::Dynamic(term) => dynamic.push(term),
            }
        }
        Ok(match dynamic.len() {
            0 => BoolExpr::Const(false),
            1 => BoolExpr::Dynamic(dynamic.remove(0)),
            _ => BoolExpr::Dynamic(Bool::or(dynamic)),
        })
    }

    fn quantifier_values(
        &self,
        domain_expr: &Value,
        env: &Env,
    ) -> Result<Vec<AspectValue>, String> {
        let memo_key = QuantifierValuesMemoKey {
            expr_ptr: domain_expr as *const Value as usize,
            env: self.memo_env_key(domain_expr, env),
        };
        if let Some(cached) = self.quantifier_values_cache.borrow().get(&memo_key) {
            return Ok(cached.clone());
        }
        let items = domain_expr
            .to_ref_vec()
            .ok_or_else(|| format!("unsupported domain expression: {domain_expr}"))?;
        let result = match items.first().and_then(|item| item.as_symbol()) {
            Some("int-range") if items.len() == 3 => {
                let lo = self.ctx.eval_int_expr(items[1], env)?;
                let hi = self.cached_eval_int_expr(items[2], env)?;
                if lo < 0 || hi < lo {
                    Vec::new()
                } else {
                    (lo as usize..=hi as usize).map(AspectValue::Int).collect()
                }
            }
            _ => {
                let (model, sort) = self.ctx.parse_domain_expr(domain_expr, env)?;
                let model_ref = self.ctx.model(&model)?;
                let values = crate::lang::eval::get_domain_for_sort(model_ref, &sort)
                    .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
                values
                    .into_iter()
                    .map(|name| AspectValue::Univ {
                        sort: sort.clone(),
                        name,
                    })
                    .collect()
            }
        };
        self.quantifier_values_cache
            .borrow_mut()
            .insert(memo_key, result.clone());
        Ok(result)
    }

    fn eval_semantic_bool_call(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let domain = self.ctx.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool {
            return Err(format!("{relation} does not return Bool"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let aspect = self.cached_eval_aspect(args[1], relation, env)?;
        let bit = self.bit_for(relation, &aspect, None)?;
        Ok(BoolExpr::Dynamic(
            self.semantic_term_bit(args[0], bit, env)?,
        ))
    }

    fn eval_model_bool_call(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let sig = self
            .ctx
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        if sig.ret_sort.to_string() != "Bool" {
            return Err(format!("function {func} does not return Bool"));
        }
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {func} expects {} arguments, got {}",
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        if let Some(arg_names) = self.ground_arg_names(sig, args, env) {
            let value = self.ground_model_value(func, &model, &arg_names, model_ref);
            return Ok(BoolExpr::Const(value.as_ref().is_some_and(interped_bool)));
        }
        let mut disjuncts = Vec::new();
        for tuple in self
            .ctx
            .matching_model_call_tuples(func, args, env, interped_bool)?
        {
            let conjuncts = args
                .iter()
                .zip(tuple.iter())
                .map(|(arg_expr, value)| self.eval_finite_eq(arg_expr, value, env))
                .collect::<Result<Vec<_>, _>>()?;
            disjuncts.push(and_exprs(conjuncts));
        }
        Ok(or_exprs(disjuncts))
    }

    fn eval_finite_terms_equal(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let sort = self
            .ctx
            .infer_finite_sort(lhs, env)
            .or_else(|| self.ctx.infer_finite_sort(rhs, env))
            .ok_or_else(|| format!("cannot infer finite sort for equality (= {lhs} {rhs})"))?;
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = crate::lang::eval::get_domain_for_sort(model_ref, &sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let disjuncts = values
            .into_iter()
            .map(|name| {
                let value = AspectValue::Univ {
                    sort: sort.clone(),
                    name,
                };
                Ok(and_exprs(vec![
                    self.eval_finite_eq(lhs, &value, env)?,
                    self.eval_finite_eq(rhs, &value, env)?,
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(or_exprs(disjuncts))
    }

    fn eval_equality(&self, lhs: &Value, rhs: &Value, env: &Env) -> Result<BoolExpr, String> {
        if let (Ok(lhs_value), Ok(rhs_value)) =
            (self.cached_eval_value(lhs, env), self.cached_eval_value(rhs, env))
        {
            return Ok(BoolExpr::Const(
                self.ctx.values_equal(&lhs_value, &rhs_value, env)?,
            ));
        }
        if self.cached_eval_int_expr(lhs, env).is_ok() && self.cached_eval_int_expr(rhs, env).is_ok() {
            return self.eval_int_compare(lhs, rhs, env, |l, r| l == r);
        }
        self.eval_finite_terms_equal(lhs, rhs, env)
    }

    fn eval_int_compare<F>(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
        cmp: F,
    ) -> Result<BoolExpr, String>
    where
        F: FnOnce(isize, isize) -> bool,
    {
        let lhs = self.cached_eval_int_expr(lhs, env)?;
        let rhs = self.cached_eval_int_expr(rhs, env)?;
        Ok(BoolExpr::Const(cmp(lhs, rhs)))
    }

    fn eval_finite_eq(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let memo_key = FiniteEqMemoKey {
            expr_ptr: expr as *const Value as usize,
            target: target.clone(),
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.finite_eq_cache.borrow().get(&memo_key) {
            return Ok(cached.clone());
        }
        let result = self.eval_finite_eq_uncached(expr, target, env)?;
        self.finite_eq_cache
            .borrow_mut()
            .insert(memo_key, result.clone());
        Ok(result)
    }

    fn eval_finite_eq_uncached(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        if let Ok(value) = self.cached_eval_value(expr, env) {
            return Ok(BoolExpr::Const(
                self.ctx.value_matches_target(&value, target, env)?,
            ));
        }
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(BoolExpr::Const(
                    self.ctx.value_matches_target(value, target, env)?,
                ));
            }
            if let Some(matches) = self.ctx.constant_value_matches(symbol, target, env)? {
                return Ok(BoolExpr::Const(matches));
            }
            return Ok(BoolExpr::Const(symbol == target.display()));
        }
        let Some(items) = expr.to_ref_vec() else {
            return Err(format!("expected finite expression, got {expr}"));
        };
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("finite expression operator must be a symbol: {expr}"))?;
        if op == "ite" && items.len() == 4 {
            return match self.eval_bool(items[1], env)? {
                BoolExpr::Const(true) => self.eval_finite_eq(items[2], target, env),
                BoolExpr::Const(false) => self.eval_finite_eq(items[3], target, env),
                cond => Ok(or_exprs(vec![
                    and_exprs(vec![
                        cond.clone(),
                        self.eval_finite_eq(items[2], target, env)?,
                    ]),
                    and_exprs(vec![
                        negate_expr(cond),
                        self.eval_finite_eq(items[3], target, env)?,
                    ]),
                ])),
            };
        }
        if items.len() == 2 {
            if let Some(aspect_arg) = items[1].as_symbol() {
                if let Some(aspect) = env.aspect.get(aspect_arg) {
                    if let Some((_, value)) = aspect.fields.iter().find(|(name, _)| name == op) {
                        return Ok(BoolExpr::Const(value == target));
                    }
                    if op == "model" {
                        return Ok(BoolExpr::Const(
                            AspectValue::Model(aspect.model.clone()) == *target,
                        ));
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

    fn memo_env_key(&self, expr: &Value, env: &Env) -> EnvMemoKey {
        let referenced = self.expr_symbols(expr);
        let mut aspects = Vec::new();
        let mut values = Vec::new();
        for name in &referenced {
            if let Some(aspect) = env.aspect.get(name) {
                aspects.push((name.clone(), aspect.clone()));
            }
            if let Some(value) = env.values.get(name) {
                values.push((name.clone(), value.clone()));
            }
        }
        EnvMemoKey {
            aspects,
            values,
        }
    }

    fn expr_symbols(&self, expr: &Value) -> Vec<String> {
        let expr_ptr = expr as *const Value as usize;
        if let Some(cached) = self.expr_symbol_cache.borrow().get(&expr_ptr) {
            return cached.clone();
        }
        let mut symbols = HashSet::default();
        collect_expr_symbols(expr, &mut symbols);
        let mut symbols = symbols.into_iter().collect::<Vec<_>>();
        symbols.sort();
        self.expr_symbol_cache
            .borrow_mut()
            .insert(expr_ptr, symbols.clone());
        symbols
    }

    fn eval_semantic_finite_call_eq(
        &self,
        relation: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let domain = self.ctx.domain_for_relation(relation)?;
        if !matches!(domain.kind, SemanticKind::FiniteValue { .. }) {
            return Err(format!("{relation} does not return a finite value"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let aspect = self.cached_eval_aspect(args[1], relation, env)?;
        let bit = self.bit_for(relation, &aspect, Some(target))?;
        Ok(BoolExpr::Dynamic(
            self.semantic_term_bit(args[0], bit, env)?,
        ))
    }

    fn eval_model_finite_call_eq(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let sig = self
            .ctx
            .func_sigs
            .get(func)
            .ok_or_else(|| format!("unknown function {func}"))?;
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {func} expects {} arguments, got {}",
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.ctx.current_model(env)?;
        let model_ref = self.ctx.model(&model)?;
        if let Some(arg_names) = self.ground_arg_names(sig, args, env) {
            let value = self.ground_model_value(func, &model, &arg_names, model_ref);
            return Ok(BoolExpr::Const(
                value
                    .as_ref()
                    .is_some_and(|value| interped_value_matches(value, target)),
            ));
        }
        let mut disjuncts = Vec::new();
        for tuple in self
            .ctx
            .matching_model_call_tuples(func, args, env, |value| {
                interped_value_matches(value, target)
            })?
        {
            let conjuncts = args
                .iter()
                .zip(tuple.iter())
                .map(|(arg_expr, value)| self.eval_finite_eq(arg_expr, value, env))
                .collect::<Result<Vec<_>, _>>()?;
            disjuncts.push(and_exprs(conjuncts));
        }
        Ok(or_exprs(disjuncts))
    }

    fn sort_width(&self, sort: &str) -> Result<usize, String> {
        Ok(self
            .ctx
            .domains_for_sort(sort)?
            .iter()
            .map(|domain| domain.bits.len())
            .sum::<usize>()
            .max(1))
    }

    fn bit_for(
        &self,
        relation: &str,
        aspect: &AspectKey,
        value: Option<&AspectValue>,
    ) -> Result<usize, String> {
        self.bit_index
            .get(&(relation.to_string(), aspect.clone(), value.cloned()))
            .copied()
            .ok_or_else(|| {
                format!(
                    "no bit for {}({}; {}){}",
                    relation,
                    aspect.model,
                    aspect.display_fields(),
                    value
                        .map(|v| format!(" = {}", v.display()))
                        .unwrap_or_default()
                )
            })
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
                self.cached_eval_value_for_sort(arg.0, &arg.1.to_string(), env)
                    .ok()
                    .map(|value| value.display())
            })
            .collect()
    }

    fn cached_eval_value(&self, expr: &Value, env: &Env) -> Result<AspectValue, String> {
        let key = ExprEnvMemoKey {
            expr_ptr: expr as *const Value as usize,
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.value_eval_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let value = self.ctx.eval_value(expr, env)?;
        self.value_eval_cache.borrow_mut().insert(key, value.clone());
        Ok(value)
    }

    fn cached_eval_int_expr(&self, expr: &Value, env: &Env) -> Result<isize, String> {
        let key = ExprEnvMemoKey {
            expr_ptr: expr as *const Value as usize,
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.int_eval_cache.borrow().get(&key) {
            return Ok(*cached);
        }
        let value = self.ctx.eval_int_expr(expr, env)?;
        self.int_eval_cache.borrow_mut().insert(key, value);
        Ok(value)
    }

    fn cached_eval_aspect(
        &self,
        expr: &Value,
        relation: &str,
        env: &Env,
    ) -> Result<AspectKey, String> {
        let key = AspectMemoKey {
            relation: relation.to_string(),
            expr_ptr: expr as *const Value as usize,
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.aspect_eval_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let value = self.ctx.eval_aspect(expr, relation, env)?;
        self.aspect_eval_cache.borrow_mut().insert(key, value.clone());
        Ok(value)
    }

    fn cached_eval_value_for_sort(
        &self,
        expr: &Value,
        sort: &str,
        env: &Env,
    ) -> Result<AspectValue, String> {
        let key = ValueForSortMemoKey {
            sort: sort.to_string(),
            expr_ptr: expr as *const Value as usize,
            env: self.memo_env_key(expr, env),
        };
        if let Some(cached) = self.value_for_sort_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let value = self.ctx.eval_value_for_sort(expr, sort, env)?;
        self.value_for_sort_cache
            .borrow_mut()
            .insert(key, value.clone());
        Ok(value)
    }

    fn ground_model_value(
        &self,
        func: &str,
        model: &str,
        arg_names: &[String],
        model_ref: &crate::lang::model::Model,
    ) -> Option<crate::lang::model::InterpedValue> {
        let key = (func.to_string(), model.to_string(), arg_names.to_vec());
        if let Some(value) = self.ground_model_eval.borrow().get(&key) {
            *self.ground_model_eval_hits.borrow_mut() += 1;
            return value.clone();
        }
        *self.ground_model_eval_misses.borrow_mut() += 1;
        let value = eval_symbols(func, arg_names, model_ref).cloned();
        self.ground_model_eval
            .borrow_mut()
            .insert(key, value.clone());
        value
    }
}

fn gterm(name: &str, args: Vec<String>) -> Result<SyGuSGTerm, String> {
    let id = Identifier::from_str(name)
        .map_err(|e| format!("invalid grammar identifier {name}: {e}"))?;
    if args.is_empty() {
        return Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Identifier(id)));
    }
    Ok(SyGuSGTerm::SyGuSTerm(SyGuSTerm::Application(
        id,
        args.into_iter()
            .map(|arg| {
                Identifier::from_str(&arg)
                    .map(SyGuSTerm::Identifier)
                    .map_err(|e| format!("invalid grammar argument {arg}: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    )))
}

fn negated_guard(expr: &Value) -> Option<&Value> {
    let items = expr.to_ref_vec()?;
    if items.len() == 2 && items.first().and_then(|item| item.as_symbol()) == Some("not") {
        return Some(items[1]);
    }
    None
}

fn bitvec_literal_from_lsb(bits: &[bool]) -> SyGuSTerm {
    let bitstring = bits
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect::<String>();
    Term::lit(Literal::BinConst(bitstring))
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

fn negate_expr(expr: BoolExpr) -> BoolExpr {
    match expr {
        BoolExpr::Const(value) => BoolExpr::Const(!value),
        BoolExpr::Dynamic(term) => BoolExpr::Dynamic(Bool::not(term)),
    }
}

fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
}

fn extract_arg_bit(arg_idx: usize, bit: usize) -> SyGuSTerm {
    BitVec::extract(bit, bit, Term::var(&arg_name(arg_idx)))
}

fn bit_is_one(bit: SyGuSTerm) -> SyGuSTerm {
    Bool::eq(bit, BitVec::const_(1, 1))
}

fn collect_expr_symbols(expr: &Value, out: &mut HashSet<String>) {
    if let Some(symbol) = expr.as_symbol() {
        out.insert(symbol.to_string());
        return;
    }
    if let Some(items) = expr.to_ref_vec() {
        for item in items {
            collect_expr_symbols(item, out);
        }
    }
}
