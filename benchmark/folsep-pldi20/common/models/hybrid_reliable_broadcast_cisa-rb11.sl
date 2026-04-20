(require "../signatures/hybrid_reliable_broadcast_cisa-ad5dfe37.sl")
(provide P0 N1 N2 N3 N4 N5 P6 P7 P8 N9 P10 P11 N12 P13 N14 N15)

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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N2
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N3
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

(define-model N4
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
    ((_) false)
  )))
)

(define-model N5
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

(define-model P6
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P7
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P8
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
    ((@node_0) true)
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
    ((@node_0) true)
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
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P10
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P11
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
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P13
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
    ((@node_0) true)
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N14
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
    ((@node_0) true)
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
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model N15
  (define-univ node (@node_0 @node_1))
  (define-univ quorum_a (@quorum_a_0 @quorum_a_1))
  (define-univ quorum_b (@quorum_b_0))
  (interp-fun rb.tp_correctness.n0 (@node_0))
  (interp-fun rb.tp_relay.n1 (@node_0))
  (interp-fun rb.tp_relay.n2 (@node_0))
  (interp-fun member_a (match (
    ((@node_0 @quorum_a_1) true)
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
    ((@node_0) true)
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
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)
