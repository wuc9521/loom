(require "theories/stoppable_paxos_epr-d7674849.sl")

(define-model P0
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
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
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N1
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
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
  (define-univ node (@node_0 @node_1))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_1 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
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
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N3
  (define-univ inst (@inst_0))
  (define-univ node (@node_0 @node_1))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_1 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
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

(define-model N4
  (define-univ inst (@inst_0 @inst_1))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((@uc_votemap_0 @inst_1) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((@uc_votemap_0 @inst_1) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_1 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((@inst_0 @inst_1) true)
    ((@inst_1 @inst_1) true)
    ((_ _) false)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_1 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N5
  (define-univ inst (@inst_0 @inst_1))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_0))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((@uc_votemap_0 @inst_1) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((@uc_votemap_0 @inst_1) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((@inst_1 @inst_0) true)
    ((@inst_1 @inst_1) true)
    ((_ _) false)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _ _) false)
  )))
)

(define-model N6
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_1))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
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
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @inst_0 @round_1 @value_0) true)
    ((_ _ _ _) false)
  )))
)

(define-model N7
  (define-univ inst (@inst_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (define-univ votemap (@uc_votemap_0))
  (interp-fun none (@round_1))
  (interp-fun stop (@value_0))
  (interp-fun maxr (match (
    ((@uc_votemap_0 @inst_0) @round_0)
    ((_ _) @round_0)
  )))
  (interp-fun maxv (match (
    ((@uc_votemap_0 @inst_0) @value_0)
    ((_ _) @value_0)
  )))
  (interp-fun active (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @inst_0 @round_0 @value_0) true)
    ((_ _ _ _) false)
  )))
  (interp-fun inst_le (match (
    ((@inst_0 @inst_0) true)
    ((_ _) false)
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

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N3 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N4 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N6 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N7 fol.bot fol.bot))))

(check-synth)
