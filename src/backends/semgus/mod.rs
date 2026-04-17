use crate::{
    lang::model::{FuncInterp, InterpedValue, Model, PatternElem},
    lang::signature::FuncSig,
    parser::{ast::TermTypes, DelisCmd, DelisProg},
};
use lexpr::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use sygus_parser::{Sort, SyGuSCmd, SyGuSTerm};

#[derive(Clone, Debug)]
struct AspectField {
    name: String,
    sort: String,
    enum_values: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct SemanticsHeader {
    relation: String,
    term_var: String,
    term_sort: String,
    aspect_sort: String,
}

#[derive(Clone, Debug)]
struct SemanticsCase {
    constructor: String,
    vars: Vec<String>,
    body: Value,
}

#[derive(Clone, Debug)]
struct SemanticsDef {
    header: SemanticsHeader,
    cases: Vec<SemanticsCase>,
}

#[derive(Clone, Debug)]
struct BoolTranslation {
    expr: String,
    relations: Vec<String>,
}

#[derive(Default)]
struct TranslationCtx {
    next_output: usize,
    next_arg: usize,
    aux_bindings: Vec<(String, String)>,
}

impl TranslationCtx {
    fn fresh_output(&mut self) -> String {
        let name = format!("semgus_out{}", self.next_output);
        self.next_output += 1;
        self.aux_bindings.push((name.clone(), "Bool".to_string()));
        name
    }

    fn fresh_relation_arg(&mut self, sort: &str) -> String {
        let name = format!("semgus_arg{}", self.next_arg);
        self.next_arg += 1;
        self.aux_bindings.push((name.clone(), sort.to_string()));
        name
    }
}

fn quantify_aux_outputs(body: String, ctx: &TranslationCtx) -> String {
    if ctx.aux_bindings.is_empty() {
        return body;
    }
    let bindings = ctx
        .aux_bindings
        .iter()
        .map(|(name, sort)| format!("({name} {sort})"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("(exists ({bindings}) {body})")
}

pub fn compile(program: &DelisProg) -> Result<String, String> {
    let term_types = program
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DelisCmd::TermTypes(term_types) => Some(term_types),
            _ => None,
        })
        .ok_or_else(|| "SemGuS backend requires declare-term-types".to_string())?;
    let models = program
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) => Some(model),
            _ => None,
        })
        .collect::<Vec<_>>();
    let semantics = program
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::Semantics(semantics) => Some(parse_semantics(semantics)),
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?;
    let finite_aspects = program
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::FiniteAspect(aspect) => Some(parse_aspect_fields(aspect)),
            _ => None,
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let func_sigs = program
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DeclareFunc(sig) => Some((sig.name.clone(), sig.clone())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    let mut lines = Vec::new();
    emit_prelude(program, &models, &mut lines);
    lines.push(term_types.to_string());
    emit_enum_sorts(&finite_aspects, &mut lines);
    emit_universe_sorts(&models, &mut lines);
    let use_int_universe_args = finite_aspects
        .values()
        .flatten()
        .any(|field| field.sort == "Int");
    emit_model_functions(&models, &func_sigs, use_int_universe_args, &mut lines);
    for semantics in &semantics {
        emit_semantics(
            semantics,
            term_types,
            &finite_aspects,
            &func_sigs,
            &models,
            &mut lines,
        )?;
    }
    emit_synth_funs(program, &mut lines);
    emit_constraints(
        program,
        &semantics,
        &finite_aspects,
        &func_sigs,
        &models,
        &mut lines,
    )?;
    lines.push("(check-synth)".to_string());
    Ok(lines.join("\n"))
}

fn emit_prelude(program: &DelisProg, models: &[&Model], lines: &mut Vec<String>) {
    let finite_model_sorts = models
        .iter()
        .flat_map(|model| model.univ_defs.iter().map(|univ| univ.sort.to_string()))
        .collect::<BTreeSet<_>>();
    for cmd in &program.cmds {
        let DelisCmd::As(cmd) = cmd else {
            continue;
        };
        match cmd {
            SyGuSCmd::SetLogic(_) => lines.push(cmd.to_string()),
            SyGuSCmd::DeclareSort(sort, _) if !finite_model_sorts.contains(sort) => {
                lines.push(cmd.to_string())
            }
            _ => {}
        }
    }
}

fn emit_synth_funs(program: &DelisProg, lines: &mut Vec<String>) {
    for cmd in &program.cmds {
        if let DelisCmd::As(SyGuSCmd::SynthFun(..)) = cmd {
            lines.push(cmd.to_string());
        }
    }
}

fn emit_enum_sorts(finite_aspects: &HashMap<String, Vec<AspectField>>, lines: &mut Vec<String>) {
    for (aspect, fields) in finite_aspects {
        for field in fields {
            let Some(values) = &field.enum_values else {
                continue;
            };
            lines.push(format!(
                "(declare-datatypes (({} 0)) (({})))",
                enum_sort(aspect, &field.name),
                values
                    .iter()
                    .map(|value| format!("({value})"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    }
}

fn emit_universe_sorts(models: &[&Model], lines: &mut Vec<String>) {
    let mut elems_by_sort = BTreeMap::<String, BTreeSet<String>>::new();
    for model in models {
        for univ in &model.univ_defs {
            let sort = univ.sort.to_string();
            for elem in &univ.elems {
                elems_by_sort
                    .entry(sort.clone())
                    .or_default()
                    .insert(elem.name.clone());
            }
        }
    }
    for (sort, elems) in elems_by_sort {
        lines.push(format!(
            "(declare-datatypes (({sort} 0)) (({})))",
            elems
                .iter()
                .map(|elem| format!("({elem})"))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
}

fn emit_model_functions(
    models: &[&Model],
    func_sigs: &HashMap<String, FuncSig>,
    use_int_universe_args: bool,
    lines: &mut Vec<String>,
) {
    for model in models {
        let universe_sort = model.univ_defs.first().map(|univ| univ.sort.to_string());
        for interp in &model.func_interps {
            let Some(sig) = func_sigs.get(interp.name()) else {
                continue;
            };
            let name = model_func_name(model, interp.name());
            match interp {
                FuncInterp::SimpleInterp(_, value) => {
                    lines.push(format!(
                        "(define-fun {name} () {} {})",
                        sig.ret_sort,
                        interped_value(value)
                    ));
                }
                FuncInterp::MatchInterp(_, cases) => {
                    let args = sig
                        .arg_sorts
                        .iter()
                        .enumerate()
                        .map(|(idx, sort)| {
                            let sort = if use_int_universe_args
                                && universe_sort.as_deref() == Some(&sort.to_string())
                            {
                                "Int".to_string()
                            } else {
                                sort.to_string()
                            };
                            format!("(arg{idx} {sort})")
                        })
                        .collect::<Vec<_>>();
                    let body = match sig.ret_sort.to_string().as_str() {
                        "Bool" => bool_match_body(cases, use_int_universe_args, model),
                        _ => value_match_body(cases, use_int_universe_args, model),
                    };
                    lines.push(format!(
                        "(define-fun {name} ({}) {} {body})",
                        args.join(" "),
                        sig.ret_sort
                    ));
                }
            }
        }
    }
}

fn bool_match_body(
    cases: &[crate::lang::model::MatchCase],
    use_int_universe_args: bool,
    model: &Model,
) -> String {
    let mut terms = Vec::new();
    for case in cases {
        if case.value.to_string() != "true" {
            continue;
        }
        terms.push(pattern_condition(case, use_int_universe_args, model));
    }
    bool_nary("or", terms)
}

fn value_match_body(
    cases: &[crate::lang::model::MatchCase],
    use_int_universe_args: bool,
    model: &Model,
) -> String {
    let mut default = None;
    let mut guarded = Vec::new();
    for case in cases {
        let value = interped_value(&case.value);
        if is_wildcard_pattern(case) {
            default = Some(value);
        } else {
            guarded.push((pattern_condition(case, use_int_universe_args, model), value));
        }
    }
    let mut body = default.unwrap_or_else(|| {
        guarded
            .last()
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| "false".to_string())
    });
    for (cond, value) in guarded.into_iter().rev() {
        body = format!("(ite {cond} {value} {body})");
    }
    body
}

fn pattern_condition(
    case: &crate::lang::model::MatchCase,
    use_int_universe_args: bool,
    model: &Model,
) -> String {
    let Some(pattern) = &case.pattern else {
        return "true".to_string();
    };
    let mut parts = Vec::new();
    for (idx, elem) in pattern.elements.iter().enumerate() {
        match elem {
            PatternElem::UnivElem(elem) if use_int_universe_args => {
                let pos = model
                    .univ_defs
                    .first()
                    .and_then(|univ| univ.elems.iter().position(|candidate| candidate == elem))
                    .map(|pos| pos.to_string())
                    .unwrap_or_else(|| elem.name.clone());
                parts.push(format!("(= arg{idx} {pos})"));
            }
            PatternElem::UnivElem(elem) => parts.push(format!("(= arg{idx} {})", elem.name)),
            PatternElem::Var(_) | PatternElem::Wildcard => {}
        }
    }
    bool_nary("and", parts)
}

fn is_wildcard_pattern(case: &crate::lang::model::MatchCase) -> bool {
    case.pattern.as_ref().is_some_and(|pattern| {
        pattern
            .elements
            .iter()
            .all(|elem| matches!(elem, PatternElem::Wildcard | PatternElem::Var(_)))
    })
}

fn emit_semantics(
    semantics: &SemanticsDef,
    term_types: &TermTypes,
    finite_aspects: &HashMap<String, Vec<AspectField>>,
    func_sigs: &HashMap<String, FuncSig>,
    models: &[&Model],
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let aspect_fields = finite_aspects
        .get(&semantics.header.aspect_sort)
        .ok_or_else(|| {
            format!(
                "SemGuS backend cannot find finite aspect {}",
                semantics.header.aspect_sort
            )
        })?;
    let scalar_fields = aspect_fields
        .iter()
        .filter(|field| field.sort != "Model")
        .collect::<Vec<_>>();
    for model in models {
        let relation = relation_name(&semantics.header.relation, model);
        let constructors = constructors_by_sort(term_types);
        let constructor_args = constructor_arg_sorts(term_types);
        let mut decls = Vec::new();
        let mut bodies = Vec::new();
        for (sort, constructor_names) in &constructors {
            let sorted_relation = relation_sort_name(&relation, sort);
            let mut signature_args = Vec::new();
            signature_args.push(format!("({} {})", semantics.header.term_var, sort));
            for field in &scalar_fields {
                signature_args.push(format!(
                    "({} {})",
                    field.name,
                    semgus_field_sort(&semantics.header.aspect_sort, field)
                ));
            }
            signature_args.push("(result Bool)".to_string());
            decls.push(format!(
                "({sorted_relation} ({}) Bool)",
                signature_args.join(" ")
            ));

            let mut cases = Vec::new();
            for case in semantics
                .cases
                .iter()
                .filter(|case| constructor_names.contains(&case.constructor))
            {
                let mut env = case
                    .vars
                    .iter()
                    .map(|var| (var.clone(), var.clone()))
                    .collect::<HashMap<_, _>>();
                let mut var_sorts = constructor_args
                    .get(&case.constructor)
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .filter_map(|(idx, sort)| {
                        case.vars.get(idx).map(|var| (var.clone(), sort.clone()))
                    })
                    .collect::<HashMap<_, _>>();
                var_sorts.insert(semantics.header.term_var.clone(), sort.clone());
                for field in &scalar_fields {
                    env.insert(field.name.clone(), field.name.clone());
                }
                let mut ctx = TranslationCtx::default();
                let body = translate_bool(
                    &case.body,
                    &semantics.header,
                    &semantics.header.relation,
                    sort,
                    model,
                    aspect_fields,
                    func_sigs,
                    &env,
                    &var_sorts,
                    &mut ctx,
                )?;
                let mut conjuncts = body.relations;
                conjuncts.push(format!("(= result {})", body.expr));
                let case_body = quantify_aux_outputs(bool_nary("and", conjuncts), &ctx);
                let pattern = if case.vars.is_empty() {
                    format!("({})", case.constructor)
                } else {
                    format!("({} {})", case.constructor, case.vars.join(" "))
                };
                cases.push(format!("({pattern} {case_body})"));
            }
            bodies.push(format!(
                "(! (match {} ({})) :input ({}) :output (result))",
                semantics.header.term_var,
                cases.join(" "),
                scalar_fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
        lines.push(format!(
            "(define-funs-rec\n  ({})\n  ({}))",
            decls.join("\n   "),
            bodies.join("\n   ")
        ));
    }
    Ok(())
}

fn emit_constraints(
    program: &DelisProg,
    semantics: &[SemanticsDef],
    finite_aspects: &HashMap<String, Vec<AspectField>>,
    func_sigs: &HashMap<String, FuncSig>,
    models: &[&Model],
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let semantics_by_relation = semantics
        .iter()
        .map(|semantics| (semantics.header.relation.clone(), semantics))
        .collect::<HashMap<_, _>>();
    for cmd in &program.cmds {
        let DelisCmd::As(SyGuSCmd::Constraint(term)) = cmd else {
            continue;
        };
        let constraint = translate_constraint(
            term,
            &semantics_by_relation,
            finite_aspects,
            func_sigs,
            models,
        )?;
        lines.push(format!("(constraint {constraint})"));
    }
    Ok(())
}

fn translate_constraint(
    term: &SyGuSTerm,
    semantics: &HashMap<String, &SemanticsDef>,
    finite_aspects: &HashMap<String, Vec<AspectField>>,
    func_sigs: &HashMap<String, FuncSig>,
    models: &[&Model],
) -> Result<String, String> {
    let expr = lexpr::from_str(&term.to_string())
        .map_err(|e| format!("failed to parse SemGuS constraint {term}: {e}"))?;
    let (expected, expr) = match list_items(&expr).as_deref() {
        Some([op, inner]) if op.as_symbol() == Some("not") => ("false", *inner),
        _ => ("true", &expr),
    };
    let items =
        list_items(expr).ok_or_else(|| format!("semantic constraint must be a call: {expr}"))?;
    if items.len() < 3 {
        return Err(format!(
            "semantic constraint expects relation, term, aspect: {expr}"
        ));
    }
    let relation = symbol(items[0])?;
    let semantics = semantics
        .get(relation)
        .ok_or_else(|| format!("constraint references unknown semantics {relation}"))?;
    let aspect_items = list_items(items[2])
        .ok_or_else(|| format!("constraint aspect must be an application: {}", items[2]))?;
    let aspect_name = symbol(aspect_items[0])?;
    let aspect_fields = finite_aspects
        .get(aspect_name)
        .ok_or_else(|| format!("constraint references unknown aspect {aspect_name}"))?;
    let model_idx = aspect_fields
        .iter()
        .position(|field| field.sort == "Model")
        .ok_or_else(|| format!("aspect {aspect_name} has no Model field"))?;
    let model_name = value_to_string(aspect_items[model_idx + 1]);
    let model = models
        .iter()
        .find(|model| model.name == model_name)
        .copied()
        .ok_or_else(|| format!("constraint references unknown model {model_name}"))?;
    let env = HashMap::new();
    let args = aspect_fields
        .iter()
        .zip(aspect_items.iter().skip(1))
        .filter(|(field, _)| field.sort != "Model")
        .map(|(_, value)| value_expr(value, model, func_sigs, &env))
        .collect::<Vec<_>>();
    Ok(format!(
        "({} {} {} {expected})",
        relation_sort_name(
            &relation_name(&semantics.header.relation, model),
            &semantics.header.term_sort,
        ),
        value_to_string(items[1]),
        args.join(" ")
    ))
}

fn translate_bool(
    expr: &Value,
    header: &SemanticsHeader,
    relation: &str,
    current_sort: &str,
    model: &Model,
    aspect_fields: &[AspectField],
    func_sigs: &HashMap<String, FuncSig>,
    env: &HashMap<String, String>,
    var_sorts: &HashMap<String, String>,
    ctx: &mut TranslationCtx,
) -> Result<BoolTranslation, String> {
    if let Some(symbol) = expr.as_symbol() {
        return match symbol {
            "true" | "false" => Ok(BoolTranslation {
                expr: symbol.to_string(),
                relations: Vec::new(),
            }),
            _ => Ok(BoolTranslation {
                expr: value_expr(expr, model, func_sigs, env),
                relations: Vec::new(),
            }),
        };
    }
    let items = list_items(expr).ok_or_else(|| format!("expected Boolean expression: {expr}"))?;
    let op = symbol(items[0])?;
    match op {
        "and" | "or" => items[1..]
            .iter()
            .map(|item| {
                translate_bool(
                    item,
                    header,
                    relation,
                    current_sort,
                    model,
                    aspect_fields,
                    func_sigs,
                    env,
                    var_sorts,
                    ctx,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|items| {
                let mut relations = Vec::new();
                let exprs = items
                    .into_iter()
                    .map(|item| {
                        relations.extend(item.relations);
                        item.expr
                    })
                    .collect::<Vec<_>>();
                BoolTranslation {
                    expr: bool_nary(op, exprs),
                    relations,
                }
            }),
        "not" if items.len() == 2 => {
            let inner = translate_bool(
                items[1],
                header,
                relation,
                current_sort,
                model,
                aspect_fields,
                func_sigs,
                env,
                var_sorts,
                ctx,
            )?;
            Ok(BoolTranslation {
                expr: format!("(not {})", inner.expr),
                relations: inner.relations,
            })
        }
        "=" if items.len() == 3 => Ok(BoolTranslation {
            expr: format!(
                "(= {} {})",
                value_expr(items[1], model, func_sigs, env),
                value_expr(items[2], model, func_sigs, env)
            ),
            relations: Vec::new(),
        }),
        "exists" | "forall" if items.len() == 3 => translate_quantifier(
            op,
            items[1],
            items[2],
            header,
            relation,
            current_sort,
            model,
            aspect_fields,
            func_sigs,
            env,
            var_sorts,
            ctx,
        ),
        _ if op == relation => {
            let target_sort = semantic_call_sort(items[1], current_sort, var_sorts);
            let aspect_args = semantic_call_args(
                items.get(2).copied(),
                model,
                &header.aspect_sort,
                aspect_fields,
                func_sigs,
                env,
            )?;
            let mut conjuncts = Vec::new();
            let mut relation_args = vec![relation_arg(
                value_expr(items[1], model, func_sigs, env),
                &target_sort,
                ctx,
                &mut conjuncts,
            )];
            relation_args.extend(
                aspect_args
                    .into_iter()
                    .map(|(arg, sort)| relation_arg(arg, &sort, ctx, &mut conjuncts)),
            );
            let output = ctx.fresh_output();
            relation_args.push(output.clone());
            conjuncts.push(format!(
                "({} {})",
                relation_sort_name(&relation_name(&header.relation, model), &target_sort),
                relation_args.join(" ")
            ));
            Ok(BoolTranslation {
                expr: output.clone(),
                relations: conjuncts,
            })
        }
        _ if func_sigs
            .get(op)
            .is_some_and(|sig| sig.ret_sort == Sort::bool()) =>
        {
            Ok(BoolTranslation {
                expr: format!(
                    "({} {})",
                    model_func_name(model, op),
                    items[1..]
                        .iter()
                        .map(|item| value_expr(item, model, func_sigs, env))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                relations: Vec::new(),
            })
        }
        _ => Err(format!("unsupported SemGuS Boolean expression: {expr}")),
    }
}

fn translate_quantifier(
    op: &str,
    bindings: &Value,
    body: &Value,
    header: &SemanticsHeader,
    relation: &str,
    current_sort: &str,
    model: &Model,
    aspect_fields: &[AspectField],
    func_sigs: &HashMap<String, FuncSig>,
    env: &HashMap<String, String>,
    var_sorts: &HashMap<String, String>,
    ctx: &mut TranslationCtx,
) -> Result<BoolTranslation, String> {
    let binding_items = list_items(bindings)
        .ok_or_else(|| format!("quantifier bindings must be a list: {bindings}"))?;
    if binding_items.len() != 1 {
        return Err("SemGuS backend currently supports one binding per quantifier".to_string());
    }
    let binding = list_items(binding_items[0])
        .ok_or_else(|| format!("quantifier binding must be a list: {}", binding_items[0]))?;
    if binding.len() != 2 {
        return Err(format!(
            "quantifier binding must have shape (name domain): {}",
            binding_items[0]
        ));
    }
    let var = symbol(binding[0])?.to_string();
    let domain = list_items(binding[1])
        .ok_or_else(|| format!("quantifier domain must be a list: {}", binding[1]))?;
    match domain.first().and_then(|value| value.as_symbol()) {
        Some("domain") if domain.len() == 2 => {
            let domain_expr = list_items(domain[1]).ok_or_else(|| {
                format!(
                    "SemGuS quantifier domain must have shape (domain (Sort model)): {}",
                    binding[1]
                )
            })?;
            let [sort_expr, _model_expr] = domain_expr.as_slice() else {
                return Err(format!(
                    "SemGuS quantifier domain must have shape (domain (Sort model)): {}",
                    binding[1]
                ));
            };
            let sort = symbol(sort_expr)?;
            let terms = model
                .univ_defs
                .iter()
                .filter(|univ| univ.sort.to_string() == sort)
                .flat_map(|univ| univ.elems.iter())
                .map(|elem| {
                    let mut env = env.clone();
                    env.insert(var.clone(), elem.name.clone());
                    translate_bool(
                        body,
                        header,
                        relation,
                        current_sort,
                        model,
                        aspect_fields,
                        func_sigs,
                        &env,
                        var_sorts,
                        ctx,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut relations = Vec::new();
            let exprs = terms
                .into_iter()
                .map(|term| {
                    relations.extend(term.relations);
                    term.expr
                })
                .collect::<Vec<_>>();
            Ok(BoolTranslation {
                expr: bool_nary(if op == "exists" { "or" } else { "and" }, exprs),
                relations,
            })
        }
        Some("int-range") if domain.len() == 3 => {
            let lo = value_expr(domain[1], model, func_sigs, env);
            let hi = value_expr(domain[2], model, func_sigs, env);
            let body = translate_bool(
                body,
                header,
                relation,
                current_sort,
                model,
                aspect_fields,
                func_sigs,
                env,
                var_sorts,
                ctx,
            )?;
            let guard = format!("(and (<= {lo} {var}) (<= {var} {hi}))");
            let guarded_body = if op == "exists" {
                format!("(and {guard} {})", body.expr)
            } else {
                format!("(or (not {guard}) {})", body.expr)
            };
            Ok(BoolTranslation {
                expr: format!("({op} (({var} Int)) {guarded_body})"),
                relations: body.relations,
            })
        }
        _ => Err(format!(
            "unsupported SemGuS quantifier domain: {}",
            binding[1]
        )),
    }
}

fn semantic_call_args(
    aspect: Option<&Value>,
    model: &Model,
    aspect_sort: &str,
    aspect_fields: &[AspectField],
    func_sigs: &HashMap<String, FuncSig>,
    env: &HashMap<String, String>,
) -> Result<Vec<(String, String)>, String> {
    let Some(aspect) = aspect else {
        return Err("semantic call is missing an aspect argument".to_string());
    };
    if aspect.as_symbol() == Some("asp") {
        return Ok(aspect_fields
            .iter()
            .filter(|field| field.sort != "Model")
            .map(|field| {
                (
                    env.get(&field.name)
                        .cloned()
                        .unwrap_or_else(|| field.name.clone()),
                    semgus_field_sort(aspect_sort, field),
                )
            })
            .collect());
    }
    let items = list_items(aspect)
        .ok_or_else(|| format!("aspect expression must be asp or an application: {aspect}"))?;
    let mut args = Vec::new();
    for (field, value) in aspect_fields.iter().zip(items.iter().skip(1)) {
        if field.sort == "Model" {
            continue;
        }
        args.push((
            value_expr(value, model, func_sigs, env),
            semgus_field_sort(aspect_sort, field),
        ));
    }
    Ok(args)
}

fn relation_arg(
    expr: String,
    sort: &str,
    ctx: &mut TranslationCtx,
    conjuncts: &mut Vec<String>,
) -> String {
    if is_relation_var(&expr) {
        return expr;
    }
    let var = ctx.fresh_relation_arg(sort);
    conjuncts.push(format!("(= {var} {expr})"));
    var
}

fn is_relation_var(expr: &str) -> bool {
    !expr.is_empty()
        && !expr.starts_with('$')
        && !expr.starts_with(|ch: char| ch.is_ascii_digit() || ch == '-')
        && !expr.contains(['(', ')', ' '])
        && expr != "true"
        && expr != "false"
}

fn value_expr(
    expr: &Value,
    model: &Model,
    func_sigs: &HashMap<String, FuncSig>,
    env: &HashMap<String, String>,
) -> String {
    if let Some(symbol) = expr.as_symbol() {
        if let Some(value) = env.get(symbol) {
            return value.clone();
        }
        if func_sigs
            .get(symbol)
            .is_some_and(|sig| sig.arg_sorts.is_empty())
        {
            return format!("({})", model_func_name(model, symbol));
        }
        return symbol.to_string();
    }
    if let Some(value) = expr.as_i64() {
        return value.to_string();
    }
    let Some(items) = list_items(expr) else {
        return expr.to_string();
    };
    if items.len() == 2 && items[1].as_symbol() == Some("asp") {
        if let Some(field) = items[0].as_symbol() {
            if let Some(value) = env.get(field) {
                return value.clone();
            }
        }
    }
    let Ok(op) = symbol(items[0]) else {
        return expr.to_string();
    };
    match op {
        "+" | "-" => format!(
            "({op} {})",
            items[1..]
                .iter()
                .map(|item| value_expr(item, model, func_sigs, env))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        "len" if items.len() == 2 => {
            let Some(domain_items) = list_items(items[1]) else {
                return expr.to_string();
            };
            let [sort_expr, _model_expr] = domain_items.as_slice() else {
                return expr.to_string();
            };
            let Ok(sort) = symbol(sort_expr) else {
                return expr.to_string();
            };
            model
                .univ_defs
                .iter()
                .find(|univ| univ.sort.to_string() == sort)
                .map(|univ| univ.elems.len().to_string())
                .unwrap_or_else(|| "0".to_string())
        }
        _ if func_sigs.contains_key(op) => {
            if func_sigs
                .get(op)
                .is_some_and(|sig| sig.arg_sorts.is_empty())
            {
                format!("({})", model_func_name(model, op))
            } else {
                format!(
                    "({} {})",
                    model_func_name(model, op),
                    items[1..]
                        .iter()
                        .map(|item| value_expr(item, model, func_sigs, env))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
        _ => value_to_string(expr),
    }
}

fn parse_aspect_fields(
    aspect: &crate::parser::ast::FiniteAspect,
) -> Result<(String, Vec<AspectField>), String> {
    let record = aspect
        .body
        .first()
        .ok_or_else(|| format!("define-finite {} expects a record", aspect.name))?;
    let record = lexpr::from_str(record)
        .map_err(|e| format!("failed to parse define-finite {}: {e}", aspect.name))?;
    let items = list_items(&record)
        .ok_or_else(|| format!("define-finite {} record must be a list", aspect.name))?;
    if items.first().and_then(|value| value.as_symbol()) != Some("record") {
        return Err(format!(
            "define-finite {} must start with record",
            aspect.name
        ));
    }
    let fields = items[1..]
        .iter()
        .map(|field| {
            let pair =
                list_items(field).ok_or_else(|| format!("record field must be a pair: {field}"))?;
            if pair.len() != 2 {
                return Err(format!("record field must be a pair: {field}"));
            }
            let name = symbol(pair[0])?.to_string();
            if let Some(sort) = pair[1].as_symbol() {
                return Ok(AspectField {
                    name,
                    sort: sort.to_string(),
                    enum_values: None,
                });
            }
            let domain = list_items(pair[1])
                .ok_or_else(|| format!("unsupported field domain: {}", pair[1]))?;
            if domain.first().and_then(|value| value.as_symbol()) == Some("enum") {
                let values = domain[1..]
                    .iter()
                    .map(|value| symbol(value).map(str::to_string))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(AspectField {
                    sort: enum_sort(&aspect.name, &name),
                    name,
                    enum_values: Some(values),
                });
            }
            Err(format!("unsupported field domain: {}", pair[1]))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((aspect.name.clone(), fields))
}

fn parse_semantics(semantics: &crate::parser::ast::Semantics) -> Result<SemanticsDef, String> {
    let header = semantics
        .body
        .first()
        .ok_or_else(|| "define-semantics expects a header".to_string())?;
    let body = semantics
        .body
        .get(1)
        .ok_or_else(|| "define-semantics expects a match body".to_string())?;
    let header = parse_semantics_header(header)?;
    let body =
        lexpr::from_str(body).map_err(|e| format!("failed to parse define-semantics body: {e}"))?;
    let outer =
        list_items(&body).ok_or_else(|| format!("define-semantics body must be a list: {body}"))?;
    let match_expr = if outer.len() == 1 { outer[0] } else { &body };
    let items = list_items(match_expr)
        .ok_or_else(|| format!("define-semantics body must contain match: {match_expr}"))?;
    if items.first().and_then(|value| value.as_symbol()) != Some("match") {
        return Err(format!(
            "define-semantics body must start with match: {match_expr}"
        ));
    }
    let cases = items[2..]
        .iter()
        .map(|case| {
            let items =
                list_items(case).ok_or_else(|| format!("match case must be a list: {case}"))?;
            if items.len() != 2 {
                return Err(format!("match case must have pattern and body: {case}"));
            }
            let pattern = list_items(items[0])
                .ok_or_else(|| format!("match pattern must be a list: {}", items[0]))?;
            let constructor = symbol(pattern[0])?.to_string();
            let vars = pattern[1..]
                .iter()
                .map(|value| symbol(value).map(str::to_string))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(SemanticsCase {
                constructor,
                vars,
                body: items[1].clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SemanticsDef { header, cases })
}

fn constructors_by_sort(term_types: &TermTypes) -> BTreeMap<String, BTreeSet<String>> {
    term_types
        .sort_decls
        .iter()
        .zip(term_types.term_type_decls.iter())
        .map(|(sort, term_type)| {
            (
                sort.symbol.clone(),
                term_type
                    .constructors
                    .iter()
                    .map(|constructor| constructor.symbol.to_string())
                    .collect(),
            )
        })
        .collect()
}

fn constructor_arg_sorts(term_types: &TermTypes) -> HashMap<String, Vec<String>> {
    term_types
        .term_type_decls
        .iter()
        .flat_map(|term_type| {
            term_type.constructors.iter().map(|constructor| {
                (
                    constructor.symbol.to_string(),
                    constructor
                        .sorts
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                )
            })
        })
        .collect()
}

fn parse_semantics_header(header: &str) -> Result<SemanticsHeader, String> {
    let value = lexpr::from_str(header)
        .map_err(|e| format!("failed to parse define-semantics header: {e}"))?;
    let outer =
        list_items(&value).ok_or_else(|| format!("semantics header must be a list: {value}"))?;
    let signature = if outer.len() == 1 { outer[0] } else { &value };
    let items = list_items(signature)
        .ok_or_else(|| format!("semantics signature must be a list: {signature}"))?;
    if items.len() != 3 {
        return Err(format!(
            "expected semantics signature with relation, term, aspect: {signature}"
        ));
    }
    let term = list_items(items[1])
        .ok_or_else(|| format!("term argument must be a list: {}", items[1]))?;
    let aspect = list_items(items[2])
        .ok_or_else(|| format!("aspect argument must be a list: {}", items[2]))?;
    if term.len() != 2 || aspect.len() != 2 {
        return Err(format!("invalid semantics signature: {signature}"));
    }
    Ok(SemanticsHeader {
        relation: symbol(items[0])?.to_string(),
        term_var: symbol(term[0])?.to_string(),
        term_sort: value_to_string(term[1]),
        aspect_sort: value_to_string(aspect[1]),
    })
}

fn model_func_name(model: &Model, func: &str) -> String {
    format!("{func}.{}", model.name)
}

fn relation_name(relation: &str, model: &Model) -> String {
    format!("{relation}.{}", model.name)
}

fn relation_sort_name(relation: &str, sort: &str) -> String {
    format!("{relation}.{sort}")
}

fn semantic_call_sort(
    target: &Value,
    current_sort: &str,
    var_sorts: &HashMap<String, String>,
) -> String {
    target
        .as_symbol()
        .and_then(|symbol| var_sorts.get(symbol))
        .cloned()
        .unwrap_or_else(|| current_sort.to_string())
}

fn enum_sort(aspect: &str, field: &str) -> String {
    format!("{aspect}.{field}")
}

fn semgus_field_sort(aspect: &str, field: &AspectField) -> String {
    field
        .enum_values
        .as_ref()
        .map(|_| enum_sort(aspect, &field.name))
        .unwrap_or_else(|| field.sort.clone())
}

fn interped_value(value: &InterpedValue) -> String {
    match value {
        InterpedValue::UnivElem(elem) => elem.name.clone(),
        InterpedValue::Literal(lit) => lit.to_string(),
        InterpedValue::FuncApp(name, args) => format!(
            "({} {})",
            name,
            args.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ),
    }
}

fn bool_nary(op: &str, items: Vec<String>) -> String {
    match items.len() {
        0 if op == "and" => "true".to_string(),
        0 => "false".to_string(),
        1 => items.into_iter().next().unwrap(),
        _ => format!("({op} {})", items.join(" ")),
    }
}

fn list_items(value: &Value) -> Option<Vec<&Value>> {
    value.to_ref_vec()
}

fn symbol(value: &Value) -> Result<&str, String> {
    value
        .as_symbol()
        .ok_or_else(|| format!("expected symbol, got {value}"))
}

fn value_to_string(value: &Value) -> String {
    value.to_string()
}
