(provide U s t E P N)

(set-logic ALL)
(declare-sort U 0)
(declare-fun s () U)
(declare-fun t () U)
(declare-fun E (U U) Bool)

; Positive: there is a hidden path s -> u -> v -> t.
(define-model P
  (define-univ U ($0 $1 $2 $3))
  (interp-fun s ($0))
  (interp-fun t ($1))
  (interp-fun E (match (
    (($0 $2) true)
    (($2 $3) true)
    (($3 $1) true)
    ((_ _) false)
  )))
)

; Negative: the same FO1-visible unary facts hold, but the middle bridge is gone.
; FO1 with one variable can still see:
;   exists x. E(s,x)
;   exists x. E(x,t)
; but cannot relate those witnesses through the hidden anonymous edge.
(define-model N
  (define-univ U ($0 $1 $2 $3))
  (interp-fun s ($0))
  (interp-fun t ($1))
  (interp-fun E (match (
    (($0 $2) true)
    (($3 $1) true)
    ((_ _) false)
  )))
)
