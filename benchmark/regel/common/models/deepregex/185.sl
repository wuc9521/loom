(require "../../signatures/deepregex/185.sl")
(provide deepregex_185_P1 deepregex_185_P2 deepregex_185_P3 deepregex_185_P4 deepregex_185_P5 deepregex_185_P6 deepregex_185_P7 deepregex_185_N1 deepregex_185_N2 deepregex_185_N3 deepregex_185_N4 deepregex_185_N5 deepregex_185_N6 deepregex_185_N7)

(define-model deepregex_185_P1
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_P2
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model deepregex_185_P3
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_P4
  (define-univ Pos ($0 $1))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model deepregex_185_P5
  (define-univ Pos ($0 $1))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.word (match (
    (($1) true)
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
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_P6
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
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
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
    (($2) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model deepregex_185_P7
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model deepregex_185_N1
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N2
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.hex (match (
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N3
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N4
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N5
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N6
  (define-univ Pos ($0))
  (interp-fun is.any (match (
    (($0) true)
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
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)

(define-model deepregex_185_N7
  (define-univ Pos ($0 $1))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
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
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.x20 (match (
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
)
