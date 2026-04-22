(require "../signatures/main.sl")
(provide p0 p001 p0001 p0010 p01000 p100000 n1 n101 n0110 n1010 n1001 n10001 n111000 n010000100 n000101001)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p0001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) ((_) false)))))

(define-model p0010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p01000
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p100000
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n0110
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model n1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model n10001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($4) true) ((_) false)))))

(define-model n111000
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) ((_) false)))))

(define-model n010000100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) (($5) true) (($7) true) (($8) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($6) true) ((_) false)))))

(define-model n000101001
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($4) true) (($6) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($5) true) (($8) true) ((_) false)))))
