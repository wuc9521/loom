(provide U s t E P N)

(set-logic ALL)
(declare-sort U 0)
(declare-fun s () U)
(declare-fun t () U)
(declare-fun E (U U) Bool)

; P is exactly the 6-node chain s -> a -> b -> c -> d -> t.
(define-model P
  (define-univ U ($0 $1 $2 $3 $4 $5))
  (interp-fun s ($0))
  (interp-fun t ($1))
  (interp-fun E (match (
    (($0 $2) true)
    (($2 $3) true)
    (($3 $4) true)
    (($4 $5) true)
    (($5 $1) true)
    ((_ _) false)
  )))
)

; N is the broken 6-node graph s -> a -> b   c -> d -> t.
; This matches the shape the user proposed as
;   s -> _ -> _   _ -> _ -> t
(define-model N
  (define-univ U ($0 $1 $2 $3 $4 $5))
  (interp-fun s ($0))
  (interp-fun t ($1))
  (interp-fun E (match (
    (($0 $2) true)
    (($2 $3) true)
    (($4 $5) true)
    (($5 $1) true)
    ((_ _) false)
  )))
)
