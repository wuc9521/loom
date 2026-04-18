(require "theories/byz_paxos-0f4e99fa.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((@node_0 @round_0 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((@node_0 @round_0 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((@node_1 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((@node_1 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((@node_0 @round_0 @node_0) true)
    ((@node_0 @round_0 @node_1) true)
    ((@node_1 @round_0 @node_0) true)
    ((@node_1 @round_0 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((@node_0 @round_0 @node_0) true)
    ((@node_0 @round_1 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P4
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N5
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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
  (interp-fun member_b (match (
    ((_) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((@node_1 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_w (match (
    ((@node_0 @quorum_w_0) true)
    ((@node_1 @quorum_w_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((@node_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun prepare (match (
    ((_ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun rcved_one_b (match (
    ((@node_0 @round_0 @node_0) true)
    ((@node_0 @round_0 @node_1) true)
    ((@node_1 @round_0 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (not (FO.Sem phi (FoAsp N3 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P4 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N5 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_w_0 @quorum_w_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))

(check-synth)
