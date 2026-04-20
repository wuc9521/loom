(require "../signatures/ticket_nested-ab5662a6.sl")
(provide P0 N1 N2 N3 P4 P5 N6 N7 P8 N9 N10 N11 P12 P13 N14)

(define-model P0
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
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

(define-model N1
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
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
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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

(define-model P5
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
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

(define-model N6
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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

(define-model N7
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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

(define-model P8
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
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

(define-model N9
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_1) true)
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

(define-model N10
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1 @ticket_protocol.thread_2))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_2))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
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

(define-model N11
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
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

(define-model P12
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
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

(define-model P13
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_1))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
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

(define-model N14
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1 @ticket_protocol.thread_2))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.tp1.sk1 (@ticket_protocol.thread_2))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.c (match (
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((_ _) false)
  )))
  (interp-fun ticket_protocol.m (match (
    ((@ticket_protocol.thread_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
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
