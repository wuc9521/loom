(require "../signatures/fast_paxos_epr-8ffc49d2.sl")
(provide P0 N1 N2 N3 P4 N5 P6 P7 N8 P9)

(define-model P0
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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

(define-model P4
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@round_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model N5
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@round_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
    ((_ _ _) false)
  )))
)

(define-model P6
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((@round_0) true)
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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

(define-model P7
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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

(define-model N8
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
    ((@node_0 @round_1 @value_0) true)
    ((_ _ _) false)
  )))
)

(define-model P9
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((_ _) false)
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
