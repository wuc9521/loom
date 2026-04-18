(require "../../theories/ltl-2.sl")

(define-model P0
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model P1
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
)

(define-model P2
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P3
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P4
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model N0
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    ((_) false)
  )))
)

(define-model N1
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N2
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N3
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N4
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(synth-fun phi () LTL)

(constraint (Ltl.Sem phi (LtlAsp P0 0)))
(constraint (Ltl.Sem phi (LtlAsp P1 0)))
(constraint (Ltl.Sem phi (LtlAsp P2 0)))
(constraint (Ltl.Sem phi (LtlAsp P3 0)))
(constraint (Ltl.Sem phi (LtlAsp P4 0)))
(constraint (not (Ltl.Sem phi (LtlAsp N0 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N1 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N2 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N3 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N4 0))))

(check-synth)
