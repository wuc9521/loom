use crate::{
    compile::optim,
    compile::{
        context::CompileContext,
        emit::{arg_name, BoolExpr},
        expr::sygus_term_to_value,
        eval_ctx::{EnvView, SemanticContext},
        semantic::{interped_bool, interped_value_matches},
        syntax::{TermConstructor, TermSignature},
        types::{AspectKey, AspectValue, SemanticKind},
    },
    parser::{ast::TermTypes, DelisCmd, DelisProg},
};
use lexpr::Value;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
use sygus_parser::{
    BitVec, Bool, Cmd, GrammarDef, GroupedRuleList, Identifier, Literal, Sort, SortedVar,
    SyGuSCmd, SyGuSGTerm, SyGuSTerm, Term,
};

const CLOSURE_AUX_NAME: &str = "delis.closure.aux";
const SELF_BIT_PREFIX: &str = "delis.self.";


#[derive(Clone)]
struct ChildBinding {
    arg_idx: usize,
}

#[derive(Default, Clone)]
struct Env {
    aspect: HashMap<String, AspectKey>,
    values: HashMap<String, AspectValue>,
    children: HashMap<String, ChildBinding>,
    relation: Option<String>,
    self_var: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecurrenceMode {
    Direct,
    SelfClosure,
    ParentClosure,
}

struct RecurrenceAnalysis {
    relation_modes: HashMap<String, RecurrenceMode>,
    requires_aux: bool,
    /// Term sorts whose semantic relation is in `SelfClosure` mode.
    self_closure_sorts: HashSet<String>,
}

#[derive(Default)]
struct RelationLocal {
    has_parent: bool,
    self_recursive: bool,
    calls: HashSet<String>,
}

impl EnvView for Env {
    fn aspects(&self) -> &HashMap<String, AspectKey> {
        &self.aspect
    }

    fn values(&self) -> &HashMap<String, AspectValue> {
        &self.values
    }
}

struct Compiler<'a> {
    problem: &'a DelisProg,
    term_types: &'a TermTypes,
    sig: TermSignature,
    ctx: SemanticContext<'a>,
    recurrence: RecurrenceAnalysis,
    synth_name: String,
    synth_args: Vec<sygus_parser::SortedVar>,
    /// If `Some`, the program uses `(parent ...)` and we synthesize an
    /// auxiliary constant of the same sort as g (the start sort). All
    /// `(parent n)` references in semantic bodies extract bits from this
    /// aux; the top-level closure constraint `(= g aux)` forces the LFP.
    aux_synth_name: Option<String>,
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

impl<'a> CompileContext<'a> {
    pub fn compile(&self) -> Result<Vec<SyGuSCmd>, String> {
        let mut compiler = Compiler::from_context(self, super::opts().optimize)?;
        compiler.compile()
    }
}

impl<'a> Compiler<'a> {
    fn semantic_term_bit(
        &self,
        term_expr: &Value,
        bit: usize,
        env: &Env,
    ) -> Result<SyGuSTerm, String> {
        // `(parent n)` — closure reference: extract the corresponding bit
        // from the auxiliary synth-fun.  The top-level `(= g aux)` constraint
        // forces the LFP, so the parent's evaluation at this aspect equals
        // g's own evaluation.
        if let Some(items) = term_expr.to_ref_vec() {
            if items.first().and_then(|item| item.as_symbol()) == Some("parent") {
                let aux = self.aux_synth_name.as_deref().ok_or_else(|| {
                    "semantic body uses (parent ...) but no aux synth-fun allocated".to_string()
                })?;
                return Ok(bit_is_one(BitVec::extract(bit, bit, Term::var(aux))));
            }
        }
        let child_name = term_expr
            .as_symbol()
            .ok_or_else(|| format!("semantic call term must be a symbol: {term_expr}"))?;
        if let Some(child) = env.children.get(child_name) {
            return Ok(bit_is_one(extract_arg_bit(child.arg_idx, bit)));
        }
        if env.self_var.as_deref() == Some(child_name) {
            let mode = env
                .relation
                .as_deref()
                .and_then(|relation| self.recurrence.relation_modes.get(relation))
                .copied()
                .unwrap_or(RecurrenceMode::Direct);
            return match mode {
                RecurrenceMode::Direct => Err(format!(
                    "semantic body uses self recursion in direct component {}",
                    env.relation.as_deref().unwrap_or("<unknown>")
                )),
                RecurrenceMode::SelfClosure => {
                    // Emit a placeholder Bool variable instead of a runtime
                    // (extract i synth-fun) reference. A post-pass in
                    // `compile()` collects these placeholders, topo-sorts the
                    // bit dependency graph, and wraps the constructor body in
                    // nested let-bindings — statically unrolling the LFP so
                    // dryadsynth-bv sees a pure nullary problem (no synth-fun
                    // reference in any define-fun body). This mirrors what
                    // the old CHC-saturation pipeline did.
                    Ok(Term::var(&self_bit_name(bit)))
                }
                RecurrenceMode::ParentClosure => {
                    let aux = self.aux_synth_name.as_deref().ok_or_else(|| {
                        "semantic body uses self-recursion but no aux synth-fun allocated"
                            .to_string()
                    })?;
                    Ok(bit_is_one(BitVec::extract(bit, bit, Term::var(aux))))
                }
            };
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

    fn from_context(ctx_data: &'a CompileContext<'a>, optimize: bool) -> Result<Self, String> {
        let sig = ctx_data.sig.clone();
        let ctx = ctx_data.semantic_context()?;
        let recurrence = analyze_recurrence(ctx_data)?;
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
            *offset += domain.width();
        }
        let aux_synth_name = if recurrence.requires_aux {
            Some(CLOSURE_AUX_NAME.to_string())
        } else {
            None
        };
        Ok(Self {
            problem: ctx_data.problem,
            term_types: ctx_data.term_types,
            sig,
            ctx,
            recurrence,
            synth_name: ctx_data.synth_name.clone(),
            synth_args: ctx_data.synth_args.clone(),
            aux_synth_name,
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
        let grammar_start = Instant::now();
        let grammar = self.grammar()?;
        log::info!(
            "native grammar compilation completed in {:?}",
            grammar_start.elapsed()
        );

        // Phase 1: evaluate all constructor bits
        struct ConsResult {
            name: String,
            result_sort: String,
            arg_sorts: Vec<String>,
            bool_bits: Vec<BoolExpr>,
        }
        let mut cons_results = Vec::new();
        for cons in &self.sig.constructors {
            let cons_start = Instant::now();
            let bool_bits = self.eval_constructor_bits(cons)?;
            log::info!(
                "native constructor {} compiled in {:?}",
                cons.name,
                cons_start.elapsed()
            );
            cons_results.push(ConsResult {
                name: cons.name.clone(),
                result_sort: cons.result_sort.clone(),
                arg_sorts: cons.arg_sorts.clone(),
                bool_bits,
            });
        }


        // Phase 3: emit commands with remapped bits
        let phase3_start = Instant::now();
        let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];
        cmds.extend(self.remapped_sort_alias_cmds()?);
        if let Some(aux_name) = self.aux_synth_name.clone() {
            cmds.push(self.remapped_aux_synth_fun_cmd(&aux_name)?);
        }
        let mut total_lift = Duration::default();
        let mut total_compact = Duration::default();
        let mut total_closure = Duration::default();
        for cr in &cons_results {
            let alive_bits: Vec<BoolExpr> = cr.bool_bits.clone();
            let mut sort_map: HashMap<String, &str> = HashMap::default();
            for (idx, sort) in cr.arg_sorts.iter().enumerate() {
                sort_map.insert(arg_name(idx), sort.as_str());
            }
            sort_map.insert(self.synth_name.clone(), self.sig.start_sort());
            if let Some(aux) = &self.aux_synth_name {
                sort_map.insert(aux.clone(), self.sig.start_sort());
            }
            let remapped_bits: Vec<BoolExpr> = alive_bits
                .into_iter()
                .map(|expr| match expr {
                    BoolExpr::Dynamic(term) => {
                        BoolExpr::Dynamic(term)
                    }
                    c => c,
                })
                .collect();
            let mut builder = Cmd::define_fun(&cr.name);
            let arg_names = (0..cr.arg_sorts.len()).map(arg_name).collect::<Vec<_>>();
            for (idx, sort) in cr.arg_sorts.iter().enumerate() {
                builder = builder.arg(&arg_names[idx], Sort::from_name(sort));
            }
            let lift_start = Instant::now();
            let lifted = if self.optimize {
                optim::try_lift_native_bv(&remapped_bits, &arg_names)
            } else {
                None
            };
            let lift_elapsed = lift_start.elapsed();
            total_lift += lift_elapsed;
            let body_start = Instant::now();
            let body = if let Some(body) = lifted {
                body
            } else if self.is_self_closure_sort(&cr.result_sort) {
                // Constructor body may reference __delis_self_b{j} placeholders
                // (emitted by semantic_term_bit in SelfClosure mode). Wrap in
                // a topo-sorted let-binding chain so the LFP is fully unrolled
                // at compile time — no runtime synth-fun reference required.
                BoolExpr::resolve_closure(&remapped_bits)
            } else {
                BoolExpr::compact(remapped_bits)
            };
            let body_elapsed = body_start.elapsed();
            if self.is_self_closure_sort(&cr.result_sort) {
                total_closure += body_elapsed;
            } else {
                total_compact += body_elapsed;
            }
            log::info!(
                "phase3 emit {}: lift={:?} body={:?}",
                cr.name,
                lift_elapsed,
                body_elapsed
            );
            cmds.push(
                builder
                    .ret(Sort::from_name(&cr.result_sort))
                    .body(body)
                    .build(),
            );
        }
        log::info!(
            "phase3 totals: lift={:?} compact={:?} closure={:?} elapsed={:?}",
            total_lift,
            total_compact,
            total_closure,
            phase3_start.elapsed()
        );
        let mut synth_cmd = Cmd::synth_fun(&self.synth_name)
            .ret(Sort::from_name(self.sig.start_sort()))
            .grammar(grammar);
        for arg in self.synth_args.clone() {
            synth_cmd = synth_cmd.arg(&arg.name, arg.sort);
        }
        cmds.push(synth_cmd.build());
        let env = Env::default();
        let top_level_constraints = self
            .problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DelisCmd::As(SyGuSCmd::Constraint(term)) => Some(term),
                DelisCmd::DeclareAxiom(_) => None,
                _ => None,
            })
            .map(sygus_term_to_value)
            .collect::<Result<Vec<_>, _>>()?;
        let constraints_start = Instant::now();
        for term in &top_level_constraints {
            let lowered = self.lower_top_level_constraint(term, &env)?;
            let mut constraint_sort_map: HashMap<String, &str> = HashMap::default();
            constraint_sort_map.insert(self.synth_name.clone(), self.sig.start_sort());
            if let Some(aux) = &self.aux_synth_name {
                constraint_sort_map.insert(aux.clone(), self.sig.start_sort());
            }
            let remapped = lowered;
            cmds.push(Cmd::constraint().body(remapped).build());
        }
        log::info!(
            "phase3 top-level constraints completed in {:?}",
            constraints_start.elapsed()
        );
        if let Some(aux_name) = &self.aux_synth_name {
            let closure = Bool::eq(
                Term::var(&self.synth_name),
                Term::var(aux_name),
            );
            cmds.push(Cmd::constraint().body(closure).build());
        }
        cmds.push(SyGuSCmd::CheckSynth);
        Ok(cmds)
    }

    /// True iff any semantic relation operating on `sort` is in
    /// `RecurrenceMode::SelfClosure` mode. Constructors of such sorts produce
    /// bool_bits that contain `__delis_self_b{j}` placeholders, which need
    /// the `build_self_closure_body` post-pass to be wrapped in let-bindings.
    fn is_self_closure_sort(&self, sort: &str) -> bool {
        self.recurrence
            .self_closure_sorts
            .contains(sort)
    }

    fn lower_top_level_constraint(&self, term: &Value, env: &Env) -> Result<SyGuSTerm, String> {
        match self.eval_bool(term, env)? {
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

    fn eval_constructor_bits(&self, cons: &TermConstructor) -> Result<Vec<BoolExpr>, String> {
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
            env.relation = Some(spec.relation.clone());
            env.self_var = Some(case.self_var.clone());
            for (idx, var) in case.vars.iter().enumerate() {
                env.children
                    .insert(var.clone(), ChildBinding { arg_idx: idx });
            }
            // Determine the canonical width of this domain. Under bisim
            // quotient, multiple aspects may share the same bit index;
            // equivalent aspects produce identical exprs, so it's safe to
            // assign by `bit.bit` rather than by enumeration order.
            let domain_width = domain.width();
            let domain_start = bool_bits.len();
            bool_bits.resize(domain_start + domain_width, BoolExpr::Const(false));
            let mut filled = vec![false; domain_width];
            for bit in &domain.bits {
                if filled[bit.bit] {
                    continue;
                }
                env.aspect.insert("asp".to_string(), bit.aspect.clone());
                let eval_start = Instant::now();
                let bool_expr = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) if body_is_ground => {
                        self.eval_bool(&case.body, &env)?
                    }
                    (SemanticKind::FiniteValue { sort: _ }, Some(value)) if body_is_ground => {
                        self.eval_finite_eq(&case.body, value, &env)?
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
                bool_bits[domain_start + bit.bit] = bool_expr;
                filled[bit.bit] = true;
            }
        }
        let cache_hits = *self.ground_model_eval_hits.borrow();
        let cache_misses = *self.ground_model_eval_misses.borrow();
        log::info!(
            "native constructor {} stats: bits={}, eval={:?}, ground_model_cache_hits={}, ground_model_cache_misses={}",
            cons.name,
            stats.bits,
            stats.eval_time,
            cache_hits,
            cache_misses,
        );
        Ok(bool_bits)
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
            "and" => self.eval_and(&items[1..], env),
            "or" => self.eval_or(&items[1..], env),
            "ite" if items.len() == 4 => match self.eval_bool(items[1], env)? {
                BoolExpr::Const(true) => self.eval_bool(items[2], env),
                BoolExpr::Const(false) => self.eval_bool(items[3], env),
                cond => Ok(BoolExpr::or(vec![
                    BoolExpr::and(vec![cond.clone(), self.eval_bool(items[2], env)?]),
                    BoolExpr::and(vec![cond.not(), self.eval_bool(items[3], env)?]),
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
        // Bail out cleanly if the guard isn't a compile-time constant.  The
        // caller then falls through to the general quantifier lowering
        // (eval_quantifier), which handles dynamic guards by enumerating
        // values and combining the body's BoolExpr per value.
        if self
            .const_guard_values(var, domain_expr, guard_expr, true, env)?
            .is_none()
        {
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
                "exists" => self.eval_and(body_items, &next)?,
                "forall" => self.eval_or(body_items, &next)?,
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

    fn eval_and(&self, items: &[&Value], env: &Env) -> Result<BoolExpr, String> {
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

    fn eval_or(&self, items: &[&Value], env: &Env) -> Result<BoolExpr, String> {
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
                let values = model_ref.domain_for_sort(&sort)
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
            disjuncts.push(BoolExpr::and(conjuncts));
        }
        Ok(BoolExpr::or(disjuncts))
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
        let values = model_ref.domain_for_sort(&sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let disjuncts = values
            .into_iter()
            .map(|name| {
                let value = AspectValue::Univ {
                    sort: sort.clone(),
                    name,
                };
                Ok(BoolExpr::and(vec![
                    self.eval_finite_eq(lhs, &value, env)?,
                    self.eval_finite_eq(rhs, &value, env)?,
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(BoolExpr::or(disjuncts))
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
                cond => Ok(BoolExpr::or(vec![
                    BoolExpr::and(vec![
                        cond.clone(),
                        self.eval_finite_eq(items[2], target, env)?,
                    ]),
                    BoolExpr::and(vec![
                        cond.not(),
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
            disjuncts.push(BoolExpr::and(conjuncts));
        }
        Ok(BoolExpr::or(disjuncts))
    }

    fn sort_width(&self, sort: &str) -> Result<usize, String> {
        Ok(self
            .ctx
            .domains_for_sort(sort)?
            .iter()
            .map(|domain| domain.width())
            .sum::<usize>()
            .max(1))
    }


    fn remapped_sort_alias_cmds(&self) -> Result<Vec<SyGuSCmd>, String> {
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

    fn remapped_aux_synth_fun_cmd(&self, aux_name: &str) -> Result<SyGuSCmd, String> {
        let sort = Sort::from_name(self.sig.start_sort());
        let sorted_var = SortedVar {
            name: "C".to_string(),
            sort: sort.clone(),
        };
        let constant_rule = SyGuSGTerm::Constant(sort.clone());
        let grammar = GrammarDef {
            sorted_vars: vec![sorted_var],
            grouped_rule_lists: vec![GroupedRuleList {
                symbol: "C".to_string(),
                sort: sort.clone(),
                terms: vec![constant_rule],
            }],
        };
        Ok(SyGuSCmd::SynthFun(
            aux_name.to_string(),
            Vec::new(),
            sort,
            Some(grammar),
        ))
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
        let value = model_ref.eval_symbols(func, arg_names).cloned();
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

fn extract_arg_bit(arg_idx: usize, bit: usize) -> SyGuSTerm {
    BitVec::extract(bit, bit, Term::var(&arg_name(arg_idx)))
}

fn bit_is_one(bit: SyGuSTerm) -> SyGuSTerm {
    Bool::eq(bit, BitVec::const_(1, 1))
}

/// Placeholder variable name used in `RecurrenceMode::SelfClosure` mode to
/// stand for "the Bool value of bit `i` of the synth-fun's output". The
/// post-pass in `compile()` resolves these into let-bindings.
fn self_bit_name(bit: usize) -> String {
    format!("{SELF_BIT_PREFIX}{bit}")
}

/// Walk a SyGuSTerm and collect bit indices `j` for any `__delis_self_b{j}`
/// identifier appearing in it. These are the self-bit dependencies of the
/// expression.
fn collect_self_bit_deps(term: &SyGuSTerm, out: &mut HashSet<usize>) {
    match term {
        SyGuSTerm::Identifier(id) => {
            if let Some(j) = parse_self_bit_index(id) {
                out.insert(j);
            }
        }
        SyGuSTerm::Application(_, args) => {
            for a in args {
                collect_self_bit_deps(a, out);
            }
        }
        SyGuSTerm::Annotated(inner, _) => collect_self_bit_deps(inner, out),
        SyGuSTerm::Let(bindings, body) => {
            for b in bindings {
                collect_self_bit_deps(&b.term, out);
            }
            collect_self_bit_deps(body, out);
        }
        SyGuSTerm::Forall(_, body) | SyGuSTerm::Exists(_, body) => {
            collect_self_bit_deps(body, out)
        }
        SyGuSTerm::Literal(_) => {}
    }
}

fn parse_self_bit_index(id: &Identifier) -> Option<usize> {
    match id {
        Identifier::Symbol(name) => name
            .strip_prefix(SELF_BIT_PREFIX)
            .and_then(|s| s.parse::<usize>().ok()),
        _ => None,
    }
}

/// Topological sort of bit dependencies. Returns bits in an order where
/// every dependency precedes its dependents; `Err` on cyclic graphs.
fn topo_sort_bits(deps: &[HashSet<usize>]) -> Result<Vec<usize>, String> {
    let n = deps.len();
    let mut in_deg = vec![0usize; n];
    let mut rev: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, ds) in deps.iter().enumerate() {
        for &j in ds {
            if j >= n {
                return Err(format!(
                    "self-bit dep out of range: bit {i} references {j} but only {n} bits exist"
                ));
            }
            in_deg[i] += 1;
            rev[j].push(i);
        }
    }
    let mut queue: std::collections::VecDeque<usize> =
        (0..n).filter(|&i| in_deg[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = queue.pop_front() {
        order.push(i);
        for &k in &rev[i] {
            in_deg[k] -= 1;
            if in_deg[k] == 0 {
                queue.push_back(k);
            }
        }
    }
    if order.len() != n {
        return Err("cyclic self-bit dependency".to_string());
    }
    Ok(order)
}

/// Build a constructor body that statically unrolls the LFP introduced by
/// `RecurrenceMode::SelfClosure`. If no bit has a self-dependency this falls
/// back to a plain `compact_bitvec`; otherwise it emits nested let-bindings
/// in topological order so each `__delis_self_b{i}` variable is defined
/// before any other binding (or the final body) references it.
impl BoolExpr {
    pub fn resolve_closure(bits: &[Self]) -> SyGuSTerm {
        let n = bits.len();
        let deps: Vec<HashSet<usize>> = bits
            .iter()
            .map(|expr| {
                let mut d = HashSet::default();
                if let BoolExpr::Dynamic(t) = expr {
                    collect_self_bit_deps(t, &mut d);
                }
                d
            })
            .collect();
        let has_self_refs = deps.iter().any(|d| !d.is_empty());
        if !has_self_refs {
            let bit_exprs: Vec<BoolExpr> = bits.iter().cloned().collect();
            return BoolExpr::compact(bit_exprs);
        }
        let order = match topo_sort_bits(&deps) {
            Ok(o) => o,
            Err(msg) => {
                log::warn!(
                    "self-closure body: {msg}; falling back to runtime self-references"
                );
                let bit_exprs: Vec<BoolExpr> = bits.iter().cloned().collect();
                return BoolExpr::compact(bit_exprs);
            }
        };
        // Determine which bits are referenced by other bits — only those
        // need `let`-bindings to avoid exponential subexpression duplication.
        let mut ref_count = vec![0usize; n];
        for d in &deps {
            for &j in d {
                ref_count[j] += 1;
            }
        }
        // Walk bits in topo order, building each as a BoolExpr. Bits that are
        // referenced more than once are materialized as let-bound names so
        // the surrounding term tree reuses them by reference, not by cloning.
        let mut resolved: Vec<BoolExpr> = vec![BoolExpr::Const(false); n];
        let mut bindings: Vec<(String, SyGuSTerm)> = Vec::new();
        for &i in &order {
            let computed = match &bits[i] {
                BoolExpr::Const(v) => BoolExpr::Const(*v),
                BoolExpr::Dynamic(t) => resolve_self_refs(t, &resolved),
            };
            // Only let-bind dynamic terms that are referenced more than once.
            // Constants and unreferenced terms are inlined as before.
            match computed {
                BoolExpr::Const(_) => resolved[i] = computed,
                BoolExpr::Dynamic(term) if ref_count[i] >= 2 => {
                    let name = format!("delis.share.b{i}");
                    bindings.push((name.clone(), term));
                    resolved[i] = BoolExpr::Dynamic(Term::var(&name));
                }
                expr => resolved[i] = expr,
            }
        }
        let body = BoolExpr::compact(resolved);
        if bindings.is_empty() {
            body
        } else {
            // Each let-binding references at most earlier let-vars (topo order),
            // so a single nested let chain — innermost binding outermost — works.
            // We emit one let with all bindings since they're sequential in topo
            // order and SyGuS `let` semantics allow this when no name shadowing
            // exists between siblings.
            let mut out = body;
            for (name, term) in bindings.into_iter().rev() {
                out = Term::let_binding(vec![(name, term)], out);
            }
            out
        }
    }
}

/// Resolve `__delis_self_b{j}` references in a SyGuSTerm by looking up
/// their already-resolved BoolExpr values. Boolean operators are combined
/// via BoolExpr::and/or/not which fold constants automatically.
/// Non-self-ref subterms (e.g., `(= ((_ extract k k) arg0) #b1)`) become
/// `BoolExpr::Dynamic(subterm)`.
fn resolve_self_refs(term: &SyGuSTerm, resolved: &[BoolExpr]) -> BoolExpr {
    match term {
        SyGuSTerm::Literal(Literal::Bool(v)) => BoolExpr::Const(*v),
        SyGuSTerm::Identifier(Identifier::Symbol(name)) => {
            if let Some(idx) = name.strip_prefix(SELF_BIT_PREFIX) {
                if let Ok(i) = idx.parse::<usize>() {
                    return resolved[i].clone();
                }
            }
            BoolExpr::Dynamic(term.clone())
        }
        SyGuSTerm::Application(id, args) => {
            let op = id.get_name();
            match op {
                "and" => BoolExpr::and(
                    args.iter().map(|a| resolve_self_refs(a, resolved)).collect(),
                ),
                "or" => BoolExpr::or(
                    args.iter().map(|a| resolve_self_refs(a, resolved)).collect(),
                ),
                "not" if args.len() == 1 => resolve_self_refs(&args[0], resolved).not(),
                _ => BoolExpr::Dynamic(term.clone()),
            }
        }
        _ => BoolExpr::Dynamic(term.clone()),
    }
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

fn analyze_recurrence(ctx: &CompileContext<'_>) -> Result<RecurrenceAnalysis, String> {
    let relations = ctx
        .semantic_program
        .semantics
        .iter()
        .map(|spec| spec.relation.clone())
        .collect::<Vec<_>>();
    let relation_set = relations.iter().cloned().collect::<HashSet<_>>();
    let mut locals = HashMap::<String, RelationLocal>::default();
    for spec in &ctx.semantic_program.semantics {
        let cases = crate::compile::semantic::parse_cases(&spec.body)?;
        let entry = locals.entry(spec.relation.clone()).or_default();
        entry.has_parent |= crate::compile::finite::contains_parent_navigation(&spec.body);
        for case in cases.values() {
            entry.self_recursive |= contains_self_call(&spec.relation, &case.self_var, &case.body);
            collect_relation_calls(&case.body, &relation_set, &mut entry.calls);
        }
    }

    let components = strongly_connected_components(&relations, &locals);
    let mut relation_modes = HashMap::default();
    let mut requires_aux = false;
    for component in components {
        let mut has_parent = false;
        let mut cyclic = component.len() > 1;
        for relation in &component {
            if let Some(local) = locals.get(relation) {
                has_parent |= local.has_parent;
                cyclic |= local.self_recursive;
            }
        }
        let mode = if has_parent {
            RecurrenceMode::ParentClosure
        } else if cyclic {
            RecurrenceMode::SelfClosure
        } else {
            RecurrenceMode::Direct
        };
        requires_aux |= mode == RecurrenceMode::ParentClosure;
        for relation in component {
            relation_modes.insert(relation, mode);
        }
    }
    let mut self_closure_sorts: HashSet<String> = HashSet::default();
    for spec in &ctx.semantic_program.semantics {
        if relation_modes.get(&spec.relation).copied() == Some(RecurrenceMode::SelfClosure) {
            self_closure_sorts.insert(spec.term_sort.clone());
        }
    }
    Ok(RecurrenceAnalysis {
        relation_modes,
        requires_aux,
        self_closure_sorts,
    })
}

fn contains_self_call(relation: &str, self_var: &str, expr: &lexpr::Value) -> bool {
    let Some(items) = expr.to_ref_vec() else {
        return false;
    };
    if items.first().and_then(|item| item.as_symbol()) == Some(relation)
        && items.get(1).and_then(|item| item.as_symbol()) == Some(self_var)
    {
        return true;
    }
    items
        .iter()
        .any(|item| contains_self_call(relation, self_var, item))
}

fn collect_relation_calls(expr: &lexpr::Value, relations: &HashSet<String>, out: &mut HashSet<String>) {
    let Some(items) = expr.to_ref_vec() else {
        return;
    };
    if let Some(symbol) = items.first().and_then(|item| item.as_symbol()) {
        if relations.contains(symbol) {
            out.insert(symbol.to_string());
        }
    }
    for item in items {
        collect_relation_calls(item, relations, out);
    }
}

fn strongly_connected_components(
    relations: &[String],
    locals: &HashMap<String, RelationLocal>,
) -> Vec<Vec<String>> {
    struct Tarjan<'a> {
        locals: &'a HashMap<String, RelationLocal>,
        index: usize,
        indices: HashMap<String, usize>,
        lowlink: HashMap<String, usize>,
        stack: Vec<String>,
        on_stack: HashSet<String>,
        components: Vec<Vec<String>>,
    }

    impl<'a> Tarjan<'a> {
        fn visit(&mut self, relation: &str) {
            self.indices.insert(relation.to_string(), self.index);
            self.lowlink.insert(relation.to_string(), self.index);
            self.index += 1;
            self.stack.push(relation.to_string());
            self.on_stack.insert(relation.to_string());

            if let Some(local) = self.locals.get(relation) {
                for next in &local.calls {
                    if !self.indices.contains_key(next) {
                        self.visit(next);
                        let next_low = self.lowlink[next];
                        let current = self.lowlink[relation];
                        self.lowlink.insert(relation.to_string(), current.min(next_low));
                    } else if self.on_stack.contains(next) {
                        let next_index = self.indices[next];
                        let current = self.lowlink[relation];
                        self.lowlink.insert(relation.to_string(), current.min(next_index));
                    }
                }
            }

            if self.lowlink[relation] == self.indices[relation] {
                let mut component = Vec::new();
                while let Some(node) = self.stack.pop() {
                    self.on_stack.remove(&node);
                    component.push(node.clone());
                    if node == relation {
                        break;
                    }
                }
                self.components.push(component);
            }
        }
    }

    let mut tarjan = Tarjan {
        locals,
        index: 0,
        indices: HashMap::default(),
        lowlink: HashMap::default(),
        stack: Vec::new(),
        on_stack: HashSet::default(),
        components: Vec::new(),
    };
    for relation in relations {
        if !tarjan.indices.contains_key(relation) {
            tarjan.visit(relation);
        }
    }
    tarjan.components
}
