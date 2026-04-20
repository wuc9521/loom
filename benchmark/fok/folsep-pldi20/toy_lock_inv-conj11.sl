(require "theories/toy_lock_inv-4805fc37.sl")

(define-model P0
  (define-univ epoch (@epoch_0))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((_ _) false)
  )))
)

(define-model N1
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_1) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_1 @node_0) true)
    ((_ _) false)
  )))
)

(define-model P2
  (define-univ epoch (@epoch_0 @epoch_1))
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
    ((@epoch_0 @epoch_1) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_1 @node_0) true)
    ((_ _) false)
  )))
)

(define-model N3
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((@node_1) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_0 @node_1) true)
    ((@epoch_1 @node_0) true)
    ((@epoch_1 @node_1) true)
    ((_ _) false)
  )))
)

(define-model P4
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((_ _) false)
  )))
)

(define-model N5
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((@node_1) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_1) true)
    ((_ _) false)
  )))
)

(define-model P6
  (define-univ epoch (@epoch_0 @epoch_1))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_0)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((@node_0) true)
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_1 @node_0) true)
    ((_ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P2 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N3 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P4 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P6 fol.bot fol.bot)))

(check-synth)
