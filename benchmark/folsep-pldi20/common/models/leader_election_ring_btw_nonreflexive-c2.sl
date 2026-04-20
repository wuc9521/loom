(require "../signatures/leader_election_ring_btw_nonreflexive-c4d3b61d.sl")
(provide P0 N1 P2 P3 N4 P5)

(define-model P0
  (define-univ id (@id_0))
  (define-univ node (@node_0))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ id (@id_0 @id_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((@node_1) @id_1)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((@id_1 @id_0) true)
    ((@id_1 @id_1) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((@node_0) true)
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)

(define-model P2
  (define-univ id (@id_0 @id_1))
  (define-univ node (@node_0))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((@id_0 @id_1) true)
    ((@id_1 @id_1) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)

(define-model P3
  (define-univ id (@id_0 @id_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((@node_1) @id_1)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((@id_0 @id_1) true)
    ((@id_1 @id_1) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)

(define-model N4
  (define-univ id (@id_0 @id_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((@node_1) @id_1)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((@id_1 @id_0) true)
    ((@id_1 @id_1) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((@node_1) true)
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)

(define-model P5
  (define-univ id (@id_0 @id_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun idn (match (
    ((@node_0) @id_0)
    ((@node_1) @id_1)
    ((_) @id_0)
  )))
  (interp-fun le (match (
    ((@id_0 @id_0) true)
    ((@id_1 @id_0) true)
    ((@id_1 @id_1) true)
    ((_ _) false)
  )))
  (interp-fun leader (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun pending (match (
    ((_ _) false)
  )))
  (interp-fun ring.btw (match (
    ((_ _ _) false)
  )))
)
