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
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
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
    ((@epoch_0 @epoch_2) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_1 @epoch_2) true)
    ((@epoch_2 @epoch_2) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_1 @node_0) true)
    ((@epoch_2 @node_0) true)
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

(define-model P3
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
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
    ((@epoch_0 @epoch_2) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
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

(define-model N4
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_1))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_2) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_1 @epoch_2) true)
    ((@epoch_2 @epoch_2) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_2 @node_0) true)
    ((_ _) false)
  )))
)

(define-model N5
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_2))
  (interp-fun ep (match (
    ((@node_0) @epoch_2)
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
    ((@epoch_2 @epoch_0) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
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
    ((@epoch_2 @node_0) true)
    ((@epoch_2 @node_1) true)
    ((_ _) false)
  )))
)

(define-model N6
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_2))
  (interp-fun ep (match (
    ((@node_0) @epoch_2)
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
    ((@epoch_2 @epoch_0) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
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

(define-model N7
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_2))
  (interp-fun ep (match (
    ((@node_0) @epoch_2)
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
    ((@epoch_2 @epoch_0) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_1) true)
    ((@epoch_1 @node_0) true)
    ((_ _) false)
  )))
)

(define-model P8
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_0))
  (interp-fun ep (match (
    ((@node_0) @epoch_1)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_1) true)
    ((@epoch_0 @epoch_2) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_0) true)
    ((@epoch_1 @node_0) true)
    ((@epoch_2 @node_0) true)
    ((_ _) false)
  )))
)

(define-model N9
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2 @epoch_3))
  (define-univ node (@node_0 @node_1))
  (interp-fun zero (@epoch_2))
  (interp-fun ep (match (
    ((@node_0) @epoch_2)
    ((@node_1) @epoch_3)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_3) true)
    ((@epoch_1 @epoch_0) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_1 @epoch_3) true)
    ((@epoch_2 @epoch_0) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
    ((@epoch_2 @epoch_3) true)
    ((@epoch_3 @epoch_3) true)
    ((_ _) false)
  )))
  (interp-fun locked (match (
    ((_ _) false)
  )))
  (interp-fun transfer (match (
    ((@epoch_0 @node_1) true)
    ((@epoch_1 @node_0) true)
    ((_ _) false)
  )))
)

(define-model P10
  (define-univ epoch (@epoch_0 @epoch_1 @epoch_2))
  (define-univ node (@node_0))
  (interp-fun zero (@epoch_2))
  (interp-fun ep (match (
    ((@node_0) @epoch_2)
    ((_) @epoch_0)
  )))
  (interp-fun held (match (
    ((_) false)
  )))
  (interp-fun le (match (
    ((@epoch_0 @epoch_0) true)
    ((@epoch_0 @epoch_1) true)
    ((@epoch_1 @epoch_1) true)
    ((@epoch_2 @epoch_0) true)
    ((@epoch_2 @epoch_1) true)
    ((@epoch_2 @epoch_2) true)
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

(synth-fun phi () Formula)

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P2 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P3 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N4 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N6 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N7 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P8 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N9 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P10 fol.bot fol.bot)))

(check-synth)
