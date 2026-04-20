(require "../signatures/lockserv-93e3d996.sl")
(provide P0 P1 N2 P3)

(define-model P0
  (define-univ node (@node_0))
  (interp-fun grant_msg (match (
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((_) false)
  )))
  (interp-fun lock_msg (match (
    ((_) false)
  )))
  (interp-fun server_holds_lock (false))
  (interp-fun unlock_msg (match (
    ((_) false)
  )))
)

(define-model P1
  (define-univ node (@node_0))
  (interp-fun grant_msg (match (
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((_) false)
  )))
  (interp-fun lock_msg (match (
    ((_) false)
  )))
  (interp-fun server_holds_lock (true))
  (interp-fun unlock_msg (match (
    ((_) false)
  )))
)

(define-model N2
  (define-univ node (@node_0 @node_1))
  (interp-fun grant_msg (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((_) false)
  )))
  (interp-fun lock_msg (match (
    ((_) false)
  )))
  (interp-fun server_holds_lock (true))
  (interp-fun unlock_msg (match (
    ((_) false)
  )))
)

(define-model P3
  (define-univ node (@node_0))
  (interp-fun grant_msg (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((_) false)
  )))
  (interp-fun lock_msg (match (
    ((_) false)
  )))
  (interp-fun server_holds_lock (false))
  (interp-fun unlock_msg (match (
    ((_) false)
  )))
)
