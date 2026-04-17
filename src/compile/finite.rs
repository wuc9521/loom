use crate::{
    compile::{
        expr::{eval_bool, eval_sygus_int},
        types::{
            AspectKey, AspectValue, BitMapping, ConstraintBit, Encoding, FieldDecl, FieldDomain,
            FiniteSpace, SemanticConstraint,
        },
    },
    lang::{
        eval::{collect_loaded_models, get_domain_for_sort, max_domain_len},
        model::Model,
    },
    parser::{DelisCmd, DelisProg},
};
use itertools::Itertools;
use lexpr::Value;
use std::collections::HashMap;
use sygus_parser::SyGuSTerm;

pub fn compile(problem: &DelisProg) -> Result<Encoding, String> {
    let finite_cmds = problem
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DelisCmd::FiniteAspect(cmd) => Some(cmd),
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
            DelisCmd::Semantics(cmd) => Some(cmd),
            _ => None,
        })
        .collect::<Vec<_>>();

    if finite_cmds.is_empty() {
        return Err("compilation requires at least one define-finite".to_string());
    }
    if finite_cmds.len() != 1 {
        return Err(format!(
            "compilation currently supports exactly one define-finite aspect space, got {}",
            finite_cmds.len()
        ));
    }
    if semantics.is_empty() {
        return Err("compilation requires at least one define-semantics".to_string());
    }
    if semantics.len() != 1 {
        return Err(format!(
            "compilation currently supports exactly one define-semantics declaration, got {}",
            semantics.len()
        ));
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
        .ok_or_else(|| "compilation requires a finite aspect space".to_string())?;
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
            for assignment in
                enumerate_aspects(aspect_space, model.name.as_str(), model, &models_by_name)?
            {
                let bit = bit_mappings.len();
                let fields = assignment
                    .iter()
                    .filter(|(_, value)| !matches!(value, AspectValue::Model(_)))
                    .map(|(name, value)| (name.clone(), value.clone()))
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

    let needs_saturation = semantics
        .iter()
        .flat_map(|cmd| cmd.body.iter())
        .map(|fragment| lexpr::from_str(fragment))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to parse define-semantics body for navigation: {e}"))?
        .iter()
        .any(contains_parent_navigation);

    Ok(Encoding {
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

fn contains_parent_navigation(expr: &Value) -> bool {
    let Some(items) = expr.to_ref_vec() else {
        return false;
    };
    items.first().and_then(|item| item.as_symbol()) == Some("parent")
        || items.into_iter().any(contains_parent_navigation)
}

fn collect_semantic_constraints(
    problem: &DelisProg,
    semantic_relations: &[String],
    aspect_space: &FiniteSpace,
    models_by_name: &HashMap<String, &Model>,
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
    let universe_sorts = model_universe_sorts(model);

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
            FieldDomain::Sort(sort) if get_domain_for_sort(model, sort).is_some() => {
                let domain = get_domain_for_sort(model, sort).unwrap();
                let value = arg.to_string();
                if !domain.contains(&value) {
                    return Err(format!(
                        "aspect field {} expected an element of universe sort {} but got {value}",
                        field.name, sort
                    ));
                }
                AspectValue::Univ {
                    sort: sort.clone(),
                    name: value,
                }
            }
            FieldDomain::Sort(sort) => {
                return Err(format!(
                    "unsupported finite field sort {sort} for field {}; use Model, Int, one of the model universe sorts ({}), or (enum ...)",
                    field.name,
                    universe_sorts.join(", ")
                ));
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
                "constraint aspect is outside finite space {}: {}",
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

fn parse_finite_space(cmd: &crate::parser::ast::FiniteAspect) -> Result<FiniteSpace, String> {
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
    space: &FiniteSpace,
    model_name: &str,
    model: &crate::lang::model::Model,
    models_by_name: &HashMap<String, &crate::lang::model::Model>,
) -> Result<Vec<Vec<(String, AspectValue)>>, String> {
    let universe_sorts = model_universe_sorts(model);
    let int_bound = max_domain_len(model);
    let domains = space
        .fields
        .iter()
        .map(|field| {
            let values = match &field.domain {
                FieldDomain::Sort(sort) if sort == "Model" => {
                    vec![AspectValue::Model(model_name.to_string())]
                }
                FieldDomain::Sort(sort) if sort == "Int" => {
                    (0..=int_bound).map(AspectValue::Int).collect()
                }
                FieldDomain::Sort(sort) if get_domain_for_sort(model, sort).is_some() => {
                    get_domain_for_sort(model, sort)
                        .unwrap()
                        .into_iter()
                        .map(|name| AspectValue::Univ {
                            sort: sort.clone(),
                            name,
                        })
                        .collect()
                }
                FieldDomain::Sort(sort) => {
                    return Err(format!(
                        "unsupported finite field sort {sort} for field {}; use Model, Int, one of the model universe sorts ({}), or (enum ...)",
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

fn model_universe_sorts(model: &crate::lang::model::Model) -> Vec<String> {
    model
        .univ_defs
        .iter()
        .map(|univ| univ.sort.to_string())
        .collect()
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
