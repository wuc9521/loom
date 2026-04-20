(require "../signatures/learning_switch-c3b0962f.sl")
(provide P0 N1 P2 N3 N4 N5 N6 N7 N8)

(define-model P0
  (define-univ node (@node_0))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_0) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model P2
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N4
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N5
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N6
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N7
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N8
  (define-univ node (@node_0 @node_1 @node_2))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_0 @node_1 @node_2) true)
    ((@node_0 @node_2 @node_0) true)
    ((@node_0 @node_2 @node_1) true)
    ((@node_0 @node_2 @node_2) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_0 @node_2) true)
    ((@node_1 @node_1 @node_1) true)
    ((@node_1 @node_2 @node_0) true)
    ((@node_1 @node_2 @node_1) true)
    ((@node_1 @node_2 @node_2) true)
    ((@node_2 @node_0 @node_0) true)
    ((@node_2 @node_0 @node_1) true)
    ((@node_2 @node_0 @node_2) true)
    ((@node_2 @node_1 @node_0) true)
    ((@node_2 @node_1 @node_1) true)
    ((@node_2 @node_1 @node_2) true)
    ((@node_2 @node_2 @node_2) true)
    ((_ _ _) false)
  )))
)
