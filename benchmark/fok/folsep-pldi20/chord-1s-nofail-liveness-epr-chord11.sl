(require "theories/chord-2f2f7007.sl")

(define-model P0
  (define-univ chord.node (@chord.node_0))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((@chord.node_0 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model N1
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((@chord.node_0 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_1 @chord.node_2) true)
    ((@chord.node_1 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P2
  (define-univ chord.node (@chord.node_0))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model N3
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_0 @chord.node_1 @chord.node_2) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P4
  (define-univ chord.node (@chord.node_0))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P5
  (define-univ chord.node (@chord.node_0 @chord.node_1))
  (interp-fun chord.tp1.n0 (@chord.node_1))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P6
  (define-univ chord.node (@chord.node_0 @chord.node_1))
  (interp-fun chord.tp1.n0 (@chord.node_1))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P7
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_1))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model N8
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model N9
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P10
  (define-univ chord.node (@chord.node_0 @chord.node_1))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model N11
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(define-model P12
  (define-univ chord.node (@chord.node_0 @chord.node_1))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
    ((@chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((_) false)
  )))
  (interp-fun chord.ring.btw (match (
    ((_ _ _) false)
  )))
  (interp-fun chord.scheduled_rectify (match (
    ((_ _) false)
  )))
  (interp-fun chord.scheduled_stabilize (match (
    ((_ _) false)
  )))
  (interp-fun chord.stable (false))
  (interp-fun chord.started (false))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @chord.node_0 @chord.node_0))))
(constraint (FO.Sem phi (FoAsp P2 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @chord.node_0 @chord.node_0))))
(constraint (FO.Sem phi (FoAsp P4 @chord.node_0 @chord.node_0)))
(constraint (FO.Sem phi (FoAsp P5 @chord.node_0 @chord.node_0)))
(constraint (FO.Sem phi (FoAsp P6 @chord.node_0 @chord.node_0)))
(constraint (FO.Sem phi (FoAsp P7 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N8 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N9 @chord.node_0 @chord.node_0))))
(constraint (FO.Sem phi (FoAsp P10 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N11 @chord.node_0 @chord.node_0))))
(constraint (FO.Sem phi (FoAsp P12 @chord.node_0 @chord.node_0)))

(check-synth)
