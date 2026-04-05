use crate::chc::ast::{HornAtom, HornRule, HornTerm};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct ConstructorAtom<'a> {
    pred: &'a str,
    fun: &'a str,
    args: &'a [HornTerm],
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

    let mut subst = HashMap::new();
    for (pattern_arg, candidate_arg) in pattern.args.iter().zip(candidate.args.iter()) {
        let HornTerm::Var(name) = pattern_arg else {
            return None;
        };

        match subst.get(name) {
            Some(existing) if existing != candidate_arg => return None,
            Some(_) => {}
            None => {
                subst.insert(name.clone(), candidate_arg.clone());
            }
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

pub fn saturate(horn_rules: &[HornRule]) -> Vec<HornRule> {
    let mut rules: Vec<HornRule> = horn_rules.to_vec();
    let mut seen: HashSet<(HornAtom, Vec<HornAtom>)> = rules.iter().map(|r| r.rule_key()).collect();

    loop {
        let snapshot: Vec<HornRule> = rules.clone();
        let push_rules: Vec<(&HornRule, ConstructorAtom<'_>)> = snapshot
            .iter()
            .filter_map(|r| push_head(r).map(|h| (r, h)))
            .collect();

        let mut changed = false;
        for r in &snapshot {
            for (body_idx, pop_atom) in pop_sites(r) {
                for (push_rule, push_head_atom) in &push_rules {
                    let Some(subst) = match_constructor_atom(*push_head_atom, pop_atom) else {
                        continue;
                    };

                    let new_rule = resolve_pop_clause(r, body_idx, push_rule, &subst);
                    let key = new_rule.rule_key();
                    if seen.contains(&key) || rules.iter().any(|r| r.subsumes(&new_rule)) {
                        continue;
                    }

                    rules.retain(|old| !new_rule.subsumes(old));
                    seen = rules.iter().map(|r| r.rule_key()).collect();
                    seen.insert(key);
                    rules.push(new_rule);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    rules
        .into_iter()
        .filter(|rule| !rule.is_pop_clause())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chc::ast::{HornProg, HornTerm};

    fn atom(pred: &str, arg: HornTerm) -> crate::chc::ast::HornAtom {
        crate::chc::ast::HornAtom {
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
        let pattern = constructor_atom(&pattern_atom).unwrap();
        let candidate = constructor_atom(&candidate_atom).unwrap();

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
            push_head(&push_rule).unwrap(),
            constructor_atom(&pop_rule.body_atoms[0]).unwrap(),
        )
        .unwrap();
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
}
