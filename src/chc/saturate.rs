use crate::chc::ast::{CtorPat, HornAtom, HornRule};
use std::collections::{HashMap, HashSet};

fn resolve_pop_clause(
    pop_rule: &HornRule,
    body_idx: usize,
    pop_pat: &CtorPat,
    push_rule: &HornRule,
) -> Option<HornRule> {
    let push_pat = push_rule.push_head()?;
    let mut subst = HashMap::new();
    for (from, to) in push_pat.vars.iter().zip(pop_pat.vars.iter()) {
        subst.insert(from.clone(), to.clone());
    }

    let mut body_atoms: Vec<HornAtom> = pop_rule
        .body_atoms
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != body_idx)
        .map(|(_, a)| a.clone())
        .collect();
    for atom in &push_rule.body_atoms {
        body_atoms.push(atom.substitute(&subst));
    }

    //TODO: merge vars.
    Some(HornRule {
        body_atoms,
        head: pop_rule.head.clone(),
    })
}

pub fn saturate(horn_rules: &[HornRule]) -> Vec<HornRule> {
    let mut rules: Vec<HornRule> = horn_rules.to_vec();
    let mut seen: HashSet<(HornAtom, Vec<HornAtom>)> = rules.iter().map(|r| r.rule_key()).collect();

    loop {
        // make a snapshot of the rules to avoid modifying the rules while iterating over them.
        let snapshot: Vec<HornRule> = rules.clone();
        // collect all push rules and their push heads.
        let push_rules: Vec<(&HornRule, CtorPat)> = snapshot
            .iter()
            .filter_map(|r| r.push_head().map(|h| (r, h)))
            .collect();
        // iterate over all rules and try to resolve their pop sites.
        let mut changed = false;
        for r in &snapshot {
            for (body_idx, pop_pat) in r.pop_sites() {
                // try to resolve the pop site with all push rules and their push heads.
                for (push_rule, push_pat) in &push_rules {
                    if !push_pat.matches(&pop_pat) {
                        continue;
                    }
                    if let Some(new_rule) = resolve_pop_clause(r, body_idx, &pop_pat, push_rule) {
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
    fn saturate_tata_example_7_6_4() {
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
}
