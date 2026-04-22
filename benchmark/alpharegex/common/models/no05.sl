(require "../signatures/main.sl")
(provide p__0 p__0_ p__0__ n? n?? n??1 n??1?)

(define-model p__0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p__0_
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false)))))

(define-model p__0__
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false)))))

(define-model n?
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) true)))))

(define-model n??
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n??1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n??1?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))
