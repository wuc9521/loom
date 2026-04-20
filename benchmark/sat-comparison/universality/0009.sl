(require "../theories/ltl-2.sl")

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
    (($2) true)
    (($3) true)
    (($4) true)
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
    (($0) true)
    (($3) true)
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
    (($1) true)
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P5
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P6
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
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P7
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P8
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P9
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P10
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
    ((_) false)
  )))
)

(define-model P11
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P12
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P13
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

(define-model P14
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P15
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P16
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P17
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

(define-model P18
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P19
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P20
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

(define-model P21
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P22
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
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model P23
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
    ((_) false)
  )))
)

(define-model P24
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
    ((_) false)
  )))
)

(define-model P25
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

(define-model P26
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P27
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P28
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

(define-model P29
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
    ((_) false)
  )))
)

(define-model P30
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P31
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P32
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
    ((_) false)
  )))
)

(define-model P33
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P34
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

(define-model P35
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
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P36
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
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model P37
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P38
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

(define-model P39
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P40
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P41
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
    ((_) false)
  )))
)

(define-model P42
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P43
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P44
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
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model P45
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
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P46
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

(define-model P47
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P48
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

(define-model P49
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P50
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
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P51
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

(define-model P52
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

(define-model P53
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
    ((_) false)
  )))
)

(define-model P54
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P55
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P56
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

(define-model P57
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
    ((_) false)
  )))
)

(define-model P58
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P59
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
    ((_) false)
  )))
)

(define-model P60
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P61
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
    ((_) false)
  )))
)

(define-model P62
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P63
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
    ((_) false)
  )))
)

(define-model P64
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
    ((_) false)
  )))
)

(define-model P65
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P66
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P67
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P68
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
    ((_) false)
  )))
)

(define-model P69
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P70
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P71
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
    ((_) false)
  )))
)

(define-model P72
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P73
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P74
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P75
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P76
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
    ((_) false)
  )))
)

(define-model P77
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P78
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P79
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

(define-model P80
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P81
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P82
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
    ((_) false)
  )))
)

(define-model P83
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P84
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
    ((_) false)
  )))
)

(define-model P85
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P86
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
    (($3) true)
    ((_) false)
  )))
)

(define-model P87
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P88
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

(define-model P89
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

(define-model P90
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
    (($2) true)
    ((_) false)
  )))
)

(define-model P91
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P92
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

(define-model P93
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

(define-model P94
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
    (($4) true)
    ((_) false)
  )))
)

(define-model P95
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

(define-model P96
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
    ((_) false)
  )))
)

(define-model P97
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
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P98
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model P99
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
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N0
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N1
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N2
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N3
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
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

(define-model N4
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N5
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
)

(define-model N6
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N7
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N8
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
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

(define-model N9
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N10
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N11
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N12
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N13
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    ((_) false)
  )))
)

(define-model N14
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N15
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N16
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($3) true)
    ((_) false)
  )))
)

(define-model N17
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N18
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N19
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model N20
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N21
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($4) true)
    ((_) false)
  )))
)

(define-model N22
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model N23
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N24
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N25
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N26
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N27
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N28
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N29
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N30
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
)

(define-model N31
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N32
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N33
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N34
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model N35
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N36
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
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

(define-model N37
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N38
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N39
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N40
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

(define-model N41
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N42
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N43
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N44
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
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

(define-model N45
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N46
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N47
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    ((_) false)
  )))
)

(define-model N48
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
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

(define-model N49
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N50
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N51
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model N52
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($4) true)
    ((_) false)
  )))
)

(define-model N53
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N54
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model N55
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    ((_) false)
  )))
)

(define-model N56
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N57
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model N58
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
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

(define-model N59
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N60
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($4) true)
    ((_) false)
  )))
)

(define-model N61
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N62
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N63
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N64
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N65
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
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

(define-model N66
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N67
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model N68
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($3) true)
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

(define-model N69
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N70
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N71
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($1) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N72
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N73
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
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

(define-model N74
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N75
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
)

(define-model N76
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N77
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N78
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model N79
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
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

(define-model N80
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N81
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N82
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N83
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N84
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N85
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N86
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model N87
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N88
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N89
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N90
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    ((_) false)
  )))
)

(define-model N91
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    ((_) false)
  )))
)

(define-model N92
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N93
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    (($2) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(define-model N94
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    (($4) true)
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

(define-model N95
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($1) true)
    (($2) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N96
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($2) true)
    (($3) true)
    ((_) false)
  )))
)

(define-model N97
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($0) true)
    ((_) false)
  )))
)

(define-model N98
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($1) true)
    (($2) true)
    (($4) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($2) true)
    ((_) false)
  )))
)

(define-model N99
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun p0 (match (
    (($0) true)
    (($3) true)
    ((_) false)
  )))
  (interp-fun p1 (match (
    (($1) true)
    (($3) true)
    (($4) true)
    ((_) false)
  )))
)

(synth-fun phi () LTL)

(constraint (Ltl.Sem phi (LtlAsp P0 0)))
(constraint (Ltl.Sem phi (LtlAsp P1 0)))
(constraint (Ltl.Sem phi (LtlAsp P2 0)))
(constraint (Ltl.Sem phi (LtlAsp P3 0)))
(constraint (Ltl.Sem phi (LtlAsp P4 0)))
(constraint (Ltl.Sem phi (LtlAsp P5 0)))
(constraint (Ltl.Sem phi (LtlAsp P6 0)))
(constraint (Ltl.Sem phi (LtlAsp P7 0)))
(constraint (Ltl.Sem phi (LtlAsp P8 0)))
(constraint (Ltl.Sem phi (LtlAsp P9 0)))
(constraint (Ltl.Sem phi (LtlAsp P10 0)))
(constraint (Ltl.Sem phi (LtlAsp P11 0)))
(constraint (Ltl.Sem phi (LtlAsp P12 0)))
(constraint (Ltl.Sem phi (LtlAsp P13 0)))
(constraint (Ltl.Sem phi (LtlAsp P14 0)))
(constraint (Ltl.Sem phi (LtlAsp P15 0)))
(constraint (Ltl.Sem phi (LtlAsp P16 0)))
(constraint (Ltl.Sem phi (LtlAsp P17 0)))
(constraint (Ltl.Sem phi (LtlAsp P18 0)))
(constraint (Ltl.Sem phi (LtlAsp P19 0)))
(constraint (Ltl.Sem phi (LtlAsp P20 0)))
(constraint (Ltl.Sem phi (LtlAsp P21 0)))
(constraint (Ltl.Sem phi (LtlAsp P22 0)))
(constraint (Ltl.Sem phi (LtlAsp P23 0)))
(constraint (Ltl.Sem phi (LtlAsp P24 0)))
(constraint (Ltl.Sem phi (LtlAsp P25 0)))
(constraint (Ltl.Sem phi (LtlAsp P26 0)))
(constraint (Ltl.Sem phi (LtlAsp P27 0)))
(constraint (Ltl.Sem phi (LtlAsp P28 0)))
(constraint (Ltl.Sem phi (LtlAsp P29 0)))
(constraint (Ltl.Sem phi (LtlAsp P30 0)))
(constraint (Ltl.Sem phi (LtlAsp P31 0)))
(constraint (Ltl.Sem phi (LtlAsp P32 0)))
(constraint (Ltl.Sem phi (LtlAsp P33 0)))
(constraint (Ltl.Sem phi (LtlAsp P34 0)))
(constraint (Ltl.Sem phi (LtlAsp P35 0)))
(constraint (Ltl.Sem phi (LtlAsp P36 0)))
(constraint (Ltl.Sem phi (LtlAsp P37 0)))
(constraint (Ltl.Sem phi (LtlAsp P38 0)))
(constraint (Ltl.Sem phi (LtlAsp P39 0)))
(constraint (Ltl.Sem phi (LtlAsp P40 0)))
(constraint (Ltl.Sem phi (LtlAsp P41 0)))
(constraint (Ltl.Sem phi (LtlAsp P42 0)))
(constraint (Ltl.Sem phi (LtlAsp P43 0)))
(constraint (Ltl.Sem phi (LtlAsp P44 0)))
(constraint (Ltl.Sem phi (LtlAsp P45 0)))
(constraint (Ltl.Sem phi (LtlAsp P46 0)))
(constraint (Ltl.Sem phi (LtlAsp P47 0)))
(constraint (Ltl.Sem phi (LtlAsp P48 0)))
(constraint (Ltl.Sem phi (LtlAsp P49 0)))
(constraint (Ltl.Sem phi (LtlAsp P50 0)))
(constraint (Ltl.Sem phi (LtlAsp P51 0)))
(constraint (Ltl.Sem phi (LtlAsp P52 0)))
(constraint (Ltl.Sem phi (LtlAsp P53 0)))
(constraint (Ltl.Sem phi (LtlAsp P54 0)))
(constraint (Ltl.Sem phi (LtlAsp P55 0)))
(constraint (Ltl.Sem phi (LtlAsp P56 0)))
(constraint (Ltl.Sem phi (LtlAsp P57 0)))
(constraint (Ltl.Sem phi (LtlAsp P58 0)))
(constraint (Ltl.Sem phi (LtlAsp P59 0)))
(constraint (Ltl.Sem phi (LtlAsp P60 0)))
(constraint (Ltl.Sem phi (LtlAsp P61 0)))
(constraint (Ltl.Sem phi (LtlAsp P62 0)))
(constraint (Ltl.Sem phi (LtlAsp P63 0)))
(constraint (Ltl.Sem phi (LtlAsp P64 0)))
(constraint (Ltl.Sem phi (LtlAsp P65 0)))
(constraint (Ltl.Sem phi (LtlAsp P66 0)))
(constraint (Ltl.Sem phi (LtlAsp P67 0)))
(constraint (Ltl.Sem phi (LtlAsp P68 0)))
(constraint (Ltl.Sem phi (LtlAsp P69 0)))
(constraint (Ltl.Sem phi (LtlAsp P70 0)))
(constraint (Ltl.Sem phi (LtlAsp P71 0)))
(constraint (Ltl.Sem phi (LtlAsp P72 0)))
(constraint (Ltl.Sem phi (LtlAsp P73 0)))
(constraint (Ltl.Sem phi (LtlAsp P74 0)))
(constraint (Ltl.Sem phi (LtlAsp P75 0)))
(constraint (Ltl.Sem phi (LtlAsp P76 0)))
(constraint (Ltl.Sem phi (LtlAsp P77 0)))
(constraint (Ltl.Sem phi (LtlAsp P78 0)))
(constraint (Ltl.Sem phi (LtlAsp P79 0)))
(constraint (Ltl.Sem phi (LtlAsp P80 0)))
(constraint (Ltl.Sem phi (LtlAsp P81 0)))
(constraint (Ltl.Sem phi (LtlAsp P82 0)))
(constraint (Ltl.Sem phi (LtlAsp P83 0)))
(constraint (Ltl.Sem phi (LtlAsp P84 0)))
(constraint (Ltl.Sem phi (LtlAsp P85 0)))
(constraint (Ltl.Sem phi (LtlAsp P86 0)))
(constraint (Ltl.Sem phi (LtlAsp P87 0)))
(constraint (Ltl.Sem phi (LtlAsp P88 0)))
(constraint (Ltl.Sem phi (LtlAsp P89 0)))
(constraint (Ltl.Sem phi (LtlAsp P90 0)))
(constraint (Ltl.Sem phi (LtlAsp P91 0)))
(constraint (Ltl.Sem phi (LtlAsp P92 0)))
(constraint (Ltl.Sem phi (LtlAsp P93 0)))
(constraint (Ltl.Sem phi (LtlAsp P94 0)))
(constraint (Ltl.Sem phi (LtlAsp P95 0)))
(constraint (Ltl.Sem phi (LtlAsp P96 0)))
(constraint (Ltl.Sem phi (LtlAsp P97 0)))
(constraint (Ltl.Sem phi (LtlAsp P98 0)))
(constraint (Ltl.Sem phi (LtlAsp P99 0)))
(constraint (not (Ltl.Sem phi (LtlAsp N0 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N1 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N2 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N3 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N4 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N5 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N6 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N7 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N8 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N9 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N10 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N11 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N12 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N13 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N14 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N15 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N16 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N17 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N18 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N19 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N20 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N21 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N22 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N23 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N24 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N25 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N26 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N27 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N28 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N29 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N30 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N31 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N32 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N33 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N34 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N35 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N36 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N37 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N38 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N39 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N40 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N41 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N42 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N43 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N44 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N45 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N46 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N47 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N48 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N49 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N50 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N51 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N52 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N53 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N54 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N55 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N56 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N57 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N58 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N59 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N60 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N61 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N62 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N63 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N64 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N65 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N66 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N67 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N68 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N69 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N70 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N71 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N72 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N73 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N74 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N75 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N76 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N77 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N78 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N79 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N80 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N81 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N82 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N83 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N84 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N85 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N86 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N87 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N88 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N89 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N90 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N91 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N92 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N93 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N94 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N95 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N96 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N97 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N98 0))))
(constraint (not (Ltl.Sem phi (LtlAsp N99 0))))

(check-synth)
