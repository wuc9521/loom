(require "../signatures/main.sl")
(provide p0 p1 p01 p010 p0101 p1010 p00111 p001100111 p100110011 n00 n100 n001 n0010 n1001 n11001 n00101 n001100 n1001011)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model p00111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p001100111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($1) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($6) true) (($7) true) (($8) true) ((_) false)))))

(define-model p100110011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($1) true) (($2) true) (($5) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) (($4) true) (($7) true) (($8) true) ((_) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model n0010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model n11001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($4) true) ((_) false)))))

(define-model n00101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($4) true) ((_) false)))))

(define-model n001100
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model n1001011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($1) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) (($5) true) (($6) true) ((_) false)))))
