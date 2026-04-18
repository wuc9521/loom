(require "theories/shared-a2d4218b.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ value (@value_0))
  (interp-fun dec (match (
    ((@node_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun done (match (
    ((_) false)
  )))
  (interp-fun input (match (
    ((@node_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((_) false)
  )))
  (interp-fun rcv_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun rcv_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun sent_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun sent_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun und_cons (match (
    ((_ _) false)
  )))
  (interp-fun und_cons_dec (match (
    ((@value_0) true)
    ((_) false)
  )))
)

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun dec (match (
    ((@node_0 @value_0) true)
    ((@node_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun done (match (
    ((_) false)
  )))
  (interp-fun input (match (
    ((@node_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((_) false)
  )))
  (interp-fun rcv_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun rcv_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun sent_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun sent_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun und_cons (match (
    ((_ _) false)
  )))
  (interp-fun und_cons_dec (match (
    ((@value_0) true)
    ((@value_1) true)
    ((_) false)
  )))
)

(define-model P2
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun dec (match (
    ((@node_0 @value_0) true)
    ((@node_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun done (match (
    ((_) false)
  )))
  (interp-fun input (match (
    ((@node_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((_) false)
  )))
  (interp-fun rcv_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun rcv_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun sent_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun sent_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun und_cons (match (
    ((_ _) false)
  )))
  (interp-fun und_cons_dec (match (
    ((_) false)
  )))
)

(define-model N3
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun dec (match (
    ((@node_0 @value_0) true)
    ((@node_0 @value_1) true)
    ((_ _) false)
  )))
  (interp-fun done (match (
    ((_) false)
  )))
  (interp-fun input (match (
    ((@node_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((_) false)
  )))
  (interp-fun rcv_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun rcv_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun sent_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun sent_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun und_cons (match (
    ((_ _) false)
  )))
  (interp-fun und_cons_dec (match (
    ((@value_0) true)
    ((_) false)
  )))
)

(define-model P4
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (define-univ quorum_c (@quorum_c_0))
  (define-univ value (@value_0 @value_1))
  (interp-fun dec (match (
    ((@node_0 @value_0) true)
    ((_ _) false)
  )))
  (interp-fun done (match (
    ((_) false)
  )))
  (interp-fun input (match (
    ((_ _) false)
  )))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_c (match (
    ((@node_0 @quorum_c_0) true)
    ((_ _) false)
  )))
  (interp-fun member_f (match (
    ((_) false)
  )))
  (interp-fun rcv_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun rcv_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun sent_msg (match (
    ((_ _ _) false)
  )))
  (interp-fun sent_msg_proj (match (
    ((_ _) false)
  )))
  (interp-fun und_cons (match (
    ((_ _) false)
  )))
  (interp-fun und_cons_dec (match (
    ((@value_0) true)
    ((_) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0 @quorum_c_0 @quorum_c_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0 @quorum_c_0 @quorum_c_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P2 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0 @quorum_c_0 @quorum_c_0 @value_0 @value_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0 @quorum_c_0 @quorum_c_0 @value_0 @value_0))))
(constraint (FO.Sem phi (FoAsp P4 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0 @quorum_c_0 @quorum_c_0 @value_0 @value_0)))

(check-synth)
