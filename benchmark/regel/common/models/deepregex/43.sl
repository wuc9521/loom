(require "../../signatures/deepregex/43.sl")
(provide deepregex_43_P1 deepregex_43_P2 deepregex_43_P3 deepregex_43_P4 deepregex_43_P5 deepregex_43_P6 deepregex_43_P7 deepregex_43_N1 deepregex_43_N2 deepregex_43_N3 deepregex_43_N4 deepregex_43_N5 deepregex_43_N6 deepregex_43_N7)

(define-model deepregex_43_P1
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P2
  (define-univ Pos ($0 $1))
  (interp-fun is.any (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.digit (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.lower (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
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
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P3
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P4
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P5
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P6
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_P7
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.upper (match (
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model deepregex_43_N1
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
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N2
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N3
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
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($1) true)
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
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N4
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
    ((_) false)
  )))
  (interp-fun is.upper (match (
    (($1) true)
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
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N5
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
    (($0) true)
    (($1) true)
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
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N6
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
    (($1) true)
    ((_) false)
  )))
  (interp-fun is.alpha (match (
    (($1) true)
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)

(define-model deepregex_43_N7
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
    ((_) false)
  )))
  (interp-fun is.x21 (match (
    ((_) false)
  )))
  (interp-fun is.x27 (match (
    ((_) false)
  )))
  (interp-fun is.x28 (match (
    ((_) false)
  )))
  (interp-fun is.x2A (match (
    ((_) false)
  )))
  (interp-fun is.x3B (match (
    ((_) false)
  )))
  (interp-fun is.x40 (match (
    ((_) false)
  )))
  (interp-fun is.x5B (match (
    ((_) false)
  )))
)
