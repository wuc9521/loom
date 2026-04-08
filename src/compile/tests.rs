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
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
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
  :where (and (<= 0 left) (<= left right) (<= right (len m))))
(define-semantics
  ((Regex.Sem (r Regex) (asp Span)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (Span w 0 (len w))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

    assert_eq!(summary.bit_mappings.len(), 6);
    assert_eq!(summary.constraint_bits.len(), 1);
    assert_eq!(summary.constraint_bits[0].bit, 2);
    assert_eq!(
        summary.bit_mappings[2].aspect.fields,
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
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Regex.Sem (r Regex) (asp ReAsp)))
  ((match r ((a) true))))
(synth-fun r () Regex)
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
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
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
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
(constraint (Regex.Sem r (ReAsp w1 0 (len w1))))
(constraint (not (Regex.Sem r (ReAsp w5 0 (len w5)))))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem).expect("projected semantic predicates should compile");

    assert!(!sygus.is_empty());
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
  :where (and (<= 0 i) (<= i j) (<= j (len model))))
(define-semantics
  ((Cfg.Sem (r Root) (asp CfgAsp)))
  ((match r ((cfg.top.S x y) (Cfg.Sem (parent r) asp)))))
(synth-fun r () Root)
(constraint (Cfg.Sem r (CfgAsp w1 0 (len w1))))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("compilation should summarize");

    assert!(summary.needs_saturation);
}
