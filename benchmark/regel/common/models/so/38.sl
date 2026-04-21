(require "../../signatures/so/38.sl")
(provide so_38_P1 so_38_P2 so_38_N1 so_38_N2 so_38_N3 so_38_N4)

(define-model so_38_P1
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model so_38_P2
  (define-univ Pos ($0 $1))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
)

(define-model so_38_N1
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
)

(define-model so_38_N2
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model so_38_N3
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model so_38_N4
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x26 (match (
    (($3) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($0) true)
    ((_) false)
  )))
)
