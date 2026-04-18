(require "theories/multi_paxos_epr-ed65a58f.sl")

(define-model P0
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_0))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N1
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_0))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_1)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((@round_1 @round_2) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model P2
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_1))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_1)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N3
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_2))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_1) true)
    ((@round_1 @round_2) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N4
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2 @round_3))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_2))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_3)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((@round_2 @round_0) true)
    ((@round_2 @round_1) true)
    ((@round_2 @round_2) true)
    ((@round_2 @round_3) true)
    ((@round_3 @round_0) true)
    ((@round_3 @round_1) true)
    ((@round_3 @round_3) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((@node_0 @round_3 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((@node_0 @inst_0 @round_3 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N5
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0 @votemap_1))
  (interp-fun none (@round_2))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_0)
    ((@votemap_1 @inst_0) @round_2)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((@votemap_1 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_1) true)
    ((@round_2 @round_1) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_0 @votemap_1) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_1 @votemap_1) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((@node_0 @round_2 @votemap_1) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N6
  (define-univ inst (@inst_0 @inst_1))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_2))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_0)
    ((@votemap_0 @inst_1) @round_2)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((@votemap_0 @inst_1) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_1) true)
    ((@round_2 @round_1) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((@node_0 @inst_1 @round_0 @value_0) true)
    ((@node_0 @inst_1 @round_1 @value_0) true)
    ((@node_0 @inst_1 @round_2 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model P7
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_1))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_1)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((@round_2 @round_0) true)
    ((@round_2 @round_1) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((@node_0 @round_2 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((@node_0 @inst_0 @round_2 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model P8
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_1))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b_msg (match (
    ((@node_0 @round_0 @votemap_0) true)
    ((@node_0 @round_1 @votemap_0) true)
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0))))
(constraint (FO.Sem phi (FoAsp P2 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N4 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N5 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N6 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0))))
(constraint (FO.Sem phi (FoAsp P7 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0)))
(constraint (FO.Sem phi (FoAsp P8 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @value_0 @value_0 @votemap_0 @votemap_0)))

(check-synth)
