(require "theories/fast_paxos-46d74de9.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((@round_1) true)
    ((_) false)
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
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((@round_1) true)
    ((_) false)
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
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model P3
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N4
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N5
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((@round_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
    ((_ _ _) false)
  )))
)

(define-model P6
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model P7
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N8
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_1) true)
    ((_) false)
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
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model P9
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_f (@quorum_f_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun fast (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((@node_0 @quorum_f_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_) false)
  )))
  (interp-fun one_b (match (
    ((_ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun s (match (
    ((_) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P3 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N4 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (not (FO.Sem phi (FoAsp N5 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P6 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (FO.Sem phi (FoAsp P7 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N8 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P9 @node_0 @node_0 @quorum_c_0 @quorum_c_0 @quorum_f_0 @quorum_f_0 @round_0 @round_0 @value_0 @value_0)))

(check-synth)
