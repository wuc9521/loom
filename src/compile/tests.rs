use super::*;

fn ensure_opts() {
    init_options(CompileOptions {
        optimize: false,
        bisim: false,
    });
}

fn parse(input: &str) -> LoomProg {
    ensure_opts();
    crate::parser::LoomProg::parse(input, None).expect("compilation smoke program should parse")
}

#[test]
fn eval_symbols_matches_unary_model_interpretations() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun p0 (Pos) Bool)
(define-model w
  (interp-sort Pos ($0 $1 $2))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  ))))
"#,
    );

    let model = problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            crate::parser::ast::LoomCmd::DefineModel(model) => Some(model),
            _ => None,
        })
        .expect("test model should exist");

    let t0 = model
        .eval_symbols("p0", &["$0".to_string()])
        .expect("lookup for $0 should exist");
    let t1 = model
        .eval_symbols("p0", &["$1".to_string()])
        .expect("lookup for $1 should exist");
    let t2 = model
        .eval_symbols("p0", &["$2".to_string()])
        .expect("lookup for $2 should exist");

    assert!(crate::compile::semantics::interped_bool(t0));
    assert!(crate::compile::semantics::interped_bool(t1));
    assert!(!crate::compile::semantics::interped_bool(t2));
}

#[test]
fn cloned_models_preserve_stable_cache_identity() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun p0 (Pos) Bool)
(define-model w
  (interp-sort Pos ($0 $1))
  (interp-fun p0 (match (
    (($0) true)
    ((_) false)
  ))))
"#,
    );

    let model = problem
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            crate::parser::ast::LoomCmd::DefineModel(model) => Some(model),
            _ => None,
        })
        .expect("test model should exist");
    let cloned = model.clone();
    let reparsed = parse(
        r#"

(declare-sort Pos 0)
(declare-fun p0 (Pos) Bool)
(define-model w
  (interp-sort Pos ($0 $1))
  (interp-fun p0 (match (
    (($0) true)
    ((_) false)
  ))))
"#,
    );
    let reparsed_model = reparsed
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            crate::parser::ast::LoomCmd::DefineModel(model) => Some(model),
            _ => None,
        })
        .expect("reparsed model should exist");

    assert_eq!(model.cache_id, cloned.cache_id);
    assert_ne!(model.cache_id, reparsed_model.cache_id);
    assert!(crate::compile::semantics::interped_bool(
        cloned
            .eval_symbols("p0", &["$0".to_string()])
            .expect("lookup for cloned model should exist")
    ));
}

#[test]
fn enumerates_subword_aspects_and_constraint_polarities() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (interp-sort Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (RegLan.Asp w1 0 (size (Pos w1)))))
(constraint (not (Regex.Sem r (RegLan.Asp w5 0 (size (Pos w5))))))
(check-synth)
"#,
    );

    let summary = problem.encode().expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 16);
    assert_eq!(problem.bv_sz().expect("BV length should compile"), 16);
    assert_eq!(summary.semantic_relations, vec!["Regex.Sem"]);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert_eq!(summary.constraint_bits[0].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[0].model, "w1");
    assert!(summary.constraint_bits[0].expected);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    assert_eq!(summary.constraint_bits[1].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[1].model, "w5");
    assert!(!summary.constraint_bits[1].expected);
    assert_eq!(summary.constraint_bits[1].bit, 9);
}

#[test]
fn aspect_fields_are_not_named_primitives() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (interp-sort Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects Span
  (record (m Model) (left Int) (right Int))
  :where (and (<= 0 left) (<= left right) (<= right (size (Pos m)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp Span)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (Span w 0 (size (Pos w)))))
(check-synth)
"#,
    );

    let summary = problem.encode().expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 6);
    assert_eq!(summary.constraint_bits.len(), 1);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    let fields = summary.bit_mappings[2]
        .aspect
        .fields
        .iter()
        .map(|(name, value)| (name.clone(), value.display()))
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        vec![
            ("left".to_string(), "0".to_string()),
            ("right".to_string(), "2".to_string())
        ]
    );
}

#[test]
fn model_less_aspects_use_with_model_context() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (interp-sort Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp
  (record (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (with-model w1 (Regex.Sem r (RegLan.Asp 0 (size (Pos model))))))
(constraint (not (with-model w5 (Regex.Sem r (RegLan.Asp 0 (size (Pos model)))))))
(check-synth)
"#,
    );

    let summary = problem.encode().expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 16);
    assert_eq!(summary.semantic_relations, vec!["Regex.Sem"]);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert_eq!(summary.constraint_bits[0].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[0].model, "w1");
    assert!(summary.constraint_bits[0].expected);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    assert_eq!(summary.constraint_bits[1].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[1].model, "w5");
    assert!(!summary.constraint_bits[1].expected);
    assert_eq!(summary.constraint_bits[1].bit, 9);
}

#[test]
fn enum_aspect_symbols_compare_as_finite_values() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(define-model w
  (interp-sort Pos ($0)))
(declare-term-types ((Root 0)) (((leaf))))
(define-aspects Asp
  (record (model Model) (mode (enum base reset))))
(define-semantics
  ((Root.Sem (r Root) (asp Asp)))
  ((match r
     ((leaf) (= (mode asp) base)))))
(synth-fun r () Root)
(constraint (Root.Sem r (Asp w base)))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("enum aspect equality should compile");

    assert!(!sygus.is_empty());
}

#[test]
fn parses_direct_semantic_constraints() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (interp-sort Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (RegLan.Asp w1 0 (size (Pos w1)))))
(constraint (not (Regex.Sem r (RegLan.Asp w5 0 (size (Pos w5))))))
(check-synth)
"#,
    );

    let summary = problem.encode().expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 16);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert_eq!(summary.constraint_bits[0].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[0].model, "w1");
    assert!(summary.constraint_bits[0].expected);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    assert_eq!(summary.constraint_bits[1].relation, "Regex.Sem");
    assert_eq!(summary.constraint_bits[1].model, "w5");
    assert!(!summary.constraint_bits[1].expected);
    assert_eq!(summary.constraint_bits[1].bit, 9);
}

#[test]
fn semantic_predicates_use_aspect_projection() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0 $1))
  (interp-fun a (match ((($0) true) ((_) false))))
  (interp-fun b (match ((($1) true) ((_) false)))))
(define-model w5
  (interp-sort Pos ($0 $1 $2))
  (interp-fun a (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun b (match (((_) false)))))
(declare-term-types ((Regex 0)) (((a) (b) (re.++ Regex Regex))))
(define-aspects RegLan.Asp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r
 ((a) (and (= (j asp) (+ (i asp) 1)) (a (i asp))))
 ((b) (and (= (j asp) (+ (i asp) 1)) (b (i asp))))
 ((re.++ x y)
  (exists ((k (int-range (i asp) (j asp))))
    (and (Regex.Sem x (RegLan.Asp (model asp) (i asp) k))
         (Regex.Sem y (RegLan.Asp (model asp) k (j asp)))))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (RegLan.Asp w1 0 (size (Pos w1)))))
(constraint (not (Regex.Sem r (RegLan.Asp w5 0 (size (Pos w5))))))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("projected semantic predicates should compile");

    assert!(!sygus.is_empty());
    let output = sygus
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !output.contains(".Regex.Sem."),
        "SyGuS output should inline semantic aspect helper functions"
    );
}

#[test]
fn semantic_compilation_loads_models_referenced_only_by_axioms() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects Asp
  (record (model Model) (i Int))
  :where (= i 0))
(define-semantics
  ((Regex.Sem (r Regex) (asp Asp)))
  ((match r
     ((a) (a (i asp))))))
(synth-fun r () Regex)
(declare-axiom (Regex.Sem r (Asp w 0)))
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("axiom-referenced model should be enumerated");

    assert_eq!(summary.semantic_relations, vec!["Regex.Sem"]);
    assert_eq!(summary.constraint_bits.len(), 0);
    assert_eq!(summary.bit_mappings.len(), 1);
    assert_eq!(summary.bit_mappings[0].aspect.model, "w");
}

#[test]
fn detects_two_way_navigation() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Root 0)) (((cfg.top.S Root Root))))
(define-aspects CfgAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r ((cfg.top.S x y) (Cfg.Sem (parent r) asp)))))
(synth-fun r () Root)
(constraint (Cfg.Sem r (CfgAsp w1 0 (size (Pos w1)))))
(check-synth)
"#,
    );

    let semantic_program = problem
        .semantic_program()
        .expect("semantic program should compile");
    assert!(
        semantic_program
            .semantics
            .iter()
            .any(|spec| super::aspects::contains_parent_navigation(&spec.body)),
        "expected a semantic spec that uses (parent ...)"
    );
}

#[test]
fn parent_bearing_negatives_require_manual_dual() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Root 0)) (((leaf) (cfg.top.S Root Root))))
(define-aspects CfgAsp
  (record (i Int))
  :where (and (<= 0 i) (<= i (size (Pos model)))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r
     ((leaf) false)
     ((cfg.top.S x y) (Cfg.Sem (parent x) (CfgAsp 0))))))
(synth-fun r () Root)
(constraint (not (with-model w1 (Cfg.Sem r (CfgAsp 0)))))
(check-synth)
"#,
    );

    let err = problem
        .semantic_program()
        .expect_err("parent-bearing negative without manual dual should error");

    assert!(err.contains("manual dual"));
}

#[test]
fn parent_navigation_to_sygus_uses_parent_encoding() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Root 0)) (((leaf) (cfg.top.S Root Root))))
(define-aspects CfgAsp
  (record (model Model) (i Int))
  :where (and (<= 0 i) (<= i (size (Pos model)))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r
     ((leaf) false)
     ((cfg.top.S x y) (Cfg.Sem (parent x) asp)))))
(synth-fun r () Root)
(constraint (Cfg.Sem r (CfgAsp w1 0)))
(check-synth)
"#,
    );

    let sygus = String::from_utf8(
        problem
            .to_sygus_bytes()
            .expect("parent-bearing program should emit the parent SyGuS encoding"),
    )
    .expect("SyGuS output should be UTF-8");

    assert!(
        sygus.contains("(Constant"),
        "parent-bearing to_sygus must use the parent certificate encoding (Constant holes)"
    );
}

#[test]
fn aspects_and_predicates_use_declared_universe_sorts() {
    let problem = parse(
        r#"

(declare-sort A 0)
(declare-sort B 0)
(declare-fun P (A B) Bool)
(define-model m
  (interp-sort B ($b0))
  (interp-sort A ($a0 $a1))
  (interp-fun P (match ((($a0 $b0) true) ((_ _) false)))))
(declare-term-types ((Formula 0)) (((P.x.y))))
(define-aspects FO.Asp
  (record (model Model) (x A) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FO.Asp)))
  ((match phi
     ((P.x.y) (P (x asp) (y asp))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FO.Asp m $a0 $b0)))
(constraint (not (FO.Sem phi (FO.Asp m $a1 $b0))))
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("many-sorted aspects should compile");

    assert_eq!(summary.bit_mappings.len(), 2);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert_eq!(summary.constraint_bits[0].relation, "FO.Sem");
    assert!(summary.constraint_bits[0].expected);
    assert_eq!(summary.constraint_bits[1].relation, "FO.Sem");
    assert!(!summary.constraint_bits[1].expected);

    let sygus = problem
        .to_sygus()
        .expect("many-sorted predicates should lower to SyGuS");
    assert!(!sygus.is_empty());
}

#[test]
fn quantified_domains_can_select_a_non_primary_universe_sort() {
    let problem = parse(
        r#"

(declare-sort A 0)
(declare-sort B 0)
(declare-fun P (A B) Bool)
(define-model m
  (interp-sort B ($b0))
  (interp-sort A ($a0 $a1))
  (interp-fun P (match ((($a1 $b0) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0))
 (((fol.exists.a Formula)
   (P.x.y))))
(define-aspects FO.Asp
  (record (model Model) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FO.Asp)))
  ((match phi
     ((fol.exists.a x)
      (exists ((v A))
        (FO.Sem x (FO.Asp (model asp) (y asp)))))
     ((P.x.y)
      (exists ((v A))
        (P v (y asp)))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FO.Asp m $b0)))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("typed quantified domains should lower to SyGuS");
    assert!(!sygus.is_empty());
}

#[test]
fn bare_sort_quantifiers_use_current_model() {
    let problem = parse(
        r#"

(declare-sort A 0)
(declare-sort B 0)
(declare-fun P (A B) Bool)
(define-model m
  (interp-sort B ($b0))
  (interp-sort A ($a0 $a1))
  (interp-fun P (match ((($a1 $b0) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0))
 (((fol.forall.a Formula)
   (fol.exists.a Formula)
   (P.x.y))))
(define-aspects FO.Asp
  (record (model Model) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FO.Asp)))
  ((match phi
     ((fol.forall.a x)
      (forall ((v A))
        (FO.Sem x (FO.Asp (model asp) (y asp)))))
     ((fol.exists.a x)
      (exists ((v A))
        (FO.Sem x (FO.Asp (model asp) (y asp)))))
     ((P.x.y)
      (exists ((v A))
        (P v (y asp)))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FO.Asp m $b0)))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("bare-sort quantifier domains should lower to SyGuS");
    assert!(!sygus.is_empty());
}

#[test]
fn model_terms_support_constants_functions_and_nested_applications() {
    let problem = parse(
        r#"

(declare-sort A 0)
(declare-sort B 0)
(declare-fun c () A)
(declare-fun flag () Bool)
(declare-fun f (A) B)
(declare-fun g (B) A)
(declare-fun P (A) Bool)
(define-model m
  (interp-sort A (@A.0 @A.1))
  (interp-sort B (@B_0))
  (interp-fun c (@A.0))
  (interp-fun flag (true))
  (interp-fun f (match (((@A.0) @B_0) ((@A.1) @B_0))))
  (interp-fun g (match (((@B_0) @A.1))))
  (interp-fun P (match (((@A.1) true) ((_) false)))))
(declare-term-types
 ((Formula 0))
 (((flag.atom)
   (nested.pred))))
(define-aspects FO.Asp
  (record (model Model)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FO.Asp)))
  ((match phi
     ((flag.atom) (flag))
     ((nested.pred) (P (g (f c)))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FO.Asp m)))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("typed model terms should lower to SyGuS");

    assert!(!sygus.is_empty());
}

#[test]
fn model_terms_support_binary_functions_and_relations_as_bool_functions() {
    let problem = parse(
        r#"

(declare-sort A 0)
(declare-sort B 0)
(declare-fun h (A B) A)
(declare-fun R (A B) Bool)
(define-model m
  (interp-sort A (@a0 @a1))
  (interp-sort B (@b0))
  (interp-fun h (match (((@a0 @b0) @a1) ((@a1 @b0) @a0))))
  (interp-fun R (match (((@a1 @b0) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0))
 (((binary.term))))
(define-aspects FO.Asp
  (record (model Model) (x A) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FO.Asp)))
  ((match phi
     ((binary.term) (R (h (x asp) (y asp)) (y asp))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FO.Asp m @a0 @b0)))
(constraint (not (FO.Sem phi (FO.Asp m @a1 @b0))))
(check-synth)
"#,
    );

    let sygus = problem
        .to_sygus()
        .expect("binary typed model terms should compile");

    assert!(!sygus.is_empty());
}

#[test]
fn builds_separate_bool_and_finite_value_semantic_domains() {
    let problem = parse(
        r#"

(declare-sort U 0)
(define-model m
  (interp-sort U ($0 $1)))
(declare-term-types
 ((Formula 0) (TermU 0))
 (((pred TermU))
  ((var.x))))
(define-aspects EnvAsp
  (record (model Model) (x U)))
(define-semantics
  ((TermU.Sem (t TermU) (asp EnvAsp)))
   :returns U
  ((match t
     ((var.x) (x asp)))))
(define-semantics
  ((Formula.Sem (phi Formula) (asp EnvAsp)))
  ((match phi
     ((pred t) true))))
(synth-fun phi () Formula)
(constraint (Formula.Sem phi (EnvAsp m $0)))
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("mixed semantic domains should compile");

    let term_domain = summary
        .semantic_domains
        .iter()
        .find(|domain| domain.relation == "TermU.Sem")
        .expect("term semantic domain should exist");
    assert_eq!(
        term_domain.kind,
        crate::compile::types::SemanticKind::FiniteValue {
            sort: "U".to_string()
        }
    );
    assert_eq!(term_domain.bits.len(), 4);
    assert_eq!(
        term_domain
            .bits
            .iter()
            .filter(|bit| bit.value.is_some())
            .count(),
        4
    );

    let formula_domain = summary
        .semantic_domains
        .iter()
        .find(|domain| domain.relation == "Formula.Sem")
        .expect("formula semantic domain should exist");
    assert_eq!(
        formula_domain.kind,
        crate::compile::types::SemanticKind::Bool
    );
    assert_eq!(formula_domain.bits.len(), 2);
    assert_eq!(summary.bit_mappings.len(), 2);
}

#[test]
fn rejects_unknown_aspect_field_sort_instead_of_singleton_symbol() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects BadAsp
  (record (model Model) (color Color)))
(define-semantics
  ((Regex.Sem (r Regex) (asp BadAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (BadAsp w red)))
(check-synth)
"#,
    );

    let err = problem
        .encode()
        .expect_err("unknown aspect field sort should be rejected");

    assert!(err.contains("unsupported aspect field sort Color"));
}

#[test]
fn allows_multiple_aspect_spaces() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(define-model w (interp-sort Pos ($0)))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects FirstAsp (record (model Model)))
(define-aspects SecondAsp (record (model Model)))
(define-semantics
  ((Regex.Sem (r Regex) (asp FirstAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("multiple aspect spaces should be summarized");

    assert_eq!(summary.aspect_spaces, vec!["FirstAsp", "SecondAsp"]);
    assert_eq!(summary.semantic_domains.len(), 1);
    assert_eq!(summary.semantic_domains[0].aspect_space, "FirstAsp");
}

#[test]
fn allows_multiple_define_semantics_declarations() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(define-model w (interp-sort Pos ($0)))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp (record (model Model)))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r ((a) true))))
(define-semantics
  ((Regex.Other (r Regex) (asp RegLan.Asp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("multiple semantics should be summarized");

    assert_eq!(summary.semantic_relations, vec!["Regex.Sem", "Regex.Other"]);
    assert_eq!(summary.semantic_domains.len(), 2);
}

// Multi-non-terminal CFG with (parent ...): out of scope for the native
// multisynth path, which only handles a single recursive-ref constructor.
#[test]
fn typed_cfg_semantics_with_parent_navigation_compile_to_chc() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (interp-sort Pos ($0))
  (interp-fun a (match ((($0) true) ((_) false)))))
(declare-term-types
 ((CFG 0) (Rhs 0) (Rules 0))
 (
   ((cfg.top.S Rhs Rules))
   ((tok.a) (cfg.rhs.S))
   ((cfg.lhs.S Rhs Rules) (end))
 ))
(define-aspects CFG.Asp
  (record
    (model Model)
    (i Int)
    (j Int)
    (mode (enum base reset.S find.S reset.T find.T)))
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
    );

    let chc = problem.to_chc_ast().expect("typed CFG CHC should compile");

    assert!(!chc.rules.is_empty());
    let horn = chc.to_fixedpoint_query_text("Goal");
    assert!(horn.contains("CFG.Sem"));
    assert!(horn.contains("Rhs.Sem"));
    assert!(horn.contains("Rules.Sem"));
}

#[test]
fn rejects_wrong_aspect_constructor_in_semantic_body() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r
     ((a) (Regex.Sem r (WrongAsp (model asp) (i asp) (j asp)))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (RegLan.Asp w 0 (size (Pos w)))))
(check-synth)
"#,
    );

    let err = problem
        .to_sygus()
        .expect_err("wrong aspect constructor should be rejected");

    assert!(err.contains("aspect constructor must be RegLan.Asp"));
}

#[test]
fn rejects_wrong_aspect_arity_in_semantic_body() {
    let problem = parse(
        r#"

(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (interp-sort Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-aspects RegLan.Asp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp RegLan.Asp)))
  ((match r
     ((a) (Regex.Sem r (RegLan.Asp (model asp) (i asp)))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (RegLan.Asp w 0 (size (Pos w)))))
(check-synth)
"#,
    );

    let err = problem
        .to_sygus()
        .expect_err("wrong aspect arity should be rejected");

    assert!(err.contains("provides 2 fields, expected 3"));
}

#[test]
fn aspect_field_accepts_model_function_expression() {
    let problem = parse(
        r#"

(declare-sort U 0)
(declare-sort Ctr 0)
(declare-fun a (U) Bool)
(declare-fun initial () U)
(declare-fun bound (U) Ctr)
(define-model m
  (interp-sort U (s0 s1 s2))
  (interp-sort Ctr ($0 $1 $2))
  (interp-fun a (match (((s0) true) ((_) false))))
  (interp-fun initial (s0))
  (interp-fun bound (match (((s0) $2) ((s1) $1) ((s2) $0)))))
(declare-term-types ((F 0)) (((ap.a) (f.not F))))
(define-aspects Asp
  (record (model Model) (state U) (depth Ctr)))
(define-semantics
  ((F.Sem (f F) (asp Asp)))
  ((match f
     ((ap.a) (a (state asp)))
     ((f.not x) (not (F.Sem x asp))))))
(synth-fun phi () F)
(constraint (F.Sem phi (Asp m initial (bound initial))))
(check-synth)
"#,
    );

    let summary = problem
        .encode()
        .expect("function-expr aspect should compile");

    assert_eq!(summary.constraint_bits.len(), 1);
    assert_eq!(summary.constraint_bits[0].model, "m");
    assert!(summary.constraint_bits[0].expected);
    let key = &summary.bit_mappings[summary.constraint_bits[0].bit].aspect;
    assert_eq!(key.display_fields(), "state=s0,depth=$2");
}
