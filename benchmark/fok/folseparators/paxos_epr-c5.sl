(require "theories/paxos_epr-5637217b.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P3
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_1))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N4
  (define-univ node (@node_0 @node_1))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((@node_0 @round_0) true)
    ((@node_1 @round_0) true)
    ((@node_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((@node_1 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((@node_1 @round_0) true)
    ((@node_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P5
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N6
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_1))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P7
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P3 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N4 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P5 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N6 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P7 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))

(check-synth)
