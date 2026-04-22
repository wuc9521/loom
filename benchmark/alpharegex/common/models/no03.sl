(require "../signatures/main.sl")
(provide p0101 p_0101_ p__0101__ n0 n1 n?? n??? n???0 n?0?? n1??? n??1?)

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) true)))))

(define-model p_0101_
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) true) (($5) false)))))

(define-model p__0101__
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) true) (($5) false) (($6) false) (($7) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) false) (($5) true) (($6) false) (($7) false)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model n??
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n???
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n???0
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) false)))))

(define-model n?0??
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) true)))))

(define-model n1???
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(define-model n??1?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))
