(require "../../signatures/so/29.sl")
(provide so_29_P1 so_29_P2 so_29_P3 so_29_P4 so_29_P5 so_29_P6 so_29_P7 so_29_P8 so_29_N1 so_29_N2 so_29_N3 so_29_N4)

(define-model so_29_P1
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($2) true)
    (($4) true)
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
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_P2
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    (($12) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($12) true)
    (($13) true)
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
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($12) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($12) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($12) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($3) true)
    (($10) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    (($11) true)
    ((_) false)
  )))
)

(define-model so_29_P3
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
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
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_P4
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
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
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_P5
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
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
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model so_29_P6
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($2) true)
    (($4) true)
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
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_P7
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
  (interp-fun is.x2C (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model so_29_P8
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
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
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
    (($2) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_N1
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_N2
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
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
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    (($3) true)
    ((_) false)
  )))
)

(define-model so_29_N3
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($8) true)
    (($9) true)
    (($10) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($8) true)
    (($9) true)
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($3) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)

(define-model so_29_N4
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($5) true)
    (($6) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($5) true)
    (($6) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    (($5) true)
    (($7) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
)
