(require "../signatures/main.sl")
(provide p0 p00 p000 p111 p111111 p10101 p1001001 p100100110010010 n11 n011 n101 n110 n1111 n11111 n1111111)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model p111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false) (($4) true)))))

(define-model p1001001
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true) (($5) true) (($6) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false) (($5) false) (($6) true)))))

(define-model p100100110010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13 $14))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true) (($5) true) (($6) false) (($7) false) (($8) true) (($9) true) (($10) false) (($11) true) (($12) true) (($13) false) (($14) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false) (($5) false) (($6) true) (($7) true) (($8) false) (($9) false) (($10) true) (($11) false) (($12) false) (($13) true) (($14) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n011
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) true)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(define-model n11111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true)))))

(define-model n1111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true) (($6) true)))))
