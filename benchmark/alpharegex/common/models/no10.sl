(require "../signatures/main.sl")
(provide p0 p1 p01 p10 p101 p010 p1010 p0101 p10101 n00 n11 n011 n001 n110 n1001 n101101 n010010)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model p10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) false)))))

(define-model p101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false)))))

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false) (($4) true)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n011
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) true)))))

(define-model n001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true)))))

(define-model n101101
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) false) (($4) true) (($5) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) true) (($4) false) (($5) true)))))

(define-model n010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) true) (($4) false) (($5) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) false) (($4) true) (($5) false)))))
