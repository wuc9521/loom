(require "../signatures/ticket-4357690c.sl")
(provide P0 N1 N2 N3 P4 N5 N6 P7 N8 N9 P10)

(define-model P0
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N1
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N2
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N3
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model P4
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N5
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N6
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_1) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model P7
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N8
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((@ticket_protocol.thread_1) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model N9
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)

(define-model P10
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1 @ticket_protocol.ticket_2))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_2))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_2) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_2) true)
    ((@ticket_protocol.ticket_2 @ticket_protocol.ticket_2) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.pc1 (match (
    ((@ticket_protocol.thread_0) true)
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc2 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.pc3 (match (
    ((_) false)
  )))
  (interp-fun ticket_protocol.scheduled (match (
    ((_) false)
  )))
)
