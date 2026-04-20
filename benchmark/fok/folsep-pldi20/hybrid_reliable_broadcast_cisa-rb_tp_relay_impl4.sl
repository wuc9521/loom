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
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((@node_0) true)
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
    ((@node_0 @node_0) true)
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
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
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
    ((@node_0) true)
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

(define-model N4
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_1 @node_0) true)
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_1) true)
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P7
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
    ((@node_0) true)
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

(define-model P8
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
    ((@node_0 @node_0) true)
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
    ((_) false)
  )))
  (interp-fun rb.correct (match (
    ((@node_0) true)
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
    ((@node_1 @node_0) true)
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

(define-model P10
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
    ((@node_0) true)
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
    ((@node_0 @node_0) true)
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_1 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg (match (
    ((_ _) false)
  )))
  (interp-fun rb.sent_msg_proj (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun rb.symmetric (match (
    ((_) false)
  )))
)

(define-model P12
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
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

(define-model N13
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
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

(define-model N14
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
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

(define-model P15
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
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

(define-model N16
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
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

(define-model P17
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
    ((@node_0) true)
    ((@node_1) true)
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
    ((@node_0 @node_0) true)
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

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P3 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N4 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N6 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P7 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P8 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N9 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P10 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N11 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P12 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N13 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N14 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P15 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N16 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P17 fol.bot fol.bot)))

(check-synth)
