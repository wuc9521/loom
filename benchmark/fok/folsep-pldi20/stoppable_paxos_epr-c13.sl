(require "theories/stoppable_paxos_epr-d7674849.sl")

(define-model P0
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N1
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N2
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model P3
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_1))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N4
  (define-univ inst (@inst_0))
  (define-univ node (@node_0 @node_1))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((@node_1 @round_0) true)
    ((@node_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
    ((@node_0 @round_0) true)
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N5
  (define-univ inst (@inst_0))
  (define-univ node (@node_0 @node_1))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((@node_1 @round_0) true)
    ((@node_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model P6
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N7
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_1))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_round (match (
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
  (interp-fun one_b_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N8
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (define-univ votemap (@votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@uc_value_0))
  (interp-fun maxr (match (
    ((@votemap_0 @inst_0) @round_1)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@votemap_0 @inst_0) @uc_value_0)
    ((_ _) @uc_value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
  )))
  (interp-fun joined_round (match (
    ((@node_0 @round_0) true)
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0))))
(constraint (FO.Sem phi (FoAsp P3 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0)))
(constraint (not (FO.Sem phi (FoAsp N4 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N5 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0))))
(constraint (FO.Sem phi (FoAsp P6 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0)))
(constraint (not (FO.Sem phi (FoAsp N7 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @uc_votemap_0 @uc_votemap_0))))
(constraint (not (FO.Sem phi (FoAsp N8 @inst_0 @inst_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0 @votemap_0 @votemap_0))))

(check-synth)
