use crate::backends::smt2::ast::{HornAtom, HornRule, HornTerm};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct ConstructorAtom<'a> {
    pred: &'a str,
    fun: &'a str,
    args: &'a [HornTerm],
}

fn constructor_atom(atom: &HornAtom) -> Option<ConstructorAtom<'_>> {
    match &atom.arg {
        HornTerm::App { fun, args } if args.iter().all(|arg| matches!(arg, HornTerm::Var(_))) => {
            Some(ConstructorAtom {
                pred: &atom.pred,
                fun,
                args,
            })
        }
        HornTerm::Var(_) => None,
        HornTerm::App { .. } => None,
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

pub fn saturate(horn_rules: &[HornRule]) -> Vec<HornRule> {
    let mut rules: Vec<HornRule> = horn_rules.to_vec();
    let mut seen: HashSet<(HornAtom, Vec<HornAtom>)> = rules.iter().map(|r| r.rule_key()).collect();

    loop {
        let snapshot: Vec<HornRule> = rules.clone();
        let mut push_rules: HashMap<(&str, &str, usize), Vec<(&HornRule, ConstructorAtom<'_>)>> =
            HashMap::new();
        for rule in &snapshot {
            if let Some(head) = push_head(rule) {
                push_rules
                    .entry((head.pred, head.fun, head.args.len()))
                    .or_default()
                    .push((rule, head));
            }
        }

        let mut changed = false;
        for r in &snapshot {
            for (body_idx, pop_atom) in pop_sites(r) {
                let Some(candidates) =
                    push_rules.get(&(pop_atom.pred, pop_atom.fun, pop_atom.args.len()))
                else {
                    continue;
                };
                for (push_rule, push_head_atom) in candidates {
                    let Some(subst) = match_constructor_atom(*push_head_atom, pop_atom) else {
                        continue;
                    };

                    let new_rule = resolve_pop_clause(r, body_idx, push_rule, &subst);
                    let key = new_rule.rule_key();
                    if seen.contains(&key) {
                        continue;
                    }

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
}
