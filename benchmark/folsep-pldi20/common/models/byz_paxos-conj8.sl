(require "../signatures/byz_paxos-0f4e99fa.sl")
(provide P0 N1 P2 N3)

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((@node_0 @round_0 @value_0) true)
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

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
  (define-univ round (@round_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun none (@round_0))
  (interp-fun decision (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
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

(define-model P2
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
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

(define-model N3
  (define-univ node (@node_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ quorum_w (@quorum_w_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@value_0 @value_1))
  (interp-fun none (@round_1))
  (interp-fun decision (match (
    ((@node_0 @round_0 @value_0) true)
    ((@node_0 @round_0 @value_1) true)
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
