(require "../signatures/main.sl")
(provide p00 p1001 p010010 p1011001110 n01 n11 n000 n00100)

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model p010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($4) true) ((_) false)))))

(define-model p1011001110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9))
  (interp-fun zero (match ((($1) true) (($4) true) (($5) true) (($9) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($3) true) (($6) true) (($7) true) (($8) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n00100
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))
