(require "../signatures/main.sl")
(provide p00 p01 p11 p_00 p_01 p_11 p__00 p__01 p__11 n? n10 n?10 n??10)

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p_00
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_01
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p_11
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p__00
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($2) true) (($3) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__01
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) ((_) false)))))

(define-model p__11
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model n?
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) true)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n?10
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n??10
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) ((_) false)))))
