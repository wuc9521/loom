(require "../signatures/toy_lock_inv-4805fc37.sl")
(provide P0 N1 P2 N3 N4 N5)

(define-model P0
  (define-univ epoch (@epoch_0))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model N1
  (define-univ epoch (@epoch_0))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((@node_1) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_0 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model P2
  (define-univ epoch (@epoch_0))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((@node_1) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_0 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model N3
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((@node_1) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_1 @node_0) true)
    ((@epoch_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model N4
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((@node_1) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_0 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model N5
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((@node_1) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_1) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_0 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)
