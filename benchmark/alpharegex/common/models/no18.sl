(require "../signatures/main.sl")
(provide p0 p0110 p01110 p011110 p111 p01111 p1111110 n00 n010 n10101 n010111 n111010 n0011111 n1111100)

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p0110
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p01110
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) ((_) false)))))

(define-model p011110
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p1111110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) ((_) false)))))

(define-model n010111
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($4) true) (($5) true) ((_) false)))))

(define-model n111010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($3) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($4) true) ((_) false)))))

(define-model n0011111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($4) true) (($5) true) (($6) true) ((_) false)))))

(define-model n1111100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($5) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))
