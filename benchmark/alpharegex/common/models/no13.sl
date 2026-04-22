(require "../signatures/main.sl")
(provide p0 p1 p00 p11 p10101 p11011011 p111 p1111 n000 n11011010 n11000 n00011 n11010101 n0000 n101010101)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) ((_) false)))))

(define-model p11011011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) ((_) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n11011010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($5) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) ((_) false)))))

(define-model n11000
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n00011
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) ((_) false)))))

(define-model n11010101
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($4) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($5) true) (($7) true) ((_) false)))))

(define-model n0000
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n101010101
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($1) true) (($3) true) (($5) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) (($6) true) (($8) true) ((_) false)))))
