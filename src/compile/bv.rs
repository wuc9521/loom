use crate::{
    backends::sygus::optim::{self, BitwiseExpr},
    compile::{
        semantic::{interped_bool, interped_value_matches, parse_cases, Case},
        syntax::{synth_name, TermConstructor, TermSignature},
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
use std::collections::HashMap;
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

struct NativeCompiler<'a> {
    problem: &'a DelisProg,
    program: SemanticProgram,
    sig: TermSignature,
    models: HashMap<String, &'a Model>,
    func_sigs: HashMap<String, FuncSig>,
    spaces: HashMap<String, FiniteSpace>,
    specs: HashMap<String, SemanticSpec>,
    domains_by_relation: HashMap<String, SemanticDomain>,
    domain_by_sort: HashMap<String, SemanticDomain>,
    cases: HashMap<String, HashMap<String, Case>>,
    bit_index: HashMap<(String, AspectKey, Option<AspectValue>), usize>,
    optimize: bool,
}

pub fn compile(
    problem: &DelisProg,
    program: SemanticProgram,
    optimize: bool,
) -> Result<Vec<SyGuSCmd>, String> {
    let declare_term_types = problem.cmds.iter().find_map(|cmd| match cmd {
        DelisCmd::TermTypes(cmd) => Some(cmd),
        _ => None,
    });
    let Some(declare_term_types) = declare_term_types else {
        return Err("SyGuS compilation requires declare-term-types".to_string());
    };
    let sig = TermSignature::from_declare_term_types(declare_term_types)?;
    let mut compiler = NativeCompiler::new(problem, program, sig, optimize)?;
    compiler.compile()
}

impl<'a> NativeCompiler<'a> {
    fn new(
        problem: &'a DelisProg,
        program: SemanticProgram,
        sig: TermSignature,
        optimize: bool,
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
        let mut bit_index = HashMap::new();
        for domain in &program.domains {
            for bit in &domain.bits {
                bit_index.insert(
                    (
                        domain.relation.clone(),
                        bit.aspect.clone(),
                        bit.value.clone(),
                    ),
                    bit.bit,
                );
            }
        }
        Ok(Self {
            problem,
            program,
            sig,
            models,
            func_sigs,
            spaces,
            specs,
            domains_by_relation,
            domain_by_sort,
            cases,
            bit_index,
            optimize,
        })
    }

    fn compile(&mut self) -> Result<Vec<SyGuSCmd>, String> {
        let mut cmds = vec![SyGuSCmd::SetLogic("BV".to_string())];
        let grammar = self.grammar()?;
        for cons in &self.sig.constructors {
            cmds.push(self.constructor_fun(cons)?);
        }
        let start_domain = self.domain_for_sort(self.sig.start_sort())?;
        cmds.push(
            Cmd::synth_fun(&synth_name(self.problem)?)
                .ret(Sort::bitvec(start_domain.bits.len().max(1)))
                .grammar(grammar)
                .build(),
        );
        let synth = Term::var(&synth_name(self.problem)?);
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
            let idx = self.bit_for(&constraint.relation, &aspect, None)?;
            let bit = BitVec::extract(idx, idx, synth.clone());
            cmds.push(
                Cmd::constraint()
                    .body(Bool::eq(
                        bit,
                        BitVec::const_(if constraint.expected { 1 } else { 0 }, 1),
                    ))
                    .build(),
            );
        }
        cmds.push(SyGuSCmd::CheckSynth);
        Ok(cmds)
    }

    fn grammar(&self) -> Result<GrammarDef, String> {
        let declare_term_types = self.problem.cmds.iter().find_map(|cmd| match cmd {
            DelisCmd::TermTypes(cmd) => Some(cmd),
            _ => None,
        });
        let Some(decl) = declare_term_types else {
            return Err("SyGuS compilation requires declare-term-types".to_string());
        };
        let sorted_vars = decl
            .sort_decls
            .iter()
            .map(|sort_decl| {
                let domain = self.domain_for_sort(&sort_decl.symbol)?;
                Ok(SortedVar {
                    name: self.sig.grammar_symbol_for_sort(&sort_decl.symbol),
                    sort: Sort::bitvec(domain.bits.len().max(1)),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let grouped_rule_lists = decl
            .sort_decls
            .iter()
            .zip(decl.term_type_decls.iter())
            .map(|(sort_decl, term_type_decl)| {
                let domain = self.domain_for_sort(&sort_decl.symbol)?;
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
                    sort: Sort::bitvec(domain.bits.len().max(1)),
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
        let domain = self.domain_for_sort(&cons.result_sort)?;
        let spec = self.spec_for_relation(&domain.relation)?;
        let cases = self.cases.get(&spec.relation).ok_or_else(|| {
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
                "match case for constructor {} binds {} variables, expected {}",
                cons.name,
                case.vars.len(),
                cons.arg_sorts.len()
            ));
        }
        let mut env = Env::default();
        for (idx, var) in case.vars.iter().enumerate() {
            env.children
                .insert(var.clone(), ChildBinding { arg_idx: idx });
        }

        let mut bool_bits = Vec::new();
        for bit in &domain.bits {
            env.aspect.insert("asp".to_string(), bit.aspect.clone());
            let bool_expr = match (&domain.kind, &bit.value) {
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
            bool_bits.push(bool_expr);
        }

        let mut builder = Cmd::define_fun(&cons.name);
        let arg_names = (0..cons.arg_sorts.len()).map(arg_name).collect::<Vec<_>>();
        for (idx, sort) in cons.arg_sorts.iter().enumerate() {
            let arg_domain = self.domain_for_sort(sort)?;
            builder = builder.arg(&arg_names[idx], Sort::bitvec(arg_domain.bits.len().max(1)));
        }
        let lift_bits = bool_bits
            .iter()
            .map(|expr| match expr {
                BoolExpr::Const(value) => BitwiseExpr::Const(*value),
                BoolExpr::Dynamic(term) => BitwiseExpr::Expr(term.clone()),
            })
            .collect::<Vec<_>>();
        let body = (if self.optimize {
            optim::try_lift_native_bv(&lift_bits, &arg_names)
        } else {
            None
        })
        .unwrap_or_else(|| {
            let bits = bool_bits
                .into_iter()
                .map(|expr| match expr {
                    BoolExpr::Const(value) => BitExpr::Const(value),
                    BoolExpr::Dynamic(term) => bool_to_bit(term),
                })
                .collect::<Vec<_>>();
            compact_bitvec(bits)
        });
        Ok(builder
            .ret(Sort::bitvec(domain.bits.len().max(1)))
            .body(body)
            .build())
    }

    fn eval_bool(&self, expr: &Value, env: &Env) -> Result<BoolExpr, String> {
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
                let parts = items
                    .iter()
                    .skip(1)
                    .map(|item| self.eval_bool(item, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(and_exprs(parts))
            }
            "or" => {
                let parts = items
                    .iter()
                    .skip(1)
                    .map(|item| self.eval_bool(item, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(or_exprs(parts))
            }
            "=" if items.len() == 3 => self.eval_finite_terms_equal(items[1], items[2], env),
            "forall" | "exists" if items.len() == 3 => {
                self.eval_quantifier(op, items[1], items[2], env)
            }
            relation if self.domains_by_relation.contains_key(relation) => {
                self.eval_semantic_bool_call(relation, &items[1..], env)
            }
            func if self.func_sigs.contains_key(func) => {
                self.eval_model_bool_call(func, &items[1..], env)
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
        let (model, sort) = self.parse_domain_expr(domain_expr, env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let values = get_domain_for_sort(model_ref, &sort)
            .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))?;
        let mut parts = Vec::new();
        for value in values {
            let mut next = env.clone();
            next.values.insert(
                var.to_string(),
                AspectValue::Univ {
                    sort: sort.clone(),
                    name: value,
                },
            );
            parts.push(self.eval_bool(body, &next)?);
        }
        match op {
            "forall" => Ok(and_exprs(parts)),
            "exists" => Ok(or_exprs(parts)),
            _ => unreachable!(),
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

    fn eval_semantic_bool_call(
        &self,
        relation: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let domain = self.domain_for_relation(relation)?;
        if domain.kind != SemanticKind::Bool {
            return Err(format!("{relation} does not return Bool"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let child_name = args[0]
            .as_symbol()
            .ok_or_else(|| format!("semantic call term must be a child variable: {}", args[0]))?;
        let child = env
            .children
            .get(child_name)
            .ok_or_else(|| format!("unknown semantic child variable {child_name}"))?;
        let aspect = self.eval_aspect(args[1], relation, env)?;
        let bit = self.bit_for(relation, &aspect, None)?;
        Ok(BoolExpr::Dynamic(bit_is_one(extract_arg_bit(
            child.arg_idx,
            bit,
        ))))
    }

    fn eval_model_bool_call(
        &self,
        func: &str,
        args: &[&Value],
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let sig = self.func_sigs.get(func).unwrap();
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
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let domains = sig
            .arg_sorts
            .iter()
            .map(|sort| {
                let sort = sort.to_string();
                get_domain_for_sort(model_ref, &sort)
                    .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))
                    .map(|values| {
                        values
                            .into_iter()
                            .map(|name| AspectValue::Univ {
                                sort: sort.clone(),
                                name,
                            })
                            .collect::<Vec<_>>()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut disjuncts = Vec::new();
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
            if !interped_bool(value) {
                continue;
            }
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

    fn eval_finite_eq(
        &self,
        expr: &Value,
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        if let Some(symbol) = expr.as_symbol() {
            if let Some(value) = env.values.get(symbol) {
                return Ok(BoolExpr::Const(value == target));
            }
            let model = self.current_model(env)?;
            let model_ref = self
                .models
                .get(&model)
                .ok_or_else(|| format!("unknown model {model}"))?;
            if let Some(value) = eval_symbols(symbol, &[], &model_ref.func_interps) {
                return Ok(BoolExpr::Const(interped_value_matches(value, target)));
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
        if self.domains_by_relation.contains_key(op) {
            return self.eval_semantic_finite_call_eq(op, &items[1..], target, env);
        }
        if self.func_sigs.contains_key(op) {
            return self.eval_model_finite_call_eq(op, &items[1..], target, env);
        }
        Err(format!("unsupported finite expression: {expr}"))
    }

    fn eval_semantic_finite_call_eq(
        &self,
        relation: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let domain = self.domain_for_relation(relation)?;
        if !matches!(domain.kind, SemanticKind::FiniteValue { .. }) {
            return Err(format!("{relation} does not return a finite value"));
        }
        if args.len() != 2 {
            return Err(format!("semantic call {relation} expects term and aspect"));
        }
        let child_name = args[0]
            .as_symbol()
            .ok_or_else(|| format!("semantic call term must be a child variable: {}", args[0]))?;
        let child = env
            .children
            .get(child_name)
            .ok_or_else(|| format!("unknown semantic child variable {child_name}"))?;
        let aspect = self.eval_aspect(args[1], relation, env)?;
        let bit = self.bit_for(relation, &aspect, Some(target))?;
        Ok(BoolExpr::Dynamic(bit_is_one(extract_arg_bit(
            child.arg_idx,
            bit,
        ))))
    }

    fn eval_model_finite_call_eq(
        &self,
        func: &str,
        args: &[&Value],
        target: &AspectValue,
        env: &Env,
    ) -> Result<BoolExpr, String> {
        let sig = self.func_sigs.get(func).unwrap();
        if sig.arg_sorts.len() != args.len() {
            return Err(format!(
                "function {func} expects {} arguments, got {}",
                sig.arg_sorts.len(),
                args.len()
            ));
        }
        let model = self.current_model(env)?;
        let model_ref = self
            .models
            .get(&model)
            .ok_or_else(|| format!("unknown model {model}"))?;
        let domains = sig
            .arg_sorts
            .iter()
            .map(|sort| {
                let sort = sort.to_string();
                get_domain_for_sort(model_ref, &sort)
                    .ok_or_else(|| format!("model {model} has no universe for sort {sort}"))
                    .map(|values| {
                        values
                            .into_iter()
                            .map(|name| AspectValue::Univ {
                                sort: sort.clone(),
                                name,
                            })
                            .collect::<Vec<_>>()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut disjuncts = Vec::new();
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
            let conjuncts = args
                .iter()
                .zip(tuple.iter())
                .map(|(arg_expr, value)| self.eval_finite_eq(arg_expr, value, env))
                .collect::<Result<Vec<_>, _>>()?;
            disjuncts.push(and_exprs(conjuncts));
        }
        Ok(or_exprs(disjuncts))
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
        if items.len() - 1 != space.fields.len() {
            return Err(format!(
                "aspect constructor {cons} expects {} fields, got {}",
                space.fields.len(),
                items.len() - 1
            ));
        }
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

fn arg_name(idx: usize) -> String {
    format!("arg{idx}")
}

fn extract_arg_bit(arg_idx: usize, bit: usize) -> SyGuSTerm {
    BitVec::extract(bit, bit, Term::var(&arg_name(arg_idx)))
}

fn bit_is_one(bit: SyGuSTerm) -> SyGuSTerm {
    Bool::eq(bit, BitVec::const_(1, 1))
}
