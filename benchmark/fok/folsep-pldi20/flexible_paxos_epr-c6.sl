(require "theories/flexible_paxos_epr-12d2d6cd.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_1 (@quorum_1_0))
  (define-univ quorum_2 (@quorum_2_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
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
  (interp-fun member_1 (match (
    ((@node_0 @quorum_1_0) true)
    ((_ _) false)
  )))
  (interp-fun member_2 (match (
    ((@node_0 @quorum_2_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
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
  (define-univ quorum_1 (@quorum_1_0))
  (define-univ quorum_2 (@quorum_2_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
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
  (interp-fun member_1 (match (
    ((@node_0 @quorum_1_0) true)
    ((_ _) false)
  )))
  (interp-fun member_2 (match (
    ((@node_0 @quorum_2_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_1 (@quorum_1_0))
  (define-univ quorum_2 (@quorum_2_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
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
  (interp-fun member_1 (match (
    ((@node_0 @quorum_1_0) true)
    ((@node_1 @quorum_1_0) true)
    ((_ _) false)
  )))
  (interp-fun member_2 (match (
    ((@node_0 @quorum_2_0) true)
    ((@node_1 @quorum_2_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_1 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ node (@node_0))
  (define-univ quorum_1 (@quorum_1_0))
  (define-univ quorum_2 (@quorum_2_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0 @value_1))
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
  (interp-fun member_1 (match (
    ((@node_0 @quorum_1_0) true)
    ((_ _) false)
  )))
  (interp-fun member_2 (match (
    ((@node_0 @quorum_2_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_1_0 @quorum_1_0 @quorum_2_0 @quorum_2_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_1_0 @quorum_1_0 @quorum_2_0 @quorum_2_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @node_0 @node_0 @quorum_1_0 @quorum_1_0 @quorum_2_0 @quorum_2_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (not (FO.Sem phi (FoAsp N3 @node_0 @node_0 @quorum_1_0 @quorum_1_0 @quorum_2_0 @quorum_2_0 @round_0 @round_0 @value_0 @value_0))))

(check-synth)
