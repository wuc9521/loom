(require "theories/chord-2f2f7007.sl")

(define-model P0
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
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N2
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N3
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N4
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N5
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N6
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N7
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model P8
  (define-univ chord.node (@chord.node_0 @chord.node_1))
  (interp-fun chord.tp1.n0 (@chord.node_1))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model P9
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_1))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_0 @chord.node_1 @chord.node_2) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_1 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N10
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N11
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_1 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N12
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_0 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N13
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_0 @chord.node_1 @chord.node_2) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_2) true)
    ((@chord.node_2 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N14
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_0))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_1) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N15
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_1) true)
    ((@chord.node_0 @chord.node_2 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_2) true)
    ((@chord.node_2 @chord.node_2 @chord.node_0) true)
    ((@chord.node_2 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N16
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N17
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_0 @chord.node_0) true)
    ((@chord.node_2 @chord.node_1 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(define-model N18
  (define-univ chord.node (@chord.node_0 @chord.node_1 @chord.node_2))
  (interp-fun chord.tp1.n0 (@chord.node_2))
  (interp-fun chord.bs.f (match (
    ((_ _) false)
  )))
  (interp-fun chord.bs.p (match (
    ((@chord.node_0 @chord.node_0 @chord.node_0) true)
    ((@chord.node_0 @chord.node_0 @chord.node_2) true)
    ((@chord.node_0 @chord.node_2 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1 @chord.node_1) true)
    ((@chord.node_1 @chord.node_1 @chord.node_2) true)
    ((@chord.node_1 @chord.node_2 @chord.node_1) true)
    ((@chord.node_2 @chord.node_2 @chord.node_2) true)
    ((_ _ _) false)
  )))
  (interp-fun chord.live (match (
    ((_) false)
  )))
  (interp-fun chord.prd (match (
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

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N2 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N3 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N4 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N5 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N6 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N7 @chord.node_0 @chord.node_0))))
(constraint (FO.Sem phi (FoAsp P8 @chord.node_0 @chord.node_0)))
(constraint (FO.Sem phi (FoAsp P9 @chord.node_0 @chord.node_0)))
(constraint (not (FO.Sem phi (FoAsp N10 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N11 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N12 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N13 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N14 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N15 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N16 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N17 @chord.node_0 @chord.node_0))))
(constraint (not (FO.Sem phi (FoAsp N18 @chord.node_0 @chord.node_0))))

(check-synth)
