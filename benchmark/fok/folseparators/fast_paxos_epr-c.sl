(require "theories/fast_paxos_epr-8ffc49d2.sl")

(define-model P0
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
    ((@node_0 @round_0 @value_0) true)
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
    ((_ _ _) false)
  )))
)

(define-model N1
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
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
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
    ((_ _ _) false)
  )))
)

(define-model P2
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

(define-model N3
  (define-univ c_quorum (@c_quorum_0))
  (define-univ f_quorum (@f_quorum_0))
  (define-univ node (@node_0 @node_1))
  (define-univ round (@round_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun none (@round_0))
  (interp-fun any_msg (match (
    ((_) false)
  )))
  (interp-fun c_member (match (
    ((@node_0 @c_quorum_0) true)
    ((@node_1 @c_quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
    ((_ _ _) false)
  )))
  (interp-fun f_member (match (
    ((@node_0 @f_quorum_0) true)
    ((@node_1 @f_quorum_0) true)
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
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @c_quorum_0 @c_quorum_0 @f_quorum_0 @f_quorum_0 @node_0 @node_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @c_quorum_0 @c_quorum_0 @f_quorum_0 @f_quorum_0 @node_0 @node_0 @round_0 @round_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P2 @c_quorum_0 @c_quorum_0 @f_quorum_0 @f_quorum_0 @node_0 @node_0 @round_0 @round_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @c_quorum_0 @c_quorum_0 @f_quorum_0 @f_quorum_0 @node_0 @node_0 @round_0 @round_0 @value_0 @value_0))))

(check-synth)
