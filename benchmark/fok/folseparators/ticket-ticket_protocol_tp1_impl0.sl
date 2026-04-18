(require "theories/ticket-4357690c.sl")

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
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
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

(define-model P2
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
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

(define-model N4
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1 @ticket_protocol.ticket_2))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.service (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.tp1.sk0 (@ticket_protocol.thread_0))
  (interp-fun ticket_protocol.zero (@ticket_protocol.ticket_0))
  (interp-fun ticket_protocol.le (match (
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_0 @ticket_protocol.ticket_2) true)
    ((@ticket_protocol.ticket_1 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_2 @ticket_protocol.ticket_1) true)
    ((@ticket_protocol.ticket_2 @ticket_protocol.ticket_2) true)
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

(define-model N5
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

(define-model P6
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

(define-model P7
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_0))
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

(define-model P8
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0))
  (define-univ ticket_protocol.ticket (@ticket_protocol.ticket_0 @ticket_protocol.ticket_1))
  (interp-fun ticket_protocol.next_ticket (@ticket_protocol.ticket_1))
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

(define-model P9
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

(define-model P10
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

(define-model P11
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

(define-model P12
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
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
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_0) true)
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_1) true)
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

(define-model N13
  (define-univ ticket_protocol.thread (@ticket_protocol.thread_0 @ticket_protocol.thread_1))
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
    ((@ticket_protocol.thread_1 @ticket_protocol.ticket_1) true)
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

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0))))
(constraint (FO.Sem phi (FoAsp P2 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0))))
(constraint (not (FO.Sem phi (FoAsp N4 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0))))
(constraint (not (FO.Sem phi (FoAsp N5 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0))))
(constraint (FO.Sem phi (FoAsp P6 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P7 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P8 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P9 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P10 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P11 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (FO.Sem phi (FoAsp P12 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0)))
(constraint (not (FO.Sem phi (FoAsp N13 @ticket_protocol.thread_0 @ticket_protocol.thread_0 @ticket_protocol.ticket_0 @ticket_protocol.ticket_0))))

(check-synth)
