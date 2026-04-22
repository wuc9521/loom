(require "../signatures/main.sl")
(provide p0 p11 p01010 p00100100 p1111 p010101010 p00100100100100 n1 n01 n10 n100 n111 n0101010 n11111)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01010
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p00100100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($5) true) ((_) false)))))

(define-model p1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p010101010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) (($6) true) (($8) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($5) true) (($7) true) ((_) false)))))

(define-model p00100100100100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) (($9) true) (($10) true) (($12) true) (($13) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($5) true) (($8) true) (($11) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n0101010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($5) true) ((_) false)))))

(define-model n11111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))
