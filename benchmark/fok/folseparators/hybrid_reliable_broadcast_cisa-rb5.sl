(require "theories/hybrid_reliable_broadcast_cisa-ad5dfe37.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N1
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N2
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P3
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P4
  (define-univ node (@node_0))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_0 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P5
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N6
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N7
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N8
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N9
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N10
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N11
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_1 @quorum_a_0) true)
    ((_ _) false)
  )))
  (interp-fun member_b (match (
    ((@node_1 @quorum_b_0) true)
    ((_ _) false)
  )))
  (interp-fun member_fa (match (
    ((_) false)
  )))
  (interp-fun member_fc (match (
    ((_) false)
  )))
  (interp-fun member_fi (match (
    ((_) false)
  )))
  (interp-fun member_fs (match (
    ((_) false)
  )))
  (interp-fun rb.accept (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((_) false)
  )))
  (interp-fun rb.d (match (
    ((_) false)
  )))
  (interp-fun rb.obedient (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.rcv_init (match (
    ((_) false)
  )))
  (interp-fun rb.rcv_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (FO.Sem phi (FoAsp P3 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0)))
(constraint (FO.Sem phi (FoAsp P4 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0)))
(constraint (FO.Sem phi (FoAsp P5 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0)))
(constraint (not (FO.Sem phi (FoAsp N6 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N7 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N8 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N9 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N10 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))
(constraint (not (FO.Sem phi (FoAsp N11 @node_0 @node_0 @quorum_a_0 @quorum_a_0 @quorum_b_0 @quorum_b_0))))

(check-synth)
