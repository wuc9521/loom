(require "../../signatures/deepregex/47.sl")
(provide deepregex_47_P1 deepregex_47_P2 deepregex_47_P3 deepregex_47_P4 deepregex_47_P5 deepregex_47_P6 deepregex_47_P7 deepregex_47_N1 deepregex_47_N2 deepregex_47_N3 deepregex_47_N4 deepregex_47_N5 deepregex_47_N6 deepregex_47_N7)

(define-model deepregex_47_P1
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
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
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P2
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
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P3
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P4
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
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($1) true)
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
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P5
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
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.alnum (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.space (match (
    ((_) false)
  )))
  (interp-fun is.hex (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P6
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
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($2) true)
    (($4) true)
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
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_P7
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($2) true)
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
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N1
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
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N2
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
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N3
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
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model deepregex_47_N4
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N5
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
    (($1) true)
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
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N6
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
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($0) true)
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    ((_) false)
  )))
)

(define-model deepregex_47_N7
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
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
    (($2) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x24 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x2C (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x3C (match (
    ((_) false)
  )))
  (interp-fun is.x60 (match (
    (($0) true)
    ((_) false)
  )))
)
