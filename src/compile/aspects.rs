use crate::{
    compile::{
        expr::{eval_bool, eval_sygus_int},
        types::{
            AspectKey, AspectValue, BitMapping, ConstraintBit, DomainBit, Encoding, FieldDecl,
            FieldDomain, AspectSpace, SemanticConstraint, SemanticDomain, SemanticKind,
            SemanticProgram, SemanticSpec,
        },
    },
    lang::{
        eval::collect_loaded_models,
        model::Model,
    },
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use std::sync::Arc;
use sygus_parser::SExpr as Value;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sygus_parser::{Identifier, SyGuSTerm};

type AspectAssignment = Vec<(String, AspectValue)>;

impl DelisProg {
    #[allow(dead_code)]
    pub fn encode(&self) -> Result<Encoding, String> {
        encode(self)
    }
}

fn encode(problem: &DelisProg) -> Result<Encoding, String> {
    let program = problem.semantic_program()?;

    let bool_domains = program
        .domains
        .iter()
        .filter(|domain| domain.kind == SemanticKind::Bool)
        .collect::<Vec<_>>();
    let mut bit_mappings = Vec::new();
    let mut bit_by_aspect = HashMap::default();
    for domain in bool_domains {
        for domain_bit in &domain.bits {
            let bit = bit_mappings.len();
            bit_by_aspect.insert(domain_bit.aspect.clone(), bit);
            bit_mappings.push(BitMapping {
                bit,
                aspect: domain_bit.aspect.clone(),
            });
        }
    }

    let kind_by_relation = program
        .domains
        .iter()
        .map(|domain| (domain.relation.clone(), domain.kind.clone()))
        .collect::<HashMap<_, _>>();
    let constraint_bits = program
        .constraints
        .iter()
        .map(|constraint| {
            if kind_by_relation.get(&constraint.relation) != Some(&SemanticKind::Bool) {
                return Err(format!(
                    "top-level constraint {} must target Bool-returning semantics",
                    constraint.relation
                ));
            }
            let key = AspectKey {
                relation: constraint.relation.clone(),
                model: constraint.model.clone(),
                fields: constraint.fields.clone(),
            };
            let bit = bit_by_aspect.get(&key).copied().ok_or_else(|| {
                format!(
                    "constraint aspect does not belong to aspect space: {}({}; {})",
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

    Ok(Encoding {
        aspect_spaces: program
            .aspect_spaces
            .iter()
            .map(|space| space.name.clone())
            .collect(),
        semantic_relations: program
            .semantics
            .iter()
            .map(|semantics| semantics.relation.clone())
            .collect(),
        semantic_domains: program.domains,
        bit_mappings,
        constraint_bits,
    })
}

impl DelisProg {
    pub fn semantic_program(&self) -> Result<SemanticProgram, String> {
        compile_semantic_program(self)
    }
}

fn compile_semantic_program(problem: &DelisProg) -> Result<SemanticProgram, String> {
    let aspect_cmds = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineAspects(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();
    let aspect_spaces = aspect_cmds
        .iter()
        .map(|cmd| AspectSpace::from_cmd(cmd))
        .collect::<Result<Vec<_>, _>>()?;
    let semantics = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::Semantics(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();

    if aspect_cmds.is_empty() {
        return Err("compilation requires at least one define-aspects".to_string());
    }
    if semantics.is_empty() {
        return Err("compilation requires at least one define-semantics".to_string());
    }

    let semantic_specs = semantics
        .iter()
        .map(|cmd| SemanticSpec::from_cmd(cmd))
        .collect::<Result<Vec<_>, _>>()?;
    let spaces_by_name = aspect_spaces
        .iter()
        .map(|space| (space.name.clone(), space))
        .collect::<HashMap<_, _>>();
    for spec in &semantic_specs {
        if !spaces_by_name.contains_key(&spec.aspect_space) {
            return Err(format!(
                "semantics {} references unknown aspect space {}",
                spec.relation, spec.aspect_space
            ));
        }
    }
    let models_by_name = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::DefineModel(model) => Some((model.name.clone(), model)),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let semantic_constraints =
        collect_semantic_constraints(problem, &semantic_specs, &spaces_by_name, &models_by_name)?;
    let mut model_names = Vec::new();
    let mut seen_model_names = HashSet::default();
    for constraint in &semantic_constraints {
        if models_by_name.contains_key(&constraint.model)
            && seen_model_names.insert(constraint.model.clone())
        {
            model_names.push(constraint.model.clone());
        }
    }
    for model_name in referenced_models_in_constraints(problem, &spaces_by_name) {
        if models_by_name.contains_key(&model_name) && seen_model_names.insert(model_name.clone()) {
            model_names.push(model_name);
        }
    }
    let loaded_models = collect_loaded_models(&model_names, &models_by_name);
    let model_univs = loaded_models
        .iter()
        .map(|model| (model.name.clone(), univs_by_sort(model)))
        .collect::<HashMap<_, _>>();
    let mut aspect_cache: HashMap<(String, String), Vec<AspectAssignment>> = HashMap::default();

    // A parametric synth-fun `(synth-fun f ((x0 U) ...) T)` compiles to a
    // genuine SyGuS function, not a fold: its formal parameters carry the free
    // variables. The matching aspect fields (same names, on the root semantics
    // whose term sort is the synth return sort) are therefore NOT enumerated
    // into the fold — they become BV parameters of `f`. Dropping them keeps the
    // value BV width at Σ_M |U_M| instead of multiplying by |U| per parameter.
    let param_field_names = match problem.synth_signature() {
        Some((_, args, _sort)) => args.iter().map(|a| a.name.clone()).collect::<HashSet<_>>(),
        None => HashSet::default(),
    };

    let mut domains = Vec::new();
    for spec in &semantic_specs {
        let aspect_space = spaces_by_name
            .get(&spec.aspect_space)
            .ok_or_else(|| format!("unknown aspect space {}", spec.aspect_space))?;
        let skip_fields = &param_field_names;
        let mut bits = Vec::new();
        for model in &loaded_models {
            let cache_key = (aspect_space.name.clone(), model.name.clone());
            let assignments = if let Some(assignments) = aspect_cache.get(&cache_key) {
                assignments
            } else {
                let model_univ_map = model_univs.get(&model.name).ok_or_else(|| {
                    format!(
                        "internal error: missing cached univs for model {}",
                        model.name
                    )
                })?;
                let assignments = enumerate_aspects(
                    aspect_space,
                    model.name.as_str(),
                    model,
                    &models_by_name,
                    model_univ_map,
                    skip_fields,
                )?;
                aspect_cache.entry(cache_key).or_insert(assignments)
            };
            for assignment in assignments {
                let fields = assignment
                    .iter()
                    .filter(|(_, value)| !matches!(value, AspectValue::Model(_)))
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect::<Vec<_>>();
                let aspect = AspectKey {
                    relation: spec.relation.clone(),
                    model: model.name.clone(),
                    fields,
                };
                match &spec.return_kind {
                    SemanticKind::Bool => {
                        bits.push(DomainBit {
                            bit: bits.len(),
                            aspect,
                            value: None,
                        });
                    }
                    SemanticKind::FiniteValue { sort } => {
                        let values = model_univs
                            .get(&model.name)
                            .and_then(|univs| univs.get(sort))
                            .ok_or_else(|| {
                                format!(
                                    "model {} has no universe for return sort {} of semantics {}",
                                    model.name, sort, spec.relation
                                )
                            })?;
                        let sort_arc: Arc<str> = Arc::from(sort.as_str());
                        for name in values {
                            bits.push(DomainBit {
                                bit: bits.len(),
                                aspect: aspect.clone(),
                                value: Some(AspectValue::Univ {
                                    sort: sort_arc.clone(),
                                    name: Arc::from(name.as_str()),
                                }),
                            });
                        }
                    }
                }
            }
        }
        domains.push(SemanticDomain {
            relation: spec.relation.clone(),
            term_sort: spec.term_sort.clone(),
            aspect_space: spec.aspect_space.clone(),
            kind: spec.return_kind.clone(),
            bits,
        });
    }

    let mut program = SemanticProgram {
        aspect_spaces,
        semantics: semantic_specs,
        domains,
        constraints: semantic_constraints,
    };

    if super::opts().bisim {
        let _ = &loaded_models; // models are reachable via the SemanticContext built inside bisim
        program.quotient_by_bisim(problem);
    }

    Ok(program)
}

fn referenced_models_in_constraints(
    problem: &DelisProg,
    spaces_by_name: &HashMap<String, &AspectSpace>,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::default();
    for cmd in &problem.cmds {
        let term = match cmd {
            DelisCmd::As(sygus_parser::SyGuSCmd::Constraint(term)) => term,
            DelisCmd::DeclareAxiom(term) => term,
            _ => {
                continue;
            }
        };
        collect_models_in_term(term, spaces_by_name, &mut seen, &mut out);
    }
    out
}

fn collect_models_in_term(
    term: &SyGuSTerm,
    spaces_by_name: &HashMap<String, &AspectSpace>,
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    match term {
        SyGuSTerm::Application(id, args) => {
            let name = id.get_name();
            if let Some(space) = spaces_by_name.get(name) {
                for (field, arg) in space.fields.iter().zip(args.iter()) {
                    if matches!(&field.domain, FieldDomain::Sort(sort) if sort == "Model") {
                        if let SyGuSTerm::Identifier(Identifier::Symbol(model_name)) = arg {
                            if seen.insert(model_name.clone()) {
                                out.push(model_name.clone());
                            }
                        }
                    }
                }
            }
            for arg in args {
                collect_models_in_term(arg, spaces_by_name, seen, out);
            }
        }
        SyGuSTerm::Let(bindings, body) => {
            for binding in bindings {
                collect_models_in_term(&binding.term, spaces_by_name, seen, out);
            }
            collect_models_in_term(body, spaces_by_name, seen, out);
        }
        SyGuSTerm::Annotated(inner, _) => collect_models_in_term(inner, spaces_by_name, seen, out),
        _ => {}
    }
}

pub fn contains_parent_navigation(expr: &Value) -> bool {
    let Some(items) = expr.to_ref_vec() else {
        return false;
    };
    items.first().and_then(|item| item.as_symbol()) == Some("parent")
        || items.into_iter().any(contains_parent_navigation)
}

fn collect_semantic_constraints(
    problem: &DelisProg,
    semantic_specs: &[SemanticSpec],
    spaces_by_name: &HashMap<String, &AspectSpace>,
    models_by_name: &HashMap<String, &Model>,
) -> Result<Vec<SemanticConstraint>, String> {
    let semantics_by_relation = semantic_specs
        .iter()
        .map(|spec| (spec.relation.clone(), spec))
        .collect::<HashMap<_, _>>();
    let mut constraints = Vec::new();
    for cmd in &problem.cmds {
        let term = match cmd {
            DelisCmd::As(sygus_parser::SyGuSCmd::Constraint(term)) => term,
            DelisCmd::DeclareAxiom(_) => continue,
            _ => continue,
        };
        if let Some(constraint) =
            parse_semantic_constraint(term, &semantics_by_relation, spaces_by_name, models_by_name)?
        {
            constraints.push(constraint);
        }
    }
    Ok(constraints)
}

fn parse_semantic_constraint(
    term: &SyGuSTerm,
    semantics_by_relation: &HashMap<String, &SemanticSpec>,
    spaces_by_name: &HashMap<String, &AspectSpace>,
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
    let Some(spec) = semantics_by_relation.get(&relation) else {
        return Ok(None);
    };
    let Some(aspect_term) = args.last() else {
        return Err(format!(
            "semantic constraint {relation} is missing an aspect argument"
        ));
    };
    let aspect_space = spaces_by_name
        .get(&spec.aspect_space)
        .ok_or_else(|| format!("unknown aspect space {}", spec.aspect_space))?;
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
    aspect_space: &AspectSpace,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
) -> Result<(String, Vec<(String, AspectValue)>), String> {
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
        if field.domain.as_sort() == Some("Model") {
            model_name = Some(arg.to_string());
            break;
        }
    }
    let model_name =
        model_name.ok_or_else(|| format!("aspect {} has no Model field", aspect_space.name))?;
    let model = models_by_name
        .get(&model_name)
        .ok_or_else(|| format!("constraint references unknown model {model_name}"))?;
    let univs = univs_by_sort(model);

    let mut assignment = Vec::new();
    for (field, arg) in aspect_space.fields.iter().zip(args.iter()) {
        let value = match &field.domain {
            FieldDomain::Sort(sort) if sort == "Model" => AspectValue::Model(arg.to_string()),
            FieldDomain::Sort(sort) if sort == "Int" => {
                let value = eval_sygus_int(arg, &assignment, models_by_name)?;
                if value < 0 {
                    return Err(format!(
                        "aspect field {} must be non-negative, got {value}",
                        field.name
                    ));
                }
                AspectValue::Int(value as usize)
            }
            FieldDomain::Sort(sort) if univs.contains_key(sort) => {
                let univ = univs.get(sort).ok_or_else(|| {
                    format!(
                        "universe sort {sort} not found while evaluating aspect {}",
                        field.name
                    )
                })?;
                let value = eval_univ_expr(arg, sort, univ, model, &field.name)?;
                AspectValue::Univ {
                    sort: Arc::from(sort.as_str()),
                    name: Arc::from(value),
                }
            }
            FieldDomain::Sort(_sort) => {
                AspectValue::Symbol(arg.to_string())
            }
            FieldDomain::Enum(values) => {
                let value = arg.to_string();
                if !values.contains(&value) {
                    return Err(format!(
                        "aspect field {} expected one of ({}) but got {value}",
                        field.name,
                        values.join(" ")
                    ));
                }
                AspectValue::Symbol(value)
            }
        };
        assignment.push((field.name.clone(), value));
    }
    if let Some(where_expr) = &aspect_space.where_expr {
        if !eval_bool(where_expr, &assignment, models_by_name)? {
            return Err(format!(
                "constraint aspect is outside aspect space {}: {}",
                aspect_space.name, term
            ));
        }
    }
    let fields = assignment
        .iter()
        .filter(|(_, value)| !matches!(value, AspectValue::Model(_)))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    Ok((model_name, fields))
}

fn eval_univ_expr(
    arg: &SyGuSTerm,
    sort: &str,
    univ: &[String],
    model: &Model,
    field_name: &str,
) -> Result<String, String> {
    let literal = arg.to_string();
    if univ.contains(&literal) {
        return Ok(literal);
    }
    let value = eval_model_expr(arg, model)?;
    if univ.contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "aspect field {field_name} expected an element of {sort}, got {value} (from {arg})"
        ))
    }
}

fn eval_model_expr(term: &SyGuSTerm, model: &Model) -> Result<String, String> {
    match term {
        SyGuSTerm::Identifier(id) => {
            let name = id.to_string();
            match model.eval_symbols(&name, &[]) {
                Some(val) => Ok(val.to_string()),
                None => Ok(name),
            }
        }
        SyGuSTerm::Literal(_) => Ok(term.to_string()),
        SyGuSTerm::Application(func, args) => {
            let func_name = func.to_string();
            let resolved: Vec<String> = args
                .iter()
                .map(|a| eval_model_expr(a, model))
                .collect::<Result<_, _>>()?;
            model
                .eval_symbols(&func_name, &resolved)
                .map(|v| v.to_string())
                .ok_or_else(|| {
                    format!(
                        "function {} undefined for [{}] in model {}",
                        func_name,
                        resolved.join(", "),
                        model.name
                    )
                })
        }
        _ => Ok(term.to_string()),
    }
}

impl AspectSpace {
    fn from_cmd(cmd: &crate::parser::ast::DefineAspects) -> Result<Self, String> {
    let record_src = cmd
        .body
        .first()
        .ok_or_else(|| format!("define-aspects {} expects a record body", cmd.name))?;
    let record = sygus_parser::SExpr::from_str(record_src)
        .map_err(|e| format!("failed to parse define-aspects {} record: {e}", cmd.name))?;
    let fields = parse_record_fields(&record)?;
    let where_expr = cmd
        .body
        .iter()
        .position(|item| item == ":where")
        .and_then(|idx| cmd.body.get(idx + 1))
        .map(|src| {
            sygus_parser::SExpr::from_str(src)
                .map_err(|e| format!("failed to parse define-aspects {} :where: {e}", cmd.name))
        })
        .transpose()?;
    Ok(AspectSpace {
        name: cmd.name.clone(),
        fields,
        where_expr,
    })
}
}

fn parse_record_fields(record: &Value) -> Result<Vec<FieldDecl>, String> {
    let list = record
        .to_ref_vec()
        .ok_or_else(|| "define-aspects record must be a list".to_string())?;
    if list.first().and_then(|v| v.as_symbol()) != Some("record") {
        return Err("define-aspects body must start with (record ...)".to_string());
    }
    list.iter()
        .skip(1)
        .map(|field| {
            let pair = field
                .to_ref_vec()
                .ok_or_else(|| format!("record field must be a list, got {field}"))?;
            match pair.as_slice() {
                [name, domain] => Ok(FieldDecl {
                    name: name
                        .as_symbol()
                        .ok_or_else(|| format!("record field name must be a symbol, got {name}"))?
                        .to_string(),
                    domain: parse_field_domain(domain)?,
                }),
                _ => Err(format!(
                    "record field must have shape (name sort), got {field}"
                )),
            }
        })
        .collect()
}

fn parse_field_domain(domain: &Value) -> Result<FieldDomain, String> {
    if let Some(sort) = domain.as_symbol() {
        return Ok(FieldDomain::Sort(sort.to_string()));
    }
    let items = domain.to_ref_vec().ok_or_else(|| {
        format!("record field domain must be a sort or domain expression: {domain}")
    })?;
    match items.as_slice() {
        [op, values @ ..] if op.as_symbol() == Some("enum") => values
            .iter()
            .map(|value| {
                value
                    .as_symbol()
                    .map(str::to_string)
                    .ok_or_else(|| format!("enum value must be a symbol: {value}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(FieldDomain::Enum),
        _ => Err(format!("unsupported record field domain {domain}")),
    }
}

fn enumerate_aspects(
    space: &AspectSpace,
    model_name: &str,
    model: &crate::lang::model::Model,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
    domains_by_sort: &HashMap<String, Vec<String>>,
    skip_fields: &HashSet<String>,
) -> Result<Vec<AspectAssignment>, String> {
    let universe_sorts = model_universe_sorts(model);
    let int_bound = model.max_univ_len();
    let domains = space
        .fields
        .iter()
        .filter(|field| !skip_fields.contains(&field.name))
        .map(|field| {
            let values = match &field.domain {
                FieldDomain::Sort(sort) if sort == "Model" => {
                    vec![AspectValue::Model(model_name.to_string())]
                }
                FieldDomain::Sort(sort) if sort == "Int" => {
                    (0..=int_bound).map(AspectValue::Int).collect()
                }
                FieldDomain::Sort(sort) if domains_by_sort.contains_key(sort) => {
                    let sort_arc: Arc<str> = Arc::from(sort.as_str());
                    domains_by_sort
                        .get(sort)
                        .ok_or_else(|| {
                            format!(
                                "universe sort {sort} not found while enumerating aspect space {}",
                                space.name
                            )
                        })?
                        .iter()
                        .cloned()
                        .map(|name| AspectValue::Univ {
                            sort: sort_arc.clone(),
                            name: Arc::from(name),
                        })
                        .collect()
                }
                FieldDomain::Sort(sort) => {
                    return Err(format!(
                        "unsupported aspect field sort {sort} for field {}; use Model, Int, one of the model universe sorts ({}), or (enum ...)",
                        field.name,
                        universe_sorts.join(", ")
                    ));
                }
                FieldDomain::Enum(values) => {
                    values.iter().cloned().map(AspectValue::Symbol).collect()
                }
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
            Some(expr) => eval_bool(expr, &assignment, models_by_name)?,
            None => true,
        };
        if keep {
            assignments.push(assignment);
        }
    }
    Ok(assignments)
}

fn univs_by_sort(model: &crate::lang::model::Model) -> HashMap<String, Vec<String>> {
    model
        .sort_interps
        .iter()
        .map(|sort_interp| {
            (
                sort_interp.sort.to_string(),
                sort_interp.elems.iter().map(|elem| elem.name.clone()).collect(),
            )
        })
        .collect()
}

fn model_universe_sorts(model: &crate::lang::model::Model) -> Vec<String> {
    model
        .sort_interps
        .iter()
        .map(|sort_interp| sort_interp.sort.to_string())
        .collect()
}

impl SemanticSpec {
    fn from_cmd(cmd: &crate::parser::ast::Semantics) -> Result<Self, String> {
    let header = sygus_parser::SExpr::from_str(&cmd.header)
        .map_err(|e| format!("failed to parse define-semantics header: {e}"))?;
    let outer = header
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics header must be a list: {}", cmd.header))?;
    let signature = match outer.as_slice() {
        [single] => single,
        _ => &header,
    };
    let items = signature
        .to_ref_vec()
        .ok_or_else(|| format!("define-semantics signature must be a list: {signature}"))?;
    let relation = items
        .first()
        .and_then(|value| value.as_symbol())
        .ok_or_else(|| format!("define-semantics signature is missing relation: {signature}"))?
        .to_string();
    let [term_arg, aspect_arg] = items
        .get(1..)
        .ok_or_else(|| format!("define-semantics signature is malformed: {signature}"))?
    else {
        return Err(format!(
            "define-semantics signature must have term and aspect arguments: {signature}"
        ));
    };
    let term_items = term_arg
        .to_ref_vec()
        .ok_or_else(|| format!("term argument must be sorted: {term_arg}"))?;
    let aspect_items = aspect_arg
        .to_ref_vec()
        .ok_or_else(|| format!("aspect argument must be sorted: {aspect_arg}"))?;
    if term_items.len() != 2 || aspect_items.len() != 2 {
        return Err(format!(
            "define-semantics arguments must have shape (name Sort): {signature}"
        ));
    }
    let term_sort = term_items[1]
        .as_symbol()
        .ok_or_else(|| format!("term argument sort must be a symbol: {}", term_items[1]))?
        .to_string();
    let aspect_var = aspect_items[0]
        .as_symbol()
        .ok_or_else(|| format!("aspect argument name must be a symbol: {}", aspect_items[0]))?
        .to_string();
    let aspect_space = aspect_items[1]
        .as_symbol()
        .ok_or_else(|| format!("aspect argument sort must be a symbol: {}", aspect_items[1]))?
        .to_string();
    let body = sygus_parser::SExpr::from_str(&cmd.body)
        .map_err(|e| format!("failed to parse define-semantics body: {e}"))?;
    let return_sort = cmd
        .return_sort
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "Bool".to_string());
    let return_kind = if return_sort == "Bool" {
        SemanticKind::Bool
    } else {
        SemanticKind::FiniteValue { sort: return_sort }
    };
    Ok(SemanticSpec {
        relation,
        term_sort,
        aspect_var,
        aspect_space,
        return_kind,
        body,
    })
}
}
