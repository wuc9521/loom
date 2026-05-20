use crate::backends::smt2::ast::{predicate_count, HornAtom, HornRule, HornTerm};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
struct ConstructorAtom<'a> {
    pred: &'a str,
    fun: &'a str,
    args: &'a [HornTerm],
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ConstructorKey {
    pred: String,
    fun: String,
    arity: usize,
}

#[derive(Clone)]
struct OwnedConstructorAtom {
    pred: String,
    fun: String,
    args: Vec<HornTerm>,
}

#[derive(Clone, Copy)]
struct PushEntry {
    rule_id: usize,
}

#[derive(Clone, Copy)]
struct PopEntry {
    rule_id: usize,
    body_idx: usize,
}

#[derive(Default)]
struct SaturationIndex {
    push_rules: HashMap<ConstructorKey, Vec<PushEntry>>,
    pop_sites: HashMap<ConstructorKey, Vec<PopEntry>>,
}

impl OwnedConstructorAtom {
    fn key(&self) -> ConstructorKey {
        ConstructorKey {
            pred: self.pred.clone(),
            fun: self.fun.clone(),
            arity: self.args.len(),
        }
    }

    fn as_borrowed(&self) -> ConstructorAtom<'_> {
        ConstructorAtom {
            pred: &self.pred,
            fun: &self.fun,
            args: &self.args,
        }
    }
}

#[derive(Clone)]
struct IndexedRule {
    rule: HornRule,
    active: bool,
}

fn constructor_atom(atom: &HornAtom) -> Option<ConstructorAtom<'_>> {
    match &atom.arg {
        HornTerm::App { fun, args } => Some(ConstructorAtom {
            pred: &atom.pred,
            fun,
            args,
        }),
        HornTerm::Var(_) => None,
    }
}

fn owned_constructor_atom(atom: &HornAtom) -> Option<OwnedConstructorAtom> {
    constructor_atom(atom).map(|ctor| OwnedConstructorAtom {
        pred: ctor.pred.to_string(),
        fun: ctor.fun.to_string(),
        args: ctor.args.to_vec(),
    })
}

fn match_constructor_atom(
    pattern: ConstructorAtom<'_>,
    candidate: ConstructorAtom<'_>,
) -> Option<HashMap<String, HornTerm>> {
    if pattern.pred != candidate.pred
        || pattern.fun != candidate.fun
        || pattern.args.len() != candidate.args.len()
    {
        return None;
    }

    let mut subst = HashMap::default();
    for (pattern_arg, candidate_arg) in pattern.args.iter().zip(candidate.args.iter()) {
        match pattern_arg {
            HornTerm::Var(name) => match subst.get(name) {
                Some(existing) if existing != candidate_arg => return None,
                Some(_) => {}
                None => {
                    subst.insert(name.clone(), candidate_arg.clone());
                }
            },
            _ if pattern_arg == candidate_arg => {}
            _ => return None,
        }
    }

    Some(subst)
}

fn resolve_pop_clause(
    pop_rule: &HornRule,
    body_idx: usize,
    push_rule: &HornRule,
    subst: &HashMap<String, HornTerm>,
) -> HornRule {
    let mut body_atoms: Vec<HornAtom> = pop_rule
        .body_atoms
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != body_idx)
        .map(|(_, a)| a.clone())
        .collect();
    for atom in &push_rule.body_atoms {
        body_atoms.push(atom.substitute(subst));
    }

    HornRule {
        body_atoms,
        head: pop_rule.head.clone(),
    }
}

fn push_head(rule: &HornRule) -> Option<ConstructorAtom<'_>> {
    constructor_atom(&rule.head)
}

fn pop_sites(rule: &HornRule) -> impl Iterator<Item = (usize, ConstructorAtom<'_>)> {
    rule.body_atoms
        .iter()
        .enumerate()
        .filter_map(|(idx, atom)| constructor_atom(atom).map(|ctor| (idx, ctor)))
}

fn insert_non_subsumed_rule(
    rules: &mut Vec<IndexedRule>,
    seen: &mut HashSet<(HornAtom, Vec<HornAtom>)>,
    new_rule: HornRule,
) -> Option<usize> {
    for existing in rules.iter().filter(|existing| existing.active) {
        if existing.rule.subsumes(&new_rule) {
            return None;
        }
    }

    let key = new_rule.rule_key();
    if seen.contains(&key) {
        return None;
    }

    let mut removed_keys = Vec::new();
    for existing in rules.iter_mut().filter(|existing| existing.active) {
        let subsumed = new_rule.subsumes(&existing.rule);
        if subsumed {
            existing.active = false;
            removed_keys.push(existing.rule.rule_key());
        }
    }
    for key in removed_keys {
        seen.remove(&key);
    }

    seen.insert(key);
    let rule_id = rules.len();
    rules.push(IndexedRule {
        rule: new_rule,
        active: true,
    });
    Some(rule_id)
}

fn index_rule(indices: &mut SaturationIndex, rules: &[IndexedRule], rule_id: usize) {
    let Some(indexed) = rules.get(rule_id) else {
        return;
    };
    if !indexed.active {
        return;
    }
    let rule = &indexed.rule;
    if let Some(head) = owned_constructor_atom(&rule.head) {
        indices
            .push_rules
            .entry(head.key())
            .or_default()
            .push(PushEntry { rule_id });
    }
    for (body_idx, atom) in rule.body_atoms.iter().enumerate() {
        if let Some(pop) = owned_constructor_atom(atom) {
            indices
                .pop_sites
                .entry(pop.key())
                .or_default()
                .push(PopEntry { rule_id, body_idx });
        }
    }
}

fn active_rule_count(rules: &[IndexedRule]) -> usize {
    rules.iter().filter(|rule| rule.active).count()
}

fn active_predicate_count(rules: &[IndexedRule]) -> usize {
    let active = rules
        .iter()
        .filter(|rule| rule.active)
        .map(|rule| rule.rule.clone())
        .collect::<Vec<_>>();
    predicate_count(&active)
}

fn active_push_count(rules: &[IndexedRule]) -> usize {
    rules.iter()
        .filter(|rule| rule.active && push_head(&rule.rule).is_some())
        .count()
}

fn active_pop_count(rules: &[IndexedRule]) -> usize {
    rules.iter()
        .filter(|rule| rule.active)
        .map(|rule| pop_sites(&rule.rule).count())
        .sum()
}

pub fn saturate(horn_rules: &[HornRule]) -> Vec<HornRule> {
    let mut rules: Vec<IndexedRule> = horn_rules
        .iter()
        .cloned()
        .map(|rule| IndexedRule { rule, active: true })
        .collect();
    let mut seen: HashSet<(HornAtom, Vec<HornAtom>)> =
        horn_rules.iter().map(|r| r.rule_key()).collect();
    let initial_predicates = predicate_count(horn_rules);
    let mut indices = SaturationIndex::default();
    let mut pending = VecDeque::new();
    let mut last_logged_rule_count = None;

    for rule_id in 0..rules.len() {
        index_rule(&mut indices, &rules, rule_id);
        pending.push_back(rule_id);
    }

    while !pending.is_empty() {
        let active_rules = active_rule_count(&rules);
        if last_logged_rule_count != Some(active_rules) {
            log::info!(
                "Saturation with {} rules, {} predicates, {} push candidates, {} pop sites",
                active_rules,
                active_predicate_count(&rules),
                active_push_count(&rules),
                active_pop_count(&rules)
            );
            last_logged_rule_count = Some(active_rules);
        }
        let Some(rule_id) = pending.pop_front() else { break; };
        if !rules.get(rule_id).is_some_and(|rule| rule.active) {
            continue;
        }

        let rule = rules[rule_id].rule.clone();

        if let Some(push_head_atom) = owned_constructor_atom(&rule.head) {
            if let Some(pops) = indices.pop_sites.get(&push_head_atom.key()).cloned() {
                for pop in pops {
                    if !rules.get(pop.rule_id).is_some_and(|r| r.active) {
                        continue;
                    }
                    let pop_rule = &rules[pop.rule_id].rule;
                    let Some(candidate) = pop_rule
                        .body_atoms
                        .get(pop.body_idx)
                        .and_then(owned_constructor_atom)
                    else {
                        continue;
                    };
                    let Some(subst) =
                        match_constructor_atom(push_head_atom.as_borrowed(), candidate.as_borrowed())
                    else {
                        continue;
                    };
                    let new_rule = resolve_pop_clause(pop_rule, pop.body_idx, &rule, &subst);
                    if let Some(new_id) = insert_non_subsumed_rule(&mut rules, &mut seen, new_rule)
                    {
                        index_rule(&mut indices, &rules, new_id);
                        pending.push_back(new_id);
                    }
                }
            }
        }

        for (body_idx, pop_atom) in rule
            .body_atoms
            .iter()
            .enumerate()
            .filter_map(|(idx, atom)| owned_constructor_atom(atom).map(|ctor| (idx, ctor)))
        {
            if let Some(pushes) = indices.push_rules.get(&pop_atom.key()).cloned() {
                for push in pushes {
                    if !rules.get(push.rule_id).is_some_and(|r| r.active) {
                        continue;
                    }
                    let push_rule = &rules[push.rule_id].rule;
                    let Some(push_head_atom) = owned_constructor_atom(&push_rule.head) else {
                        continue;
                    };
                    let Some(subst) =
                        match_constructor_atom(push_head_atom.as_borrowed(), pop_atom.as_borrowed())
                    else {
                        continue;
                    };
                    let new_rule = resolve_pop_clause(&rule, body_idx, push_rule, &subst);
                    if let Some(new_id) =
                        insert_non_subsumed_rule(&mut rules, &mut seen, new_rule)
                    {
                        index_rule(&mut indices, &rules, new_id);
                        pending.push_back(new_id);
                    }
                }
            }
        }
    }

    let filtered = rules
        .into_iter()
        .filter_map(|indexed| {
            let rule = indexed.rule;
            (indexed.active && !rule.is_pop_clause() && rule.is_one_way_compatible()).then_some(rule)
        })
        .collect::<Vec<_>>();
    log::info!(
        "Saturation complete with {} rules (from {}), {} predicates before filtering and {} after filtering",
        filtered.len(),
        horn_rules.len(),
        initial_predicates,
        predicate_count(&filtered)
    );
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::smt2::ast::{HornProg, HornTerm};

    fn atom(pred: &str, arg: HornTerm) -> HornAtom {
        HornAtom {
            pred: pred.to_string(),
            arg,
        }
    }

    #[test]
    fn tata_eg_7_6_4() {
        let rules = vec![
            HornRule {
                body_atoms: vec![
                    atom("P5", HornTerm::var("x")),
                    atom("P2", HornTerm::var("y")),
                ],
                head: atom(
                    "P1",
                    HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
                ),
            },
            HornRule {
                body_atoms: vec![
                    atom("P2", HornTerm::var("x")),
                    atom("P2", HornTerm::var("y")),
                ],
                head: atom(
                    "P5",
                    HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
                ),
            },
            HornRule {
                body_atoms: vec![],
                head: atom("P2", HornTerm::var("a")),
            },
            HornRule {
                body_atoms: vec![atom("P2", HornTerm::var("x"))],
                head: atom(
                    "P1",
                    HornTerm::app("f", vec![HornTerm::var("a"), HornTerm::var("x")]),
                ),
            },
            HornRule {
                body_atoms: vec![
                    atom("P1", HornTerm::var("x")),
                    atom("P2", HornTerm::var("y")),
                ],
                head: atom(
                    "P3",
                    HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
                ),
            },
            HornRule {
                body_atoms: vec![
                    atom("P3", HornTerm::var("x")),
                    atom("P1", HornTerm::var("x")),
                ],
                head: atom("P4", HornTerm::var("x")),
            },
            HornRule {
                body_atoms: vec![atom(
                    "P4",
                    HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
                )],
                head: atom("P2", HornTerm::var("x")),
            },
            HornRule {
                body_atoms: vec![atom(
                    "P4",
                    HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
                )],
                head: atom("P1", HornTerm::var("y")),
            },
        ];

        let prog = HornProg {
            decls: vec![],
            rules,
            queries: vec![],
            tail: vec![],
            predicates: Default::default(),
        };
        println!("Original program:");
        for rule in &prog.rules {
            println!("{}", rule);
        }
        let saturated = saturate(&prog.rules);

        println!("\n\n");
        println!("Saturated program:");
        for rule in &saturated {
            println!("{}", rule);
        }
    }

    #[test]
    fn repeated_push_variable_must_match_same_term() {
        let pattern_atom = atom(
            "P",
            HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("x")]),
        );
        let candidate_atom = atom(
            "P",
            HornTerm::app("f", vec![HornTerm::var("a"), HornTerm::var("b")]),
        );
        let pattern =
            constructor_atom(&pattern_atom).expect("pattern atom should be a constructor app");
        let candidate =
            constructor_atom(&candidate_atom).expect("candidate atom should be a constructor app");

        assert!(match_constructor_atom(pattern, candidate).is_none());
    }

    #[test]
    fn push_body_is_instantiated_with_matched_terms() {
        let push_rule = HornRule {
            body_atoms: vec![atom(
                "Q",
                HornTerm::app("g", vec![HornTerm::var("x"), HornTerm::var("y")]),
            )],
            head: atom(
                "P",
                HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
            ),
        };
        let pop_rule = HornRule {
            body_atoms: vec![atom(
                "P",
                HornTerm::app("f", vec![HornTerm::var("a"), HornTerm::var("b")]),
            )],
            head: atom("R", HornTerm::var("a")),
        };

        let subst = match_constructor_atom(
            push_head(&push_rule).expect("push rule head should be a constructor app"),
            constructor_atom(&pop_rule.body_atoms[0])
                .expect("pop body atom should be a constructor app"),
        )
        .expect("constructor matching should succeed for this test");
        let resolved = resolve_pop_clause(&pop_rule, 0, &push_rule, &subst);

        assert_eq!(resolved.head, atom("R", HornTerm::var("a")));
        assert_eq!(
            resolved.body_atoms,
            vec![atom(
                "Q",
                HornTerm::app("g", vec![HornTerm::var("a"), HornTerm::var("b")]),
            )]
        );
    }

    #[test]
    fn pop_site_can_match_nested_constructor_argument() {
        let push_rule = HornRule {
            body_atoms: vec![
                atom("Q1", HornTerm::var("x")),
                atom("Q2", HornTerm::var("y")),
            ],
            head: atom(
                "P",
                HornTerm::app("f", vec![HornTerm::var("x"), HornTerm::var("y")]),
            ),
        };
        let pop_rule = HornRule {
            body_atoms: vec![atom(
                "P",
                HornTerm::app("f", vec![HornTerm::app("g", vec![]), HornTerm::var("z")]),
            )],
            head: atom("R", HornTerm::var("z")),
        };

        let subst = match_constructor_atom(
            push_head(&push_rule).expect("push rule head should be a constructor app"),
            constructor_atom(&pop_rule.body_atoms[0])
                .expect("pop body atom should be recognized as a constructor app"),
        )
        .expect("constructor matching should succeed for nested candidate arguments");
        let resolved = resolve_pop_clause(&pop_rule, 0, &push_rule, &subst);

        assert_eq!(
            resolved.body_atoms,
            vec![
                atom("Q1", HornTerm::app("g", vec![])),
                atom("Q2", HornTerm::var("z")),
            ]
        );
        assert_eq!(resolved.head, atom("R", HornTerm::var("z")));
    }

    #[test]
    fn cfg_style_parent_pop_clause_saturates() {
        let rules = vec![
            HornRule {
                body_atoms: vec![atom("qRhsBase", HornTerm::var("x"))],
                head: atom(
                    "qCfgResetS",
                    HornTerm::app("cfg.top.S", vec![HornTerm::var("x"), HornTerm::var("y")]),
                ),
            },
            HornRule {
                body_atoms: vec![atom("qRulesFindS", HornTerm::var("y"))],
                head: atom(
                    "qCfgResetS",
                    HornTerm::app("cfg.top.S", vec![HornTerm::var("x"), HornTerm::var("y")]),
                ),
            },
            HornRule {
                body_atoms: vec![atom(
                    "qCfgResetS",
                    HornTerm::app(
                        "cfg.top.S",
                        vec![HornTerm::app("cfg.rhs.S", vec![]), HornTerm::var("__up1")],
                    ),
                )],
                head: atom("qRhsBase", HornTerm::app("cfg.rhs.S", vec![])),
            },
        ];

        let saturated = saturate(&rules);
        assert!(
            saturated.iter().all(HornRule::is_one_way_compatible),
            "expected only one-way-compatible clauses after saturation, got {:?}",
            saturated
        );
        assert!(
            saturated.iter().all(|rule| !rule.is_pop_clause()),
            "expected all pop clauses to be removed, got {:?}",
            saturated
        );
    }
}
