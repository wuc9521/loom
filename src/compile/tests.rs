use super::*;

fn parse(input: &str) -> DelisProg {
    crate::parser::ast::parse(input, None).expect("compilation smoke program should parse")
}

#[test]
fn enumerates_subword_aspects_and_constraint_polarities() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len (Pos w1)))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len (Pos w5))))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 16);
    assert_eq!(summary.semantic_relations, vec!["Regex.Sem"]);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert_eq!(summary.constraint_bits[0].model, "w1");
    assert!(summary.constraint_bits[0].expected);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    assert_eq!(summary.constraint_bits[1].model, "w5");
    assert!(!summary.constraint_bits[1].expected);
    assert_eq!(summary.constraint_bits[1].bit, 9);
}

#[test]
fn finite_aspect_fields_are_not_named_primitives() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite Span
  (record (m Model) (left Int) (right Int))
  :where (and (<= 0 left) (<= left right) (<= right (len (Pos m)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp Span)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (Span w 0 (len (Pos w)))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

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
fn parses_direct_semantic_constraints() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (((_) true)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len (Pos w1)))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len (Pos w5))))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

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
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match ((($0) true) ((_) false))))
  (interp-fun b (match ((($1) true) ((_) false)))))
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun b (match (((_) false)))))
(declare-term-types ((Regex 0)) (((a) (b) (re.++ Regex Regex))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r
 ((a) (and (= (j asp) (+ (i asp) 1)) (a (i asp))))
 ((b) (and (= (j asp) (+ (i asp) 1)) (b (i asp))))
 ((re.++ x y)
  (exists ((k (int-range (i asp) (j asp))))
    (and (Regex.Sem x (ReAsp (model asp) (i asp) k))
         (Regex.Sem y (ReAsp (model asp) k (j asp)))))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len (Pos w1)))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len (Pos w5))))))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("projected semantic predicates should compile");

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
fn detects_two_way_navigation() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w1
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Root 0)) (((cfg.top.S Root Root))))
(define-finite CfgAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r ((cfg.top.S x y) (Cfg.Sem (parent r) asp)))))
(synth-fun r () Root)
(constraint (Cfg.Sem r (CfgAsp w1 0 (len (Pos w1)))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

    assert!(summary.needs_saturation);
}

#[test]
fn finite_aspects_and_predicates_use_declared_universe_sorts() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort A 0)
(declare-sort B 0)
(declare-fun P (A B) Bool)
(define-model m
  (define-univ B ($b0))
  (define-univ A ($a0 $a1))
  (interp-fun P (match ((($a0 $b0) true) ((_ _) false)))))
(declare-term-types ((Formula 0)) (((P.x.y))))
(define-finite FoAsp
  (record (model Model) (x A) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FoAsp)))
  ((match phi
     ((P.x.y) (P (x asp) (y asp))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FoAsp m $a0 $b0)))
(constraint (not (FO.Sem phi (FoAsp m $a1 $b0))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("many-sorted finite aspects should compile");

    assert_eq!(summary.bit_mappings.len(), 2);
    assert_eq!(summary.constraint_bits.len(), 2);
    assert!(summary.constraint_bits[0].expected);
    assert!(!summary.constraint_bits[1].expected);

    let sygus = to_sygus(&problem, false).expect("many-sorted predicates should lower to SyGuS");
    assert!(!sygus.is_empty());
}

#[test]
fn quantified_domains_can_select_a_non_primary_universe_sort() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort A 0)
(declare-sort B 0)
(declare-fun P (A B) Bool)
(define-model m
  (define-univ B ($b0))
  (define-univ A ($a0 $a1))
  (interp-fun P (match ((($a1 $b0) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0))
 (((fol.exists.a Formula)
   (P.x.y))))
(define-finite FoAsp
  (record (model Model) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FoAsp)))
  ((match phi
     ((fol.exists.a x)
      (exists ((v (domain (A (model asp)))))
        (FO.Sem x (FoAsp (model asp) (y asp)))))
     ((P.x.y)
      (exists ((v (domain (A (model asp)))))
        (P v (y asp)))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FoAsp m $b0)))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("typed quantified domains should lower to SyGuS");
    assert!(!sygus.is_empty());
}

#[test]
fn model_terms_support_constants_functions_and_nested_applications() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort A 0)
(declare-sort B 0)
(declare-fun c () A)
(declare-fun flag () Bool)
(declare-fun f (A) B)
(declare-fun g (B) A)
(declare-fun P (A) Bool)
(define-model m
  (define-univ A (@A.0 @A.1))
  (define-univ B (@B_0))
  (interp-fun c (@A.0))
  (interp-fun flag (true))
  (interp-fun f (match (((@A.0) @B_0) ((@A.1) @B_0))))
  (interp-fun g (match (((@B_0) @A.1))))
  (interp-fun P (match (((@A.1) true) ((_) false)))))
(declare-term-types
 ((Formula 0))
 (((flag.atom)
   (nested.pred)
   (nested.eq))))
(define-finite FoAsp
  (record (model Model)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FoAsp)))
  ((match phi
     ((flag.atom) (flag))
     ((nested.pred) (P (g (f c))))
     ((nested.eq) (= (g (f c)) @A.1)))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FoAsp m)))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("typed model terms should lower to SyGuS");

    assert!(!sygus.is_empty());
}

#[test]
fn model_terms_support_binary_functions_and_relations_as_bool_functions() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort A 0)
(declare-sort B 0)
(declare-fun h (A B) A)
(declare-fun R (A B) Bool)
(define-model m
  (define-univ A (@a0 @a1))
  (define-univ B (@b0))
  (interp-fun h (match (((@a0 @b0) @a1) ((@a1 @b0) @a0))))
  (interp-fun R (match (((@a1 @b0) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0))
 (((binary.term))))
(define-finite FoAsp
  (record (model Model) (x A) (y B)))
(define-semantics
  ((FO.Sem (phi Formula) (asp FoAsp)))
  ((match phi
     ((binary.term) (R (h (x asp) (y asp)) (y asp))))))
(synth-fun phi () Formula)
(constraint (FO.Sem phi (FoAsp m @a0 @b0)))
(constraint (not (FO.Sem phi (FoAsp m @a1 @b0))))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("binary typed model terms should compile");

    assert!(!sygus.is_empty());
}

#[test]
fn rejects_unknown_finite_field_sort_instead_of_singleton_symbol() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite BadAsp
  (record (model Model) (color Color)))
(define-semantics
  ((Regex.Sem (r Regex) (asp BadAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (BadAsp w red)))
(check-synth)
"#,
    );

    let err = compile(&problem).expect_err("unknown finite field sort should be rejected");

    assert!(err.contains("unsupported finite field sort Color"));
}

#[test]
fn rejects_multiple_finite_aspect_spaces() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(define-model w (define-univ Pos ($0)))
(declare-term-types ((Regex 0)) (((a))))
(define-finite FirstAsp (record (model Model)))
(define-finite SecondAsp (record (model Model)))
(define-semantics
  ((Regex.Sem (r Regex) (asp FirstAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(check-synth)
"#,
    );

    let err = compile(&problem).expect_err("multiple aspect spaces are not supported");

    assert!(err.contains("exactly one define-finite"));
}

#[test]
fn rejects_multiple_define_semantics_declarations() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(define-model w (define-univ Pos ($0)))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp (record (model Model)))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(define-semantics
  ((Regex.Other (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(check-synth)
"#,
    );

    let err = compile(&problem).expect_err("multiple define-semantics should be rejected");

    assert!(err.contains("exactly one define-semantics"));
}

#[test]
fn rejects_wrong_aspect_constructor_in_semantic_body() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r
     ((a) (Regex.Sem r (WrongAsp (model asp) (i asp) (j asp)))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w 0 (len (Pos w)))))
(check-synth)
"#,
    );

    let err = to_sygus(&problem, false).expect_err("wrong aspect constructor should be rejected");

    assert!(err.contains("aspect constructor must be ReAsp"));
}

#[test]
fn rejects_wrong_aspect_arity_in_semantic_body() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(declare-fun a (Pos) Bool)
(define-model w
  (define-univ Pos ($0))
  (interp-fun a (match (((_) true)))))
(declare-term-types ((Regex 0)) (((a))))
(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r
     ((a) (Regex.Sem r (ReAsp (model asp) (i asp)))))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w 0 (len (Pos w)))))
(check-synth)
"#,
    );

    let err = to_sygus(&problem, false).expect_err("wrong aspect arity should be rejected");

    assert!(err.contains("provides 1 scalar fields, expected 2"));
}
