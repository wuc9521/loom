(require "../signatures/hybrid_reliable_broadcast_cisa-ad5dfe37.sl")
(provide P0 N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 P11 N12 N13 P14)

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
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
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
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N3
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_1))
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
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N4
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_1))
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
    ((@node_0 @node_1) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N5
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_1))
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
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N6
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_1))
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
    ((@node_0 @node_1) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N7
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_1))
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
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model N8
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_1))
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
    ((@node_0 @node_1) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model P11
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N12
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
    ((@node_0 @node_0) true)
    ((@node_1 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_0) true)
    ((_) false)
  )))
)

(define-model N13
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
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model P14
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
    ((@node_0) true)
    ((_) false)
  )))
)
