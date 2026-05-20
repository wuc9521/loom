use crate::backends::smt2::ast::{
    remove_subsumed_rules, HornAtom, HornDecl, HornProg, HornQuery, HornRule, HornTerm,
    PredicatePolarity,
};
use crate::backends::smt2::saturate::saturate;
use crate::compile::TermSignature;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sygus_parser::Sort;

#[derive(Clone)]
struct ConstructorSig {
    result_sort: String,
    arg_sorts: Vec<String>,
}

struct Flattener<'a> {
    sigs: &'a HashMap<String, ConstructorSig>,
    helper_by_pattern: HashMap<(String, HornTerm), String>,
    helper_rules: Vec<HornRule>,
    helper_sorts: HashMap<String, String>,
    next_helper_id: usize,
    next_var_id: usize,
}

impl<'a> Flattener<'a> {
    fn new(sigs: &'a HashMap<String, ConstructorSig>) -> Self {
        Self {
            sigs,
            helper_by_pattern: HashMap::default(),
            helper_rules: Vec::new(),
            helper_sorts: HashMap::default(),
            next_helper_id: 0,
            next_var_id: 0,
        }
    }

    fn fresh_var(&mut self) -> String {
        let name = format!("__flatv{}", self.next_var_id);
        self.next_var_id += 1;
        name
    }

    fn helper_pred_for(&mut self, term: &HornTerm, sort: &str) -> Result<String, String> {
        let key = (sort.to_string(), term.clone());
        if let Some(pred) = self.helper_by_pattern.get(&key) {
            return Ok(pred.clone());
        }

        let pred = format!("__flat.{}.{}", sort, self.next_helper_id);
        self.next_helper_id += 1;
        self.helper_by_pattern.insert(key, pred.clone());
        self.helper_sorts.insert(pred.clone(), sort.to_string());

        let mut body_atoms = Vec::new();
        let head_arg = self.flatten_term(term, sort, &mut body_atoms)?;
        self.helper_rules.push(HornRule {
            body_atoms,
            head: HornAtom {
                pred: pred.clone(),
                arg: head_arg,
            },
        });

        Ok(pred)
    }

    fn flatten_term(
        &mut self,
        term: &HornTerm,
        expected_sort: &str,
        body_atoms: &mut Vec<HornAtom>,
    ) -> Result<HornTerm, String> {
        match term {
            HornTerm::Var(name) => Ok(HornTerm::Var(name.clone())),
            HornTerm::App { fun, args } => {
                let ctor = self
                    .sigs
                    .get(fun)
                    .ok_or_else(|| format!("missing constructor signature for {fun}"))?;
                if ctor.result_sort != expected_sort {
                    return Err(format!(
                        "constructor {fun} has result sort {}, expected {expected_sort}",
                        ctor.result_sort
                    ));
                }
                if ctor.arg_sorts.len() != args.len() {
                    return Err(format!(
                        "constructor {fun} arity mismatch: expected {}, got {}",
                        ctor.arg_sorts.len(),
                        args.len()
                    ));
                }
                let mut flat_args = Vec::with_capacity(args.len());
                for (arg, arg_sort) in args.iter().zip(ctor.arg_sorts.iter()) {
                    match arg {
                        HornTerm::Var(name) => flat_args.push(HornTerm::Var(name.clone())),
                        _ => {
                            let helper = self.helper_pred_for(arg, arg_sort)?;
                            let fresh = self.fresh_var();
                            body_atoms.push(HornAtom {
                                pred: helper,
                                arg: HornTerm::Var(fresh.clone()),
                            });
                            flat_args.push(HornTerm::Var(fresh));
                        }
                    }
                }
                Ok(HornTerm::App {
                    fun: fun.clone(),
                    args: flat_args,
                })
            }
        }
    }

    fn flatten_atom(
        &mut self,
        atom: &HornAtom,
        pred_sorts: &HashMap<String, String>,
        body_atoms: &mut Vec<HornAtom>,
    ) -> Result<HornAtom, String> {
        let sort = pred_sorts
            .get(&atom.pred)
            .ok_or_else(|| format!("missing predicate sort for {}", atom.pred))?;
        Ok(HornAtom {
            pred: atom.pred.clone(),
            arg: self.flatten_term(&atom.arg, sort, body_atoms)?,
        })
    }

    fn flatten_rule(
        &mut self,
        rule: &HornRule,
        pred_sorts: &HashMap<String, String>,
    ) -> Result<HornRule, String> {
        let mut prefix = Vec::new();
        let head = self.flatten_atom(&rule.head, pred_sorts, &mut prefix)?;
        let mut body_atoms = prefix;
        for atom in &rule.body_atoms {
            let flat = self.flatten_atom(atom, pred_sorts, &mut body_atoms)?;
            body_atoms.push(flat);
        }
        Ok(HornRule { body_atoms, head })
    }

    fn flatten_query(
        &mut self,
        query: &HornQuery,
        pred_sorts: &HashMap<String, String>,
    ) -> Result<HornQuery, String> {
        let mut body_atoms = Vec::new();
        for atom in &query.body_atoms {
            let flat = self.flatten_atom(atom, pred_sorts, &mut body_atoms)?;
            body_atoms.push(flat);
        }
        Ok(HornQuery { body_atoms })
    }
}

fn constructor_sigs(sig: &TermSignature) -> HashMap<String, ConstructorSig> {
    sig.constructors
        .iter()
        .map(|cons| {
            (
                cons.name.clone(),
                ConstructorSig {
                    result_sort: cons.result_sort.clone(),
                    arg_sorts: cons.arg_sorts.clone(),
                },
            )
        })
        .collect()
}

fn predicate_sorts(program: &HornProg) -> HashMap<String, String> {
    let mut out = HashMap::default();
    for decl in &program.decls {
        let HornDecl::DeclareRel(name, args) = decl else {
            continue;
        };
        if let Some(sort) = args.first() {
            out.insert(name.clone(), sort.to_string());
        }
    }
    out
}

fn ensure_predicate_metadata(program: &mut HornProg) {
    let pred_sorts = predicate_sorts(program);
    for pred in program.live_predicates() {
        if program.predicates.contains_key(&pred) {
            continue;
        }
        let Some(sort) = pred_sorts.get(&pred) else {
            continue;
        };
        program.register_predicate(pred, "", PredicatePolarity::Positive, sort.clone());
    }
}

fn flatten_program(program: &HornProg, sig: &TermSignature) -> Result<HornProg, String> {
    let ctor_sigs = constructor_sigs(sig);
    let mut pred_sorts = predicate_sorts(program);
    let mut flattener = Flattener::new(&ctor_sigs);

    let mut out = program.clone();
    out.rules.clear();
    out.queries.clear();

    for rule in &program.rules {
        out.rules.push(flattener.flatten_rule(rule, &pred_sorts)?);
    }
    for query in &program.queries {
        out.queries.push(flattener.flatten_query(query, &pred_sorts)?);
    }
    out.rules.extend(flattener.helper_rules.clone());

    let mut declared = HashSet::default();
    for decl in &out.decls {
        if let HornDecl::DeclareRel(name, _) = decl {
            declared.insert(name.clone());
        }
    }
    for (pred, sort) in &flattener.helper_sorts {
        pred_sorts.insert(pred.clone(), sort.clone());
        if !declared.contains(pred) {
            out.decls
                .push(HornDecl::DeclareRel(pred.clone(), vec![Sort::from_name(sort)]));
        }
        out.register_predicate(
            pred.clone(),
            pred.clone(),
            PredicatePolarity::Positive,
            sort.clone(),
        );
    }

    Ok(out)
}

pub fn saturate_program(program: &HornProg, sig: &TermSignature) -> Result<HornProg, String> {
    let mut flat = flatten_program(program, sig)?;
    flat.rules = remove_subsumed_rules(flat.rules);
    flat.rules = saturate(&flat.rules);
    flat.rules = remove_subsumed_rules(flat.rules);
    ensure_predicate_metadata(&mut flat);
    Ok(flat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::smt2::ast::HornProg;
    use crate::compile::to_chc_ast;
    use crate::parser::DelisCmd;

    fn atom(pred: &str, arg: HornTerm) -> HornAtom {
        HornAtom {
            pred: pred.to_string(),
            arg,
        }
    }

    #[test]
    fn flattening_removes_nested_constructor_arguments() {
        let problem = crate::parser::ast::parse(
            r#"(declare-term-types
 ((CFG 0) (Rhs 0) (Rules 0))
 (((cfg.top.S Rhs Rules))
  ((cfg.rhs.S))
  ((cfg.lhs.S Rhs Rules) (end))))"#,
            None,
        )
        .expect("term declaration should parse");
        let decl = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::TermTypes(term_types) => Some(term_types),
                _ => None,
            })
            .expect("term declaration should be present");
        let sig = TermSignature::from_declare_term_types(&decl).expect("signature should build");
        let mut prog = HornProg::default();
        prog.decls.push(HornDecl::DeclareRel(
            "Q".to_string(),
            vec![Sort::from_name("CFG")],
        ));
        prog.decls.push(HornDecl::DeclareRel(
            "R".to_string(),
            vec![Sort::from_name("Rhs")],
        ));
        prog.register_predicate("Q", "Q", PredicatePolarity::Positive, "CFG");
        prog.register_predicate("R", "R", PredicatePolarity::Positive, "Rhs");
        prog.rules.push(HornRule {
            body_atoms: vec![atom(
                "Q",
                HornTerm::app(
                    "cfg.top.S",
                    vec![
                        HornTerm::app("cfg.rhs.S", vec![]),
                        HornTerm::app(
                            "cfg.lhs.S",
                            vec![HornTerm::app("cfg.rhs.S", vec![]), HornTerm::app("end", vec![])],
                        ),
                    ],
                ),
            )],
            head: atom("R", HornTerm::app("cfg.rhs.S", vec![])),
        });

        let flat = flatten_program(&prog, &sig).expect("flattening should succeed");
        assert!(flat.rules.iter().flat_map(|rule| rule.body_atoms.iter().chain(std::iter::once(&rule.head))).all(|atom| {
            match &atom.arg {
                HornTerm::Var(_) => true,
                HornTerm::App { args, .. } => args.iter().all(|arg| matches!(arg, HornTerm::Var(_))),
            }
        }));
        assert!(flat.predicates.keys().any(|pred| pred.starts_with("__flat.")));
    }

    #[test]
    fn saturated_cfg_program_is_one_way_compatible() {
        let problem = crate::parser::ast::parse(
            r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((CFG 0) (Rhs 0) (Rules 0))
 (((cfg.top.S Rhs Rules))
  ((tok.a) (cfg.rhs.S))
  ((cfg.lhs.S Rhs Rules) (end))))
(define-finite CFG.Asp
  (record (model Model) (i Int) (j Int) (mode (enum base reset.S find.S reset.T find.T)))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((CFG.Sem (n CFG) (asp CFG.Asp)))
  ((match n
     ((cfg.top.S x y)
      (or
        (and (= (mode asp) reset.S)
             (or (Rhs.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) reset.T)
             (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T))))))))
(define-semantics
  ((Rhs.Sem (n Rhs) (asp CFG.Asp)))
  ((match n
     ((tok.a)
      (or
        (and (= (mode asp) base)
             (= (j asp) (+ (i asp) 1))
             (a (i asp)))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))
     ((cfg.rhs.S)
      (and (= (mode asp) base)
           (CFG.Sem (parent n) (CFG.Asp (model asp) (i asp) (j asp) reset.S)))))))
(define-semantics
  ((Rules.Sem (n Rules) (asp CFG.Asp)))
  ((match n
     ((cfg.lhs.S x y)
      (or
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))
        (and (= (mode asp) find.S)
             (or (Rhs.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) find.T)
             (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T)))))
     ((end) false))))
(synth-fun g () CFG)
(constraint (CFG.Sem g (CFG.Asp w1 0 (size (Pos w1)) reset.S)))
(check-synth)
"#,
            None,
        )
        .expect("problem should parse");
        let chc = to_chc_ast(&problem).expect("CHC should compile");
        let decl = problem
            .cmds
            .iter()
            .find_map(|cmd| match cmd {
                DelisCmd::TermTypes(term_types) => Some(term_types),
                _ => None,
            })
            .expect("term declaration should be present");
        let sig = TermSignature::from_declare_term_types(decl).expect("signature should build");
        let saturated = saturate_program(&chc, &sig).expect("saturation should succeed");

        assert!(saturated.rules.iter().all(HornRule::is_one_way_compatible));
        for rule in &saturated.rules {
            assert!(
                saturated.predicates.contains_key(&rule.head.pred),
                "missing predicate metadata for head {} in rule {:?}",
                rule.head.pred,
                rule
            );
            for atom in &rule.body_atoms {
                assert!(
                    saturated.predicates.contains_key(&atom.pred),
                    "missing predicate metadata for body atom {} in rule {:?}",
                    atom.pred,
                    rule
                );
            }
        }
    }
}
