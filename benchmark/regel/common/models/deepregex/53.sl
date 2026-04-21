(require "../../signatures/deepregex/53.sl")
(provide deepregex_53_P1 deepregex_53_P2 deepregex_53_P3 deepregex_53_P4 deepregex_53_P5 deepregex_53_P6 deepregex_53_P7 deepregex_53_N1 deepregex_53_N2 deepregex_53_N3 deepregex_53_N4 deepregex_53_N5 deepregex_53_N6 deepregex_53_N7)

(define-model deepregex_53_P1
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
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
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_P2
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
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
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($5) true)
    (($6) true)
    (($7) true)
    (($10) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($9) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($8) true)
    (($9) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($10) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($9) true)
    (($10) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    (($10) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_P3
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_P4
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model deepregex_53_P5
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
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($6) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
    (($2) true)
    (($4) true)
    (($5) true)
    (($6) true)
    (($7) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($1) true)
    (($5) true)
    (($8) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_P6
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_P7
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N1
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N2
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
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
    ((_) false)
  )))
  (interp-fun is.word (match (
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N3
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
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
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N4
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
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
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N5
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
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
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N6
  (define-univ Pos ($0 $1 $2))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
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
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)

(define-model deepregex_53_N7
  (define-univ Pos ())
  (interp-fun is.any (match (
    ((_) false)
  )))
  (interp-fun is.digit (match (
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
    ((_) false)
  )))
  (interp-fun is.word (match (
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x23 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3E (match (
    ((_) false)
  )))
  (interp-fun is.x7B (match (
    ((_) false)
  )))
  (interp-fun is.x7E (match (
    ((_) false)
  )))
)
