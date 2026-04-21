(require "../../signatures/so/52.sl")
(provide so_52_P1 so_52_P2 so_52_P3 so_52_P4 so_52_N1 so_52_N2 so_52_N3 so_52_N4)

(define-model so_52_P1
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)

(define-model so_52_P2
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
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)

(define-model so_52_P3
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
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    (($5) true)
    ((_) false)
  )))
)

(define-model so_52_P4
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13 $14 $15 $16 $17 $18 $19 $20))
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
    (($14) true)
    (($15) true)
    (($16) true)
    (($17) true)
    (($18) true)
    (($19) true)
    (($20) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($2) true)
    (($5) true)
    (($7) true)
    (($10) true)
    (($12) true)
    (($15) true)
    (($16) true)
    (($17) true)
    (($18) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
    (($5) true)
    (($7) true)
    (($10) true)
    (($12) true)
    (($15) true)
    (($16) true)
    (($17) true)
    (($18) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($2) true)
    (($5) true)
    (($7) true)
    (($8) true)
    (($10) true)
    (($12) true)
    (($15) true)
    (($16) true)
    (($17) true)
    (($18) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($5) true)
    (($7) true)
    (($8) true)
    (($10) true)
    (($12) true)
    (($15) true)
    (($16) true)
    (($17) true)
    (($18) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    (($4) true)
    (($9) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($2) true)
    (($8) true)
    (($18) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    (($4) true)
    (($9) true)
    (($13) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    (($19) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    (($20) true)
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    (($11) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    (($14) true)
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    (($3) true)
    ((_) false)
  )))
)

(define-model so_52_N1
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
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)

(define-model so_52_N2
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
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)

(define-model so_52_N3
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
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)

(define-model so_52_N4
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
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
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($1) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    (($5) true)
    ((_) false)
  )))
  (interp-fun is.x25 (match (
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x29 (match (
    ((_) false)
  )))
  (interp-fun is.x2B (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x2D (match (
    ((_) false)
  )))
  (interp-fun is.x2E (match (
    ((_) false)
  )))
  (interp-fun is.x2F (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x5C (match (
    ((_) false)
  )))
  (interp-fun is.x5F (match (
    ((_) false)
  )))
)
