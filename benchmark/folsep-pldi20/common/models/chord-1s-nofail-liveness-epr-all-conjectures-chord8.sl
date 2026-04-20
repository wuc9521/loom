(require "../signatures/chord-2f2f7007.sl")
(provide P0 N1 N2 P3 N4 N5 N6)

(define-model P0
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
    ((@chord.node_0) true)
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

(define-model N2
  (define-univ chord.node (@chord.node_0 @chord.node_1))
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
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((@chord.node_1 @chord.node_1) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((@chord.node_0) true)
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

(define-model P3
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

(define-model N4
  (define-univ chord.node (@chord.node_0 @chord.node_1))
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
    ((@chord.node_0 @chord.node_1) true)
    ((@chord.node_1 @chord.node_0) true)
    ((_ _) false)
  )))
  (interp-fun chord.prd_proj (match (
    ((@chord.node_0) true)
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

(define-model N5
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

(define-model N6
  (define-univ chord.node (@chord.node_0 @chord.node_1))
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
    ((@chord.node_1 @chord.node_0) true)
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
