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
    assert_eq!(bv_sz(&problem).expect("BV length should compile"), 16);
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
fn enum_aspect_symbols_compare_as_finite_values() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort Pos 0)
(define-model w
  (define-univ Pos ($0)))
(declare-term-types ((Root 0)) (((leaf))))
(define-finite Asp
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

    let sygus = to_sygus(&problem, false).expect("enum aspect equality should compile");

    assert!(!sygus.is_empty());
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
fn builds_separate_bool_and_finite_value_semantic_domains() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort U 0)
(define-model m
  (define-univ U ($0 $1)))
(declare-term-types
 ((Formula 0) (TermU 0))
 (((pred TermU))
  ((var.x))))
(define-finite EnvAsp
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

    let summary = compile(&problem).expect("mixed semantic domains should compile");

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
fn semantic_bv_lowers_term_valued_semantics_to_separate_sorts() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort U 0)
(declare-fun R (U U) Bool)
(define-model m
  (define-univ U ($0 $1))
  (interp-fun R (match ((($0 $1) true) ((_ _) false)))))
(declare-term-types
 ((Formula 0) (TermU 0))
 (((atom TermU TermU)
   (exists.x Formula))
  ((x)
   (c))))
(define-finite EnvAsp
  (record (model Model) (xv U)))
(define-semantics
  ((TermU.Sem (t TermU) (asp EnvAsp)))
  :returns U
  ((match t
     ((x) (xv asp))
     ((c) $1))))
(define-semantics
  ((Formula.Sem (phi Formula) (asp EnvAsp)))
  ((match phi
     ((atom a b) (R (TermU.Sem a asp) (TermU.Sem b asp)))
     ((exists.x body)
      (exists ((v (domain (U (model asp)))))
        (Formula.Sem body (EnvAsp (model asp) v)))))))
(synth-fun phi () Formula)
(constraint (Formula.Sem phi (EnvAsp m $0)))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("mixed semantic BV should compile");
    let output = sygus
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(output.contains("(synth-fun phi () (_ BitVec 2)"));
    assert!(output.contains("(TermU (_ BitVec 4)"));
    assert!(!output.contains("(and ((_ extract"));
}

#[test]
fn union_finite_fields_enumerate_bot_plus_model_domains() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort U 0)
(define-model m
  (define-univ U ($0 $1)))
(declare-term-types
 ((Formula 0))
 (((top))))
(define-finite EnvAsp
  (record (model Model) (x (union bot U)) (y (union bot U))))
(define-semantics
  ((Formula.Sem (phi Formula) (asp EnvAsp)))
  ((match phi
     ((top) true))))
(synth-fun phi () Formula)
(constraint (Formula.Sem phi (EnvAsp m bot bot)))
(check-synth)
"#,
    );

    let summary = compile(&problem).expect("union finite fields should compile");

    assert_eq!(summary.bit_mappings.len(), 9);
    assert_eq!(summary.constraint_bits[0].bit, 0);
}

#[test]
fn semantic_bv_concatenates_multiple_domains_for_same_term_sort() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort U 0)
(define-model m
  (define-univ U ($0 $1)))
(declare-term-types
 ((Formula 0) (TermU 0))
 (((top))
  ((x))))
(define-finite EnvAsp
  (record (model Model) (xv (union bot U))))
(define-semantics
  ((TermU.Sem (t TermU) (asp EnvAsp)))
  :returns U
  ((match t
     ((x) (xv asp)))))
(define-semantics
  ((Formula.Sem (phi Formula) (asp EnvAsp)))
  ((match phi
     ((top) true))))
(define-semantics
  ((Formula.Valid (phi Formula) (asp EnvAsp)))
  ((match phi
     ((top) true))))
(synth-fun phi () Formula)
(constraint (Formula.Sem phi (EnvAsp m bot)))
(constraint (Formula.Valid phi (EnvAsp m bot)))
(check-synth)
"#,
    );

    let sygus = to_sygus(&problem, false).expect("multiple Formula domains should compile");
    let output = sygus
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(output.contains("(synth-fun phi () (_ BitVec 6)"));
    assert!(output.contains("(constraint (= ((_ extract 0 0) phi) #b1))"));
    assert!(output.contains("(constraint (= ((_ extract 3 3) phi) #b1))"));
}

#[test]
fn semantic_chc_emits_multiple_domains_for_same_term_sort() {
    let problem = parse(
        r#"
(set-logic ALL)
(declare-sort U 0)
(define-model m
  (define-univ U ($0)))
(declare-term-types
 ((Formula 0) (TermU 0))
 (((top))
  ((x))))
(define-finite EnvAsp
  (record (model Model) (xv (union bot U))))
(define-semantics
  ((TermU.Sem (t TermU) (asp EnvAsp)))
  :returns U
  ((match t
     ((x) (xv asp)))))
(define-semantics
  ((Formula.Sem (phi Formula) (asp EnvAsp)))
  ((match phi
     ((top) true))))
(define-semantics
  ((Formula.Valid (phi Formula) (asp EnvAsp)))
  ((match phi
     ((top) true))))
(synth-fun phi () Formula)
(constraint (Formula.Sem phi (EnvAsp m bot)))
(constraint (Formula.Valid phi (EnvAsp m bot)))
(check-synth)
"#,
    );

    let chc = to_chc_ast(&problem).expect("multiple Formula CHC domains should compile");
    let output = crate::backends::smt2::emit::to_fixedpoint_query_text(&chc, "Goal");

    assert!(output.contains("(rule (qFormula.Sem.m.xv.bot top))"));
    assert!(output.contains("(rule (qFormula.Valid.m.xv.bot top))"));
    assert!(output
        .contains("(=> (and (qFormula.Sem.m.xv.bot r) (qFormula.Valid.m.xv.bot r)) (Goal r))"));
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
fn allows_multiple_finite_aspect_spaces() {
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

    let summary = compile(&problem).expect("multiple aspect spaces should be summarized");

    assert_eq!(summary.finite_spaces, vec!["FirstAsp", "SecondAsp"]);
    assert_eq!(summary.semantic_domains.len(), 1);
    assert_eq!(summary.semantic_domains[0].aspect_space, "FirstAsp");
}

#[test]
fn allows_multiple_define_semantics_declarations() {
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

    let summary = compile(&problem).expect("multiple semantics should be summarized");

    assert_eq!(summary.semantic_relations, vec!["Regex.Sem", "Regex.Other"]);
    assert_eq!(summary.semantic_domains.len(), 2);
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
