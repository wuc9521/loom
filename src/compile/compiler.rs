use crate::fxmap::{HashMap, HashSet};
use crate::{
    compile::optim,
    compile::{
        bexpr::{arg_name, self_bit_name, BoolExpr},
        context::CompileContext,
        eval_ctx::{simple_sort_name, EnvView, SemanticContext},
        expr::sygus_term_to_value,
        semantics::{interped_bool, interped_value_matches, Case},
        syntax::{TermConstructor, TermSignature},
        types::{AspectKey, AspectValue, DomainBit, SemanticDomain, SemanticKind},
    },
    parser::{ast::TermTypes, LoomCmd, LoomProg},
};
use rustc_hash::FxHasher;
use std::{
    cell::{Ref, RefCell},
    hash::{Hash, Hasher as _},
    sync::Arc,
    time::{Duration, Instant},
};
use sygus_parser::SExpr as Value;
use sygus_parser::{
    BitVec, Bool, Cmd, GrammarDef, GroupedRuleList, Identifier, Literal, Sort, SortedVar, SyGuSCmd,
    SyGuSGTerm, SyGuSTerm, Term,
};

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
    /// Explicit model context established by a surrounding `(with-model M …)`.
    current_model: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecurrenceMode {
    Direct,
    SelfClosure,
}

struct RecurrenceAnalysis {
    relation_modes: HashMap<String, RecurrenceMode>,
    /// Term sorts whose semantic relation is in `SelfClosure` mode.
    self_closure_sorts: HashSet<String>,
}

#[derive(Default)]
struct RelationLocal {
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

    fn current_model(&self) -> Option<&str> {
        self.current_model.as_deref()
    }
}

struct Compiler<'a> {
    problem: &'a LoomProg,
    term_types: &'a TermTypes,
    sig: TermSignature,
    ctx: SemanticContext<'a>,
    recurrence: RecurrenceAnalysis,
    synth_name: String,
    synth_args: Vec<sygus_parser::SortedVar>,
    /// Bit positions keyed by `BitKey::fingerprint`.
    bit_index: HashMap<u64, usize>,
    bool_eval_cache: RefCell<HashMap<BoolMemoKey, BoolExpr>>,
    finite_eq_cache: RefCell<HashMap<ExprEnvMemoKey, BoolExpr>>,
    quantifier_values_cache: RefCell<HashMap<QuantifierValuesMemoKey, Vec<AspectValue>>>,
    guarded_projection_cache: RefCell<HashMap<GuardedProjectionMemoKey, Vec<usize>>>,
    value_eval_cache: RefCell<HashMap<ExprEnvMemoKey, AspectValue>>,
    int_eval_cache: RefCell<HashMap<ExprEnvMemoKey, isize>>,
    aspect_eval_cache: RefCell<HashMap<ExprEnvMemoKey, AspectKey>>,
    value_for_sort_cache: RefCell<HashMap<ExprEnvMemoKey, AspectValue>>,
    expr_symbol_cache: RefCell<HashMap<usize, Vec<String>>>,
    num_table_cache: RefCell<HashMap<(String, String), Option<NumTable>>>,
    optimize: bool,
}

#[derive(Default)]
struct ConstructorStats {
    bits: usize,
    eval_time: Duration,
}

/// Fingerprint of the env bindings an expression references; see
/// `memo_env_key`. A hash, not structure: these keys are built 10M+ times per
/// CTL instance, and the old structural key cloned a Vec<String> + AspectKey
/// per probe. Collisions silently reuse a memo entry — the same u64-FxHash
/// contract the eval_ctx caches already use.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct EnvId(u64);

impl EnvId {
    /// Fold one extra discriminator (relation name, sort name, target value)
    /// into the fingerprint so cache probes allocate no key field. Each cache
    /// folds exactly one kind of content, so distinct types can never
    /// cross-contaminate a map.
    fn mixing(self, extra: impl Hash) -> Self {
        let mut h = FxHasher::default();
        self.0.hash(&mut h);
        extra.hash(&mut h);
        Self(h.finish())
    }
}

/// Borrowed `(relation, aspect, value)` bit key. One type feeds both the
/// `bit_index` build and every probe through the same derived `Hash` stream,
/// so the two sides cannot drift. The u64 fingerprint carries the `EnvId`
/// collision contract (~6e-12 at 15K bits); the collision-free endgame is
/// interning, where these components become u32 ids.
#[derive(Hash)]
struct BitKey<'a> {
    relation: &'a str,
    aspect: &'a AspectKey,
    value: Option<&'a AspectValue>,
}

impl BitKey<'_> {
    fn fingerprint(&self) -> u64 {
        let mut h = FxHasher::default();
        self.hash(&mut h);
        h.finish()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct BoolMemoKey {
    expr_ptr: usize,
    env: EnvId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct QuantifierValuesMemoKey {
    expr_ptr: usize,
    env: EnvId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct GuardedProjectionMemoKey {
    relation: String,
    domain_expr_ptr: usize,
    guard_expr_ptr: usize,
    aspect_expr_ptr: usize,
    /// Env views of ALL THREE expressions: the cached bits depend on the
    /// domain enumeration, the guard decision, and the per-value aspect
    /// evaluation, each of which may read different aspect fields (notably
    /// `aspect_expr` reads `depth` while CTL guards do not). Keying by the
    /// guard's view alone over-shares across depth levels.
    env: EnvId,
    domain_env: EnvId,
    aspect_env: EnvId,
}


#[derive(Clone, PartialEq, Eq, Hash)]
struct ExprEnvMemoKey {
    expr_ptr: usize,
    env: EnvId,
}

impl<'a> CompileContext<'a> {
    /// Compile this context to SyGuS BV. Parent-free programs use the direct bit
    /// encoder; parent-bearing programs use the crossing-relation certificate
    /// encoder. `-O` tunes the direct encoder — the encoder choice follows the
    /// program, not the flag.
    pub fn compile(&self) -> Result<Vec<SyGuSCmd>, String> {
        if self.is_parent_free() {
            Compiler::from_context(self, super::opts().optimize)?.compile()
        } else {
            self.compile_parent()
        }
    }
}

impl<'a> Compiler<'a> {
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
                    // reference in any define-fun body).
                    Ok(Term::var(&self_bit_name(bit)))
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
                    BitKey {
                        relation: &domain.relation,
                        aspect: &bit.aspect,
                        value: bit.value.as_ref(),
                    }
                    .fingerprint(),
                    *offset + bit.bit,
                );
            }
            *offset += domain.width();
        }
        Ok(Self {
            problem: ctx_data.problem,
            term_types: ctx_data.term_types,
            sig,
            ctx,
            recurrence,
            synth_name: ctx_data.synth_name.clone(),
            synth_args: ctx_data.synth_args.clone(),
            bit_index,
            bool_eval_cache: RefCell::new(HashMap::default()),
            finite_eq_cache: RefCell::new(HashMap::default()),
            quantifier_values_cache: RefCell::new(HashMap::default()),
            guarded_projection_cache: RefCell::new(HashMap::default()),
            value_eval_cache: RefCell::new(HashMap::default()),
            int_eval_cache: RefCell::new(HashMap::default()),
            aspect_eval_cache: RefCell::new(HashMap::default()),
            value_for_sort_cache: RefCell::new(HashMap::default()),
            expr_symbol_cache: RefCell::new(HashMap::default()),
            num_table_cache: RefCell::new(HashMap::default()),
            optimize,
        })
    }

    fn compile(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        let grammar_start = Instant::now();
        let grammar = self.grammar()?;
        log::debug!("grammar built in {:?}", grammar_start.elapsed());

        // Phase 1: evaluate all constructor bits
        struct ConsResult {
            name: String,
            result_sort: String,
            arg_sorts: Vec<String>,
            bool_bits: Vec<BoolExpr>,
        }
        let mut cons_results = Vec::new();
        for cons in &self.sig.constructors {
            // A parameter-leaf constructor IS the synth-fun's formal BV
            // parameter (a bare grammar terminal), not a folded function, so it
            // gets no define-fun.
            if self.is_param_leaf(&cons.name) {
                continue;
            }
            let cons_start = Instant::now();
            let bool_bits = self.eval_constructor_bits(cons)?;
            log::debug!(
                "constructor {}: compiled in {:?}",
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
        let emit_start = Instant::now();
        let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];
        cmds.extend(self.remapped_sort_alias_cmds()?);
        let mut total_lift = Duration::default();
        let mut total_compact = Duration::default();
        let mut total_closure = Duration::default();
        for cr in &cons_results {
            let remapped_bits: Vec<BoolExpr> = cr.bool_bits.clone();
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
                // Constructor body may reference __loom_self_b{j} placeholders
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
            log::debug!(
                "emit {}: lift={:?} body={:?}",
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
        log::debug!(
            "emit totals: lift={:?} compact={:?} closure={:?} elapsed={:?}",
            total_lift,
            total_compact,
            total_closure,
            emit_start.elapsed()
        );
        let mut synth_cmd = Cmd::synth_fun(&self.synth_name)
            .ret(Sort::from_name(self.sig.start_sort()))
            .grammar(grammar);
        for arg in self.synth_args.clone() {
            synth_cmd = synth_cmd.arg(&arg.name, arg.sort);
        }
        cmds.push(synth_cmd.build());
        let env = Env::default();
        let lower_start = Instant::now();
        let top_level_constraints = self
            .problem
            .cmds
            .iter()
            .filter_map(|cmd| match cmd {
                LoomCmd::As(SyGuSCmd::Constraint(term)) => Some(term),
                LoomCmd::DeclareAxiom(_) => None,
                _ => None,
            })
            .map(sygus_term_to_value)
            .collect::<Result<Vec<_>, _>>()?;
        log::debug!(
            "top-level constraint lowering (sygus_term_to_value x{}): {:?}",
            top_level_constraints.len(),
            lower_start.elapsed(),
        );
        let constraints_start = Instant::now();
        for term in &top_level_constraints {
            let lowered = self.lower_top_level_constraint(term, &env)?;
            cmds.push(Cmd::constraint().body(lowered).build());
        }
        log::debug!(
            "top-level constraints emitted in {:?}",
            constraints_start.elapsed()
        );
        cmds.push(SyGuSCmd::CheckSynth);
        Ok(cmds)
    }

    /// True iff any semantic relation operating on `sort` is in
    /// `RecurrenceMode::SelfClosure` mode. Constructors of such sorts produce
    /// bool_bits that contain `__loom_self_b{j}` placeholders, which need
    /// the `build_self_closure_body` post-pass to be wrapped in let-bindings.
    fn is_self_closure_sort(&self, sort: &str) -> bool {
        self.recurrence.self_closure_sorts.contains(sort)
    }

    fn lower_top_level_constraint(&self, term: &Value, env: &Env) -> Result<SyGuSTerm, String> {
        self.lower_constraint(term, env)
    }

    // --- Unified structural constraint lowering ---
    //
    // Constraints are Bool formulas referencing the synth-fun via semantic
    // relation calls. We walk the formula AST structurally, producing a
    // SyGuSTerm directly. Semantic relation calls on the synth target become
    // bit extractions from the synth-fun's BV (variable or application).

    fn lower_constraint(&self, term: &Value, env: &Env) -> Result<SyGuSTerm, String> {
        if let Some(symbol) = term.as_symbol() {
            return match symbol {
                "true" => Ok(SyGuSTerm::Literal(Literal::Bool(true))),
                "false" => Ok(SyGuSTerm::Literal(Literal::Bool(false))),
                _ => Err(format!("unsupported constraint symbol: {symbol}")),
            };
        }
        let Some(items) = term.to_ref_vec() else {
            return Err(format!("expected constraint expression, got {term}"));
        };
        let op = items
            .first()
            .and_then(|item| item.as_symbol())
            .ok_or_else(|| format!("constraint operator must be a symbol: {term}"))?;
        match op {
            "not" if items.len() == 2 => Ok(Bool::not(self.lower_constraint(items[1], env)?)),
            "and" => {
                let conjuncts = items[1..]
                    .iter()
                    .map(|item| self.lower_constraint(item, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Bool::and(conjuncts))
            }
            "or" => {
                let disjuncts = items[1..]
                    .iter()
                    .map(|item| self.lower_constraint(item, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Bool::or(disjuncts))
            }
            "with-model" if items.len() == 3 => {
                let model = items[1]
                    .as_symbol()
                    .ok_or_else(|| format!("with-model expects a model symbol: {term}"))?
                    .to_string();
                let mut inner_env = env.clone();
                inner_env.current_model = Some(model);
                self.lower_constraint(items[2], &inner_env)
            }
            "=" if items.len() == 3 => self.lower_equality(items[1], items[2], env),
            relation if self.is_synth_relation_call(relation, &items[1..]) => {
                self.lower_synth_relation(relation, &items[1..], env)
            }
            _ => Err(format!("unsupported constraint expression: {term}")),
        }
    }

    /// True if `(op args...)` is a semantic relation call whose term argument
    /// is the synth target.
    fn is_synth_relation_call(&self, op: &str, args: &[&Value]) -> bool {
        if !self.ctx.has_relation(op) {
            return false;
        }
        args.first()
            .and_then(|a| a.as_symbol())
            .map_or(false, |sym| sym == self.synth_name)
    }

    /// Lower a Bool semantic relation call `(Rel synthName asp)` to a bit
    /// extraction: `(= ((_ extract b b) bv_ref) #b1)`.
    fn lower_synth_relation(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<SyGuSTerm, String> {
        if args.len() != 2 {
            return Err(format!(
                "semantic relation {relation} in constraint expects 2 args (term, aspect)"
            ));
        }
        let aspect_expr = args[1];
        let model = self.resolve_constraint_model(relation, aspect_expr, env)?;
        let seed_env = self.constraint_model_env(&model);
        let aspect =
            self.strip_param_fields(&self.cached_eval_aspect(aspect_expr, relation, &seed_env)?);
        let bit = self.bit_for(relation, &aspect, None)?;
        let bv_ref = self.synth_bv_ref(&model)?;
        Ok(Bool::eq(
            BitVec::extract(bit, bit, bv_ref),
            BitVec::const_(1, 1),
        ))
    }

    /// Lower `(= lhs rhs)` in a constraint. One side is a semantic relation
    /// call on the synth target; the other is a ground value expression.
    fn lower_equality(&self, lhs: &Value, rhs: &Value, env: &Env) -> Result<SyGuSTerm, String> {
        let (call, out_expr) = if self.is_value_synth_call(lhs) {
            (lhs, rhs)
        } else if self.is_value_synth_call(rhs) {
            (rhs, lhs)
        } else {
            return Err(format!(
                "constraint equality must have a semantic relation call on the synth target: \
                 (= {lhs} {rhs})"
            ));
        };
        let call_items = call
            .to_ref_vec()
            .ok_or_else(|| format!("call side must be a list: {call}"))?;
        let relation = call_items[0]
            .as_symbol()
            .ok_or_else(|| format!("relation must be a symbol: {}", call_items[0]))?;
        let aspect_expr = call_items[2];
        let model = self.resolve_constraint_model(relation, aspect_expr, env)?;
        let seed_env = self.constraint_model_env(&model);
        let out_value = self.cached_eval_value(out_expr, &seed_env)?;
        let aspect =
            self.strip_param_fields(&self.cached_eval_aspect(aspect_expr, relation, &seed_env)?);
        let bit = self.bit_for(relation, &aspect, Some(&out_value))?;
        let bv_ref = self.synth_bv_ref(&model)?;
        Ok(Bool::eq(
            BitVec::extract(bit, bit, bv_ref),
            BitVec::const_(1, 1),
        ))
    }

    /// True if `expr` is `(Rel synthName asp)` where Rel is a FiniteValue relation.
    fn is_value_synth_call(&self, expr: &Value) -> bool {
        let Some(items) = expr.to_ref_vec() else {
            return false;
        };
        if items.len() != 3 {
            return false;
        }
        let Some(relation) = items[0].as_symbol() else {
            return false;
        };
        if items[1].as_symbol() != Some(self.synth_name.as_str()) {
            return false;
        }
        matches!(
            self.ctx.domain_for_relation(relation),
            Ok(domain) if matches!(domain.kind, SemanticKind::FiniteValue { .. })
        )
    }

    /// Build the SyGuS term referencing the synth-fun's BV output.
    /// Nullary: `Term::var(synth_name)`.
    /// Parametric: `Application(synth_name, param_bv_consts)`.
    fn synth_bv_ref(&self, model: &str) -> Result<SyGuSTerm, String> {
        if self.synth_args.is_empty() {
            return Ok(Term::var(&self.synth_name));
        }
        let mut bind_args = Vec::new();
        for param in &self.synth_args {
            bind_args.push(self.param_bv_const(&param.name, model)?);
        }
        Ok(SyGuSTerm::Application(
            Identifier::Symbol(self.synth_name.clone()),
            bind_args,
        ))
    }

    /// Remove synth-parameter-named fields from an AspectKey. These fields
    /// were excluded from aspect enumeration, so the bit_index only contains
    /// entries without them.
    fn strip_param_fields(&self, aspect: &AspectKey) -> AspectKey {
        if self.synth_args.is_empty() {
            return aspect.clone();
        }
        AspectKey {
            relation: aspect.relation.clone(),
            model: aspect.model.clone(),
            fields: aspect
                .fields
                .iter()
                .filter(|(name, _)| !self.synth_args.iter().any(|a| a.name == *name))
                .cloned()
                .collect(),
        }
    }

    /// Resolve the model name for a constraint's aspect expression.  If the
    /// aspect space declares a `(model Model)` field, the model is taken from
    /// the aspect arguments (old style).  Otherwise it must come from the
    /// surrounding `(with-model M …)` context (new style).
    fn resolve_constraint_model(
        &self,
        relation: &str,
        aspect_expr: &Value,
        env: &Env,
    ) -> Result<String, String> {
        let space = {
            let domain = self.ctx.domain_for_relation(relation)?;
            self.ctx.aspect_space(&domain.aspect_space)?
        };
        let has_model_field = space
            .fields
            .iter()
            .any(|field| field.domain.as_sort() == Some("Model"));
        if has_model_field {
            let aspect_args = aspect_expr
                .to_ref_vec()
                .ok_or_else(|| format!("aspect expression must be a list: {aspect_expr}"))?;
            for (field, arg) in space.fields.iter().zip(aspect_args.iter().skip(1)) {
                if field.domain.as_sort() == Some("Model") {
                    if let Some(name) = arg.as_symbol() {
                        return Ok(name.to_string());
                    }
                }
            }
            return Err(format!(
                "aspect expression has no Model argument: {aspect_expr}"
            ));
        }
        env.current_model()
            .map(str::to_string)
            .ok_or_else(|| format!("aspect {aspect_expr} requires a (with-model M …) context"))
    }

    /// An `Env` seeded with a model, so `current_model()` resolves during
    /// evaluation of ground sub-expressions in constraints.
    fn constraint_model_env(&self, model: &str) -> Env {
        let mut env = Env::default();
        env.current_model = Some(model.to_string());
        env
    }

    /// Compute the BV constant for a synth parameter in a specific model.
    /// Evaluates the parameter's constructor semantics across all domains of
    /// the start sort, collecting each bit's ground truth for that model.
    fn param_bv_const(&self, param_name: &str, model: &str) -> Result<SyGuSTerm, String> {
        let start = self.sig.start_sort();
        let width = self.sort_width(start)?;
        let mut bits_vec = vec![false; width];
        let mut offset = 0usize;
        for domain in self.ctx.domains_for_sort(start)? {
            let spec = self.ctx.spec_for_relation(&domain.relation)?;
            let cases =
                self.ctx.cases().get(&spec.relation).ok_or_else(|| {
                    format!("missing semantic cases for relation {}", spec.relation)
                })?;
            let case = cases.get(param_name).ok_or_else(|| {
                format!(
                    "missing semantic case for param constructor {} in {}",
                    param_name, spec.relation
                )
            })?;
            let mut env = Env::default();
            env.relation = Some(spec.relation.clone());
            env.self_var = Some(case.self_var.clone());
            let domain_width = domain.width();
            for bit in &domain.bits {
                if bit.aspect.model != model {
                    continue;
                }
                env.aspect
                    .insert(case.aspect_var.clone(), bit.aspect.clone());
                let val = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) => self.eval_bool(&case.body, &env)?,
                    (SemanticKind::FiniteValue { .. }, Some(value)) => {
                        self.eval_finite_eq(&case.body, value, &env)?
                    }
                    _ => BoolExpr::Const(false),
                };
                if matches!(val, BoolExpr::Const(true)) && bit.bit < domain_width {
                    bits_vec[offset + bit.bit] = true;
                }
            }
            offset += domain_width;
        }
        let bv_str: String = (0..width)
            .rev()
            .map(|i| if bits_vec[i] { '1' } else { '0' })
            .collect();
        Ok(SyGuSTerm::Literal(Literal::BinConst(bv_str)))
    }

    /// True if `name` is a variable-leaf constructor: it shares its name with a
    /// synth-fun parameter, so it compiles to the bare grammar terminal (the
    /// formal parameter, automatically available in the SyGuS grammar) rather
    /// than a folded define-fun.
    fn is_param_leaf(&self, name: &str) -> bool {
        self.synth_args.iter().any(|a| a.name == name)
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
                env.aspect
                    .insert(case.aspect_var.clone(), bit.aspect.clone());
                let eval_start = Instant::now();
                let bool_expr = match (&domain.kind, &bit.value) {
                    (SemanticKind::Bool, None) if body_is_ground => {
                        self.eval_bool(&case.body, &env)?
                    }
                    (SemanticKind::FiniteValue { sort: _ }, Some(value)) if body_is_ground => self
                        .eval_finite_eq(&case.body, value, &env)
                        .map_err(|err| {
                            self.constructor_eval_error(cons, domain, bit, case, &err)
                        })?,
                    (SemanticKind::Bool, None) => self.eval_bool(&case.body, &env)?,
                    (SemanticKind::FiniteValue { sort: _ }, Some(value)) => self
                        .eval_finite_eq(&case.body, value, &env)
                        .map_err(|err| {
                            self.constructor_eval_error(cons, domain, bit, case, &err)
                        })?,
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
            // One-hot exactness: a closed (ground) value term must denote
            // exactly one element per model. This holds iff the model functions
            // it reads are total and functional. Verify it for ground
            // FiniteValue domains so a malformed example surfaces as an error
            // rather than as a silently wrong BV.
            if body_is_ground && matches!(domain.kind, SemanticKind::FiniteValue { .. }) {
                let mut per_aspect: HashMap<AspectKey, usize> = HashMap::default();
                for bit in &domain.bits {
                    let hot = matches!(bool_bits[domain_start + bit.bit], BoolExpr::Const(true));
                    *per_aspect.entry(bit.aspect.clone()).or_insert(0) += usize::from(hot);
                }
                for (aspect, hot) in &per_aspect {
                    if *hot != 1 {
                        return Err(format!(
                            "ground value term {} under {} denotes {} elements (expected exactly 1) \
                             in model {}; check that its model functions are total and functional",
                            cons.name, domain.relation, hot, aspect.model
                        ));
                    }
                }
            }
        }
        log::debug!(
            "constructor {}: bits={} eval={:?} cache hits={}/{}",
            cons.name,
            stats.bits,
            stats.eval_time,
            self.ctx.eval_cache_hits(),
            self.ctx.eval_cache_hits() + self.ctx.eval_cache_misses(),
        );
        Ok(bool_bits)
    }

    fn constructor_eval_error(
        &self,
        cons: &TermConstructor,
        domain: &SemanticDomain,
        bit: &DomainBit,
        case: &Case,
        err: &str,
    ) -> String {
        format!(
            "while compiling constructor {} for relation {} ({:?}), bit {} target {:?}, body {}: {}",
            cons.name, domain.relation, domain.kind, bit.bit, bit.value, case.body, err
        )
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
                if let Some(result) =
                    self.lower_guarded_projection("exists", var, domain_expr, guard, rest[0], env)?
                {
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
                if let Some(result) =
                    self.lower_guarded_projection("forall", var, domain_expr, guard, rest[0], env)?
                {
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
        let term_expr = items[1];
        let aspect_expr = items[2];
        // ponytail: bits lookup doubles as the constant-guard probe so a memo
        // hit never re-enumerates the domain; miss path enumerates once.
        let Some(bits) =
            self.guarded_projection_bits(var, domain_expr, guard_expr, relation, aspect_expr, env)?
        else {
            return Ok(None);
        };
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
    ) -> Result<Option<Vec<usize>>, String> {
        let key = GuardedProjectionMemoKey {
            relation: relation.to_string(),
            domain_expr_ptr: domain_expr as *const Value as usize,
            guard_expr_ptr: guard_expr as *const Value as usize,
            aspect_expr_ptr: aspect_expr as *const Value as usize,
            env: self.memo_env_key(guard_expr, env),
            domain_env: self.memo_env_key(domain_expr, env),
            aspect_env: self.memo_env_key(aspect_expr, env),
        };
        if let Some(cached) = self.guarded_projection_cache.borrow().get(&key) {
            return Ok(Some(cached.clone()));
        }
        let Some(values) = self.const_guard_values(var, domain_expr, guard_expr, true, env)? else {
            return Ok(None);
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
        Ok(Some(bits))
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
        let result = if let Some(items) = domain_expr.to_ref_vec() {
            match items.as_slice() {
                [domain, lo, hi] if domain.as_symbol() == Some("int-range") => {
                    let lo = self.ctx.eval_int_expr(lo, env)?;
                    let hi = self.cached_eval_int_expr(hi, env)?;
                    if lo < 0 || hi < lo {
                        Vec::new()
                    } else {
                        (lo as usize..=hi as usize).map(AspectValue::Int).collect()
                    }
                }
                _ => self.univ_values_for_expr(domain_expr, env)?,
            }
        } else {
            self.univ_values_for_expr(domain_expr, env)?
        };
        self.quantifier_values_cache
            .borrow_mut()
            .insert(memo_key, result.clone());
        Ok(result)
    }

    fn univ_values_for_expr(
        &self,
        domain_expr: &Value,
        env: &Env,
    ) -> Result<Vec<AspectValue>, String> {
        let (model, sort) = self.ctx.parse_univ_expr(domain_expr, env)?;
        let model_ref = self.ctx.model(&model)?;
        let values = model_ref
            .univ_for_sort(&sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let sort_arc: Arc<str> = Arc::from(sort.as_str());
        Ok(values
            .into_iter()
            .map(|name| AspectValue::Univ {
                sort: sort_arc.clone(),
                name: Arc::from(name),
            })
            .collect())
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
        if sig.ret_sort.as_simple_name() != Some("Bool") {
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
            let value = self.ctx.eval_model_symbols(model_ref, func, &arg_names);
            return Ok(BoolExpr::Const(value.as_ref().is_some_and(interped_bool)));
        }
        if let Some(native) = self.try_native_compare(func, args, env)? {
            return Ok(native);
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
        let values = model_ref
            .univ_for_sort(&sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let sort_arc: Arc<str> = Arc::from(sort.as_str());
        let disjuncts = values
            .into_iter()
            .map(|name| {
                let value = AspectValue::Univ {
                    sort: sort_arc.clone(),
                    name: Arc::from(name),
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
        if let (Ok(lhs_value), Ok(rhs_value)) = (
            self.cached_eval_value(lhs, env),
            self.cached_eval_value(rhs, env),
        ) {
            return Ok(BoolExpr::Const(
                self.ctx.values_equal(&lhs_value, &rhs_value, env)?,
            ));
        }
        if self.cached_eval_int_expr(lhs, env).is_ok()
            && self.cached_eval_int_expr(rhs, env).is_ok()
        {
            return self.eval_int_compare(lhs, rhs, env, |l, r| l == r);
        }
        if let Some(native) = self.try_native_equality(lhs, rhs, env)? {
            return Ok(native);
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
        let memo_key = ExprEnvMemoKey {
            expr_ptr: expr as *const Value as usize,
            // target folded into the fingerprint: no clone per probe.
            env: self.memo_env_key(expr, env).mixing(target),
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
            // Reaching here means the symbol is not a bound value, a declared
            // constant, or an enum/model token. eval_value's discipline rejects
            // it (e.g. a bare domain element named directly in a body); surface
            // that error rather than string-matching the element name.
            return Err(self
                .cached_eval_value(expr, env)
                .err()
                .unwrap_or_else(|| format!("unsupported finite symbol {symbol}")));
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

    fn memo_env_key(&self, expr: &Value, env: &Env) -> EnvId {
        let referenced = self.expr_symbols(expr);
        let mut h = FxHasher::default();
        let mut captured_aspect = false;
        for name in referenced.iter() {
            // The symbol-cache entry outlives the Compiler, so the name's data
            // pointer is a per-name constant here and distinct names have
            // distinct addresses — keys discriminate exactly as the byte hash
            // did, without re-walking the string 10M+ times.
            let name_id = name.as_ptr() as usize;
            if let Some(aspect) = env.aspect.get(name) {
                captured_aspect = true;
                // Whole aspect, every field: bodies pass the binder wholesale
                // to Sem/Asp constructors (`(CTL.Sem phi asp)`), which resolve
                // it to a bit via fields no projection ever names syntactically.
                name_id.hash(&mut h);
                aspect.hash(&mut h);
            }
            if let Some(value) = env.values.get(name) {
                name_id.hash(&mut h);
                value.hash(&mut h);
            }
        }
        // The model must be part of the key whenever it isn't already implied:
        // model-function and relation calls depend on it even when `expr`
        // references no aspect binder (e.g. `(out.chr $0)` with literal args).
        // If an aspect was captured its `.model` is already hashed above (the
        // env holds a single aspect during constructor eval).
        if !captured_aspect {
            env.aspect.values().next().map(|a| a.model.hash(&mut h));
        }
        env.current_model.hash(&mut h);
        EnvId(h.finish())
    }

    fn expr_symbols(&self, expr: &Value) -> Ref<'_, Vec<String>> {
        let expr_ptr = expr as *const Value as usize;
        if !self.expr_symbol_cache.borrow().contains_key(&expr_ptr) {
            let mut symbols = HashSet::default();
            collect_expr_symbols(expr, &mut symbols);
            let mut symbols = symbols.into_iter().collect::<Vec<_>>();
            symbols.sort();
            self.expr_symbol_cache.borrow_mut().insert(expr_ptr, symbols);
        }
        Ref::map(self.expr_symbol_cache.borrow(), |cache| &cache[&expr_ptr])
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
            let value = self.ctx.eval_model_symbols(model_ref, func, &arg_names);
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
        let mut cmds = self
            .sig
            .sort_order()
            .iter()
            .map(|sort| {
                Ok(SyGuSCmd::DefineSort(
                    sort.clone(),
                    Vec::new(),
                    Sort::bitvec(self.sort_width(sort)?),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        // Synth-fun parameter sorts (e.g. `U`) are declared sorts, not term-type
        // sorts, so they are not in sort_order. A parameter ranges over the same
        // per-model universe blocks as the value sort it feeds, so it gets the
        // same BV width as the start value sort. Emit one alias per distinct
        // parameter sort not already aliased.
        let mut seen: HashSet<String> = self.sig.sort_order().iter().cloned().collect();
        let param_width = self.sort_width(self.sig.start_sort())?;
        for arg in &self.synth_args {
            let sort_name = arg.sort.to_string();
            if seen.insert(sort_name.clone()) {
                cmds.push(SyGuSCmd::DefineSort(
                    sort_name,
                    Vec::new(),
                    Sort::bitvec(param_width),
                ));
            }
        }
        Ok(cmds)
    }

    fn bit_for(
        &self,
        relation: &str,
        aspect: &AspectKey,
        value: Option<&AspectValue>,
    ) -> Result<usize, String> {
        self.bit_index
            .get(&BitKey { relation, aspect, value }.fingerprint())
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
                let sort = simple_sort_name(arg.1);
                self.cached_eval_value_for_sort(arg.0, &sort, env)
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
        self.value_eval_cache
            .borrow_mut()
            .insert(key, value.clone());
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
        let key = ExprEnvMemoKey {
            expr_ptr: expr as *const Value as usize,
            // relation folded into the fingerprint: no String alloc per probe.
            env: self.memo_env_key(expr, env).mixing(relation),
        };
        if let Some(cached) = self.aspect_eval_cache.borrow().get(&key) {
            return Ok(cached.clone());
        }
        let value = self.ctx.eval_aspect(expr, relation, env)?;
        self.aspect_eval_cache
            .borrow_mut()
            .insert(key, value.clone());
        Ok(value)
    }

    fn cached_eval_value_for_sort(
        &self,
        expr: &Value,
        sort: &str,
        env: &Env,
    ) -> Result<AspectValue, String> {
        let key = ExprEnvMemoKey {
            expr_ptr: expr as *const Value as usize,
            // sort folded into the fingerprint: no String alloc per probe.
            env: self.memo_env_key(expr, env).mixing(sort),
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
        let cases = crate::compile::semantics::parse_cases(&spec.body, &spec.aspect_var)?;
        let entry = locals.entry(spec.relation.clone()).or_default();
        for case in cases.values() {
            entry.self_recursive |= contains_self_call(&relation_set, &case.self_var, &case.body);
            collect_relation_calls(&case.body, &relation_set, &mut entry.calls);
        }
    }

    let components = strongly_connected_components(&relations, &locals);
    let mut relation_modes = HashMap::default();
    for component in components {
        let mut cyclic = component.len() > 1;
        for relation in &component {
            if let Some(local) = locals.get(relation) {
                cyclic |= local.self_recursive;
            }
        }
        let mode = if cyclic {
            RecurrenceMode::SelfClosure
        } else {
            RecurrenceMode::Direct
        };
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
        self_closure_sorts,
    })
}

/// True if the body calls ANY semantic relation on the matched term itself
/// (not a child). Same-relation self calls (Reg's `re.*`) and cross-relation
/// same-node calls (e.g. `Str.Eq` consulting `Str.OccAt e`) both require the
/// SelfClosure encoding: their bits depend on sibling bits of the same node,
/// which `resolve_closure` unrolls via topo-sorted let-bindings.
fn contains_self_call(
    relations: &HashSet<String>,
    self_var: &str,
    expr: &sygus_parser::SExpr,
) -> bool {
    let Some(items) = expr.to_ref_vec() else {
        return false;
    };
    if items
        .first()
        .and_then(|item| item.as_symbol())
        .is_some_and(|symbol| relations.contains(symbol))
        && items.get(1).and_then(|item| item.as_symbol()) == Some(self_var)
    {
        return true;
    }
    items
        .iter()
        .any(|item| contains_self_call(relations, self_var, item))
}

fn collect_relation_calls(
    expr: &sygus_parser::SExpr,
    relations: &HashSet<String>,
    out: &mut HashSet<String>,
) {
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
                        self.lowlink
                            .insert(relation.to_string(), current.min(next_low));
                    } else if self.on_stack.contains(next) {
                        let next_index = self.indices[next];
                        let current = self.lowlink[relation];
                        self.lowlink
                            .insert(relation.to_string(), current.min(next_index));
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

// ===========================================================================
// Native-BV templates for numeric finite-universe tables.
//
// See docs/native-bv-design.md. A 2-ary model table that is exactly `<=`,
// `<`, "`<` with the trailing invalid element acting as +infinity on the
// right", saturating `+`, or saturating `-` over the *listed* universe order
// is recognized here, and call sites over one-hot value fields emit
// O(|U|)-size native BV formulas instead of O(|U|^2..3) element expansions.
// Recognition validates the entire table per model, so emission is exact;
// anything unrecognized falls back to the existing expansion. Disable with
// LOOM_NO_NATIVE_TABLES=1 for differential testing.
// ===========================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NumTableKind {
    Ule,
    Ult,
    /// `a < b`, except a valid `a` compared with the invalid element is true
    /// (the generator's `ltl.pos`: invalid length means "longer than B").
    UltInvBig,
    AddSat,
    SubSat,
}

#[derive(Clone, Debug)]
struct NumTable {
    kind: NumTableKind,
    arg_sort: String,
    /// Index (in listed universe order) of the invalid/saturation element.
    inv_idx: Option<usize>,
}

#[derive(Clone)]
enum FieldBits {
    /// Statically-known value (universe index), or no value.
    Const(Option<usize>),
    /// One 1-bit Bool atom per universe value, in listed order.
    Dyn(Vec<SyGuSTerm>),
}

#[derive(Clone)]
struct OneHotField {
    bits: FieldBits,
    width: usize,
    inv_idx: Option<usize>,
}

impl OneHotField {
    /// Bool atom: the field's value is universe index `i` (false if out of range).
    fn bit(&self, i: usize) -> SyGuSTerm {
        if i >= self.width {
            return bool_lit(false);
        }
        match &self.bits {
            FieldBits::Const(v) => bool_lit(*v == Some(i)),
            FieldBits::Dyn(bits) => bits[i].clone(),
        }
    }
    fn valid_indices(&self) -> Vec<usize> {
        (0..self.width)
            .filter(|i| Some(*i) != self.inv_idx)
            .collect()
    }
    /// Bool term: the field holds a valid (non-invalid) value.
    fn valid_term(&self) -> SyGuSTerm {
        or_terms(self.valid_indices().iter().map(|i| self.bit(*i)).collect())
    }
    /// Bool term: the field holds exactly the invalid element.
    fn invalid_term(&self) -> SyGuSTerm {
        match self.inv_idx {
            None => bool_lit(false),
            Some(i) => self.bit(i),
        }
    }
    /// Bool term: the field holds some value (valid or invalid).
    fn any_term(&self) -> SyGuSTerm {
        or_terms((0..self.width).map(|i| self.bit(i)).collect())
    }
    /// Bool term: value >= i (valid values only), i.e. OR of bits i.. —
    /// emitted as a suffix-OR chain so the circuit planner shares the tail.
    fn suffix_or(&self, i: isize) -> SyGuSTerm {
        let lo = i.max(0) as usize;
        let idxs: Vec<usize> = self
            .valid_indices()
            .into_iter()
            .filter(|v| *v >= lo)
            .collect();
        // Nested (or b_k (or b_{k+1} ...)) — structurally shared suffixes.
        let mut acc = bool_lit(false);
        for v in idxs.into_iter().rev() {
            let b = self.bit(v);
            acc = if acc == bool_lit(false) {
                b
            } else {
                or_terms(vec![b, acc])
            };
        }
        acc
    }
}

/// A numeric side restricted to the shapes the generated rules produce:
/// one dynamic one-hot field plus a constant offset (constants and
/// saturating +/- against constants fold into the offset).
struct NumSide {
    field: Option<OneHotField>,
    offset: isize,
    leaves: Vec<OneHotField>,
    /// Bool terms required for the side to evaluate to a valid element
    /// (per-node saturation guards; leaf validity tracked via `leaves`).
    guards: Vec<SyGuSTerm>,
}

impl NumSide {
    /// Bool atom: side value == universe index v (assumes all-valid context).
    fn value_is(&self, v: isize) -> SyGuSTerm {
        match &self.field {
            None => bool_lit(v == self.offset),
            Some(f) => {
                let idx = v - self.offset;
                if idx < 0 {
                    bool_lit(false)
                } else {
                    f.bit(idx as usize)
                }
            }
        }
    }
    /// Bool term: side value >= v (assumes all-valid context).
    fn value_ge(&self, v: isize) -> SyGuSTerm {
        match &self.field {
            None => bool_lit(self.offset >= v),
            Some(f) => f.suffix_or(v - self.offset),
        }
    }
    /// Enumerate (index, atom) pairs for possible side values (valid context).
    fn value_atoms(&self, max_valid: isize) -> Vec<(isize, SyGuSTerm)> {
        match &self.field {
            None => {
                if self.offset >= 0 && self.offset <= max_valid {
                    vec![(self.offset, bool_lit(true))]
                } else {
                    vec![]
                }
            }
            Some(f) => f
                .valid_indices()
                .into_iter()
                .map(|i| (i as isize + self.offset, f.bit(i)))
                .filter(|(v, _)| *v >= 0 && *v <= max_valid)
                .collect(),
        }
    }
}

fn bool_lit(value: bool) -> SyGuSTerm {
    SyGuSTerm::Literal(Literal::Bool(value))
}

fn and_terms(mut terms: Vec<SyGuSTerm>) -> SyGuSTerm {
    terms.retain(|t| *t != bool_lit(true));
    if terms.iter().any(|t| *t == bool_lit(false)) {
        return bool_lit(false);
    }
    match terms.len() {
        0 => bool_lit(true),
        1 => terms.pop().unwrap(),
        _ => Bool::and(terms),
    }
}

fn or_terms(mut terms: Vec<SyGuSTerm>) -> SyGuSTerm {
    terms.retain(|t| *t != bool_lit(false));
    if terms.iter().any(|t| *t == bool_lit(true)) {
        return bool_lit(true);
    }
    match terms.len() {
        0 => bool_lit(false),
        1 => terms.pop().unwrap(),
        _ => Bool::or(terms),
    }
}

impl<'a> Compiler<'a> {
    fn numeric_table(&self, model_name: &str, func: &str) -> Option<NumTable> {
        let key = (model_name.to_string(), func.to_string());
        if let Some(hit) = self.num_table_cache.borrow().get(&key) {
            return hit.clone();
        }
        let result = self.numeric_table_uncached(model_name, func);
        self.num_table_cache
            .borrow_mut()
            .insert(key, result.clone());
        result
    }

    fn numeric_table_uncached(&self, model_name: &str, func: &str) -> Option<NumTable> {
        // Rides the -O/--optimize flag (default on). The emitted gadgets are
        // pure 1-bit and/or/not circuits with shared suffix-OR chains, so
        // define-funs stay on dryadsynth-bv's bitsliced BitCircuit plan.
        // LOOM_NO_TABLE_CIRCUITS=1 disables the pass for differential runs.
        if !self.optimize || std::env::var_os("LOOM_NO_TABLE_CIRCUITS").is_some() {
            return None;
        }
        let sig = self.ctx.func_sigs.get(func)?;
        if sig.arg_sorts.len() != 2 {
            return None;
        }
        let arg_sort = sig.arg_sorts[0].to_string();
        if sig.arg_sorts[1].to_string() != arg_sort {
            return None;
        }
        let ret_sort = sig.ret_sort.to_string();
        let is_bool = ret_sort == "Bool";
        if !is_bool && ret_sort != arg_sort {
            return None;
        }
        let model = self.ctx.model(model_name).ok()?;
        let univ = model.univ_for_sort(&arg_sort)?;
        let n = univ.len();
        if n < 2 || n > 60 {
            return None;
        }
        let lookup = |a: usize, b: usize| -> Option<String> {
            model
                .eval_symbols(func, &[univ[a].clone(), univ[b].clone()])
                .map(|v| v.to_string())
        };
        // Hypotheses: the trailing element is the invalid/saturation target,
        // or (Bool tables only) there is no invalid element.
        for inv_idx in [Some(n - 1), None] {
            let valid = |i: usize| inv_idx != Some(i);
            let max_valid = match inv_idx {
                Some(i) => i.checked_sub(1)?,
                None => n - 1,
            };
            let mut kinds: Vec<NumTableKind> = if is_bool {
                let mut ks = vec![NumTableKind::Ule, NumTableKind::Ult];
                if inv_idx.is_some() {
                    ks.push(NumTableKind::UltInvBig);
                }
                ks
            } else if inv_idx.is_some() {
                vec![NumTableKind::AddSat, NumTableKind::SubSat]
            } else {
                vec![]
            };
            kinds.retain(|kind| {
                (0..n).all(|a| {
                    (0..n).all(|b| {
                        let out = lookup(a, b);
                        match kind {
                            NumTableKind::Ule => {
                                (out.as_deref() == Some("true")) == (valid(a) && valid(b) && a <= b)
                            }
                            NumTableKind::Ult => {
                                (out.as_deref() == Some("true")) == (valid(a) && valid(b) && a < b)
                            }
                            NumTableKind::UltInvBig => {
                                (out.as_deref() == Some("true"))
                                    == (valid(a) && (!valid(b) || a < b))
                            }
                            NumTableKind::AddSat => {
                                let expect = if valid(a) && valid(b) && a + b <= max_valid {
                                    &univ[a + b]
                                } else {
                                    &univ[inv_idx.unwrap()]
                                };
                                out.as_deref() == Some(expect.as_str())
                            }
                            NumTableKind::SubSat => {
                                let expect = if valid(a) && valid(b) && a >= b {
                                    &univ[a - b]
                                } else {
                                    &univ[inv_idx.unwrap()]
                                };
                                out.as_deref() == Some(expect.as_str())
                            }
                        }
                    })
                })
            });
            if let Some(kind) = kinds.first() {
                return Some(NumTable {
                    kind: *kind,
                    arg_sort,
                    inv_idx,
                });
            }
        }
        None
    }

    /// One-hot value field for `expr` over `univ` (listed order, LSB first).
    fn onehot_field(
        &self,
        expr: &Value,
        sort: &str,
        univ: &[String],
        inv_idx: Option<usize>,
        env: &Env,
    ) -> Option<OneHotField> {
        let width = univ.len();
        if let Ok(value) = self.cached_eval_value(expr, env) {
            if let AspectValue::Univ { sort: s, name } = &value {
                if s.as_ref() == sort {
                    let idx = univ.iter().position(|u| u == name.as_ref())?;
                    return Some(OneHotField {
                        bits: FieldBits::Const(Some(idx)),
                        width,
                        inv_idx,
                    });
                }
            }
            return None;
        }
        let items = expr.to_ref_vec()?;
        if items.len() != 3 {
            return None;
        }
        let relation = items[0].as_symbol()?;
        if !self.ctx.has_relation(relation) {
            return None;
        }
        let domain = self.ctx.domain_for_relation(relation).ok()?;
        let SemanticKind::FiniteValue { sort: ret_sort } = &domain.kind else {
            return None;
        };
        if ret_sort != sort {
            return None;
        }
        let child = items[1].as_symbol()?;
        let binding = env.children.get(child)?;
        let aspect = self.cached_eval_aspect(items[2], relation, env).ok()?;
        let mut atoms = Vec::with_capacity(width);
        for name in univ {
            let value = AspectValue::Univ {
                sort: Arc::from(sort),
                name: Arc::from(name.as_str()),
            };
            let bit = self.bit_for(relation, &aspect, Some(&value)).ok()?;
            atoms.push(bit_is_one(extract_arg_bit(binding.arg_idx, bit)));
        }
        Some(OneHotField {
            bits: FieldBits::Dyn(atoms),
            width,
            inv_idx,
        })
    }

    /// Parse `expr` as a numeric side: a leaf field, or nested saturating
    /// +/- applications where all but one operand folds to a constant.
    fn parse_num_side(
        &self,
        expr: &Value,
        model_name: &str,
        sort: &str,
        univ: &[String],
        inv_idx: Option<usize>,
        env: &Env,
        depth: usize,
    ) -> Option<NumSide> {
        if depth > 4 {
            return None;
        }
        let max_valid = match inv_idx {
            Some(i) => i.checked_sub(1)? as isize,
            None => univ.len() as isize - 1,
        };
        if let Some(field) = self.onehot_field(expr, sort, univ, inv_idx, env) {
            return Some(match &field.bits {
                FieldBits::Const(Some(idx)) if Some(*idx) != inv_idx => NumSide {
                    field: None,
                    offset: *idx as isize,
                    leaves: vec![],
                    guards: vec![],
                },
                FieldBits::Const(_) => NumSide {
                    // constant invalid/no value: no valid evaluation
                    field: None,
                    offset: -1,
                    leaves: vec![field],
                    guards: vec![bool_lit(false)],
                },
                FieldBits::Dyn(_) => NumSide {
                    field: Some(field.clone()),
                    offset: 0,
                    leaves: vec![field],
                    guards: vec![],
                },
            });
        }
        let items = expr.to_ref_vec()?;
        if items.len() != 3 {
            return None;
        }
        let func = items[0].as_symbol()?;
        let table = self.numeric_table(model_name, func)?;
        if table.arg_sort != sort || table.inv_idx != inv_idx {
            return None;
        }
        let lhs = self.parse_num_side(items[1], model_name, sort, univ, inv_idx, env, depth + 1)?;
        let rhs = self.parse_num_side(items[2], model_name, sort, univ, inv_idx, env, depth + 1)?;
        // only one dynamic field per side is supported
        let (mut side, other_offset, negate_other) = match (&lhs.field, &rhs.field, table.kind) {
            (_, None, NumTableKind::AddSat) => (lhs, rhs.offset, false),
            (None, _, NumTableKind::AddSat) => (rhs, lhs.offset, false),
            (_, None, NumTableKind::SubSat) => (lhs, rhs.offset, true),
            _ => return None,
        };
        if negate_other {
            // value = side - other_offset; valid iff side >= other_offset
            let guard = side_ge_const(&side, other_offset);
            side.guards.push(guard);
            side.offset -= other_offset;
        } else {
            side.offset += other_offset;
            // stepwise saturation: this node's sum must stay <= max_valid
            let guard = side_le_const(&side, max_valid);
            side.guards.push(guard);
        }
        Some(side)
    }

    fn side_all_valid(&self, side: &NumSide) -> SyGuSTerm {
        let mut conjuncts: Vec<SyGuSTerm> = side.leaves.iter().map(|f| f.valid_term()).collect();
        conjuncts.extend(side.guards.iter().cloned());
        and_terms(conjuncts)
    }

    /// Bool term: the side evaluates to the invalid element (table
    /// propagation: some leaf invalid while all leaves hold a value, or all
    /// valid but a saturation guard fails).
    fn side_invalid(&self, side: &NumSide) -> SyGuSTerm {
        let any_all = and_terms(side.leaves.iter().map(|f| f.any_term()).collect());
        let leaf_inv = or_terms(side.leaves.iter().map(|f| f.invalid_term()).collect());
        let leaf_valid = and_terms(side.leaves.iter().map(|f| f.valid_term()).collect());
        let guard_broken = or_terms(
            side.guards
                .iter()
                .map(|g| match g {
                    t if *t == bool_lit(true) => bool_lit(false),
                    t if *t == bool_lit(false) => bool_lit(true),
                    t => Bool::not(t.clone()),
                })
                .collect(),
        );
        and_terms(vec![
            any_all,
            or_terms(vec![leaf_inv, and_terms(vec![leaf_valid, guard_broken])]),
        ])
    }

    /// Native template for a recognized comparison table applied to
    /// parseable sides. Returns None to fall back to element expansion.
    fn try_native_compare(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        if args.len() != 2 {
            return Ok(None);
        }
        let model_name = self.ctx.current_model(env)?.to_string();
        let Some(table) = self.numeric_table(&model_name, func) else {
            return Ok(None);
        };
        if !matches!(
            table.kind,
            NumTableKind::Ule | NumTableKind::Ult | NumTableKind::UltInvBig
        ) {
            return Ok(None);
        }
        let model = self.ctx.model(&model_name)?;
        let Some(univ) = model.univ_for_sort(&table.arg_sort) else {
            return Ok(None);
        };
        let parse = |expr: &Value| {
            self.parse_num_side(
                expr,
                &model_name,
                &table.arg_sort,
                &univ,
                table.inv_idx,
                env,
                0,
            )
        };
        let (Some(l), Some(r)) = (parse(args[0]), parse(args[1])) else {
            return Ok(None);
        };
        let max_valid = match table.inv_idx {
            Some(i) => i as isize - 1,
            None => univ.len() as isize - 1,
        };
        let term = match table.kind {
            NumTableKind::Ule | NumTableKind::Ult => {
                let strict = table.kind == NumTableKind::Ult;
                let cmp = compare_sides(&l, &r, strict, max_valid);
                and_terms(vec![self.side_all_valid(&l), self.side_all_valid(&r), cmp])
            }
            NumTableKind::UltInvBig => {
                let cmp = compare_sides(&l, &r, true, max_valid);
                and_terms(vec![
                    self.side_all_valid(&l),
                    or_terms(vec![
                        self.side_invalid(&r),
                        and_terms(vec![self.side_all_valid(&r), cmp]),
                    ]),
                ])
            }
            _ => unreachable!(),
        };
        Ok(Some(match term {
            t if t == bool_lit(true) => BoolExpr::Const(true),
            t if t == bool_lit(false) => BoolExpr::Const(false),
            t => BoolExpr::Dynamic(t),
        }))
    }

    /// Native template for `(= lhs rhs)` where at least one side applies a
    /// recognized AddSat/SubSat function.
    fn try_native_equality(
        &self,
        lhs: &Value,
        rhs: &Value,
        env: &Env,
    ) -> Result<Option<BoolExpr>, String> {
        let head_table = |expr: &Value| -> Option<NumTable> {
            let items = expr.to_ref_vec()?;
            let func = items.first()?.as_symbol()?;
            let model_name = self.ctx.current_model(env).ok()?.to_string();
            let table = self.numeric_table(&model_name, func)?;
            matches!(table.kind, NumTableKind::AddSat | NumTableKind::SubSat).then_some(table)
        };
        let Some(table) = head_table(lhs).or_else(|| head_table(rhs)) else {
            return Ok(None);
        };
        let model_name = self.ctx.current_model(env)?.to_string();
        let model = self.ctx.model(&model_name)?;
        let Some(univ) = model.univ_for_sort(&table.arg_sort) else {
            return Ok(None);
        };
        let parse = |expr: &Value| {
            self.parse_num_side(
                expr,
                &model_name,
                &table.arg_sort,
                &univ,
                table.inv_idx,
                env,
                0,
            )
        };
        let (Some(l), Some(r)) = (parse(lhs), parse(rhs)) else {
            return Ok(None);
        };
        let max_valid = match table.inv_idx {
            Some(i) => i as isize - 1,
            None => univ.len() as isize - 1,
        };
        // valid & equal: distribute over the smaller side's atoms
        let eq = or_terms(
            l.value_atoms(max_valid)
                .into_iter()
                .map(|(v, a)| and_terms(vec![a, r.value_is(v)]))
                .collect(),
        );
        let both_valid = and_terms(vec![self.side_all_valid(&l), self.side_all_valid(&r), eq]);
        let both_invalid = and_terms(vec![self.side_invalid(&l), self.side_invalid(&r)]);
        let term = or_terms(vec![both_valid, both_invalid]);
        Ok(Some(match term {
            t if t == bool_lit(true) => BoolExpr::Const(true),
            t if t == bool_lit(false) => BoolExpr::Const(false),
            t => BoolExpr::Dynamic(t),
        }))
    }
}

/// side >= c (valid context), gate-level.
fn side_ge_const(side: &NumSide, c: isize) -> SyGuSTerm {
    side.value_ge(c)
}

/// side <= c (valid context), gate-level: NOT (side >= c+1).
fn side_le_const(side: &NumSide, c: isize) -> SyGuSTerm {
    match side.value_ge(c + 1) {
        t if t == bool_lit(false) => bool_lit(true),
        t if t == bool_lit(true) => bool_lit(false),
        t => Bool::not(t),
    }
}

/// l < r or l <= r (valid context): OR over l's atoms of r's suffix-OR,
/// index-shifted; the shared suffix chains keep the circuit O(n).
fn compare_sides(l: &NumSide, r: &NumSide, strict: bool, max_valid: isize) -> SyGuSTerm {
    let bump = if strict { 1 } else { 0 };
    match (&l.field, &r.field) {
        (None, _) => r.value_ge(l.offset + bump),
        (Some(_), _) => or_terms(
            l.value_atoms(max_valid)
                .into_iter()
                .map(|(v, a)| and_terms(vec![a, r.value_ge(v + bump)]))
                .collect(),
        ),
    }
}
