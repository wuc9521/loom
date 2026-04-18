(require "theories/lockserv-93e3d996.sl")

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
    ((@node_0) true)
    ((_) false)
  )))
)

(define-model N1
  (define-univ node (@node_0 @node_1))
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
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
)

(define-model P2
  (define-univ node (@node_0 @node_1))
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

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @node_0 @node_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @node_0 @node_0))))
(constraint (FO.Sem phi (FoAsp P2 @node_0 @node_0)))

(check-synth)
