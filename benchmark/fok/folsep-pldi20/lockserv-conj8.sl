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
  (define-univ node (@node_0))
  (interp-fun grant_msg (match (
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((@node_0) true)
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

(define-model N3
  (define-univ node (@node_0 @node_1))
  (interp-fun grant_msg (match (
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((@node_1) true)
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

(define-model P4
  (define-univ node (@node_0))
  (interp-fun grant_msg (match (
    ((_) false)
  )))
  (interp-fun holds_lock (match (
    ((@node_0) true)
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

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P1 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N3 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P4 fol.bot fol.bot)))

(check-synth)
