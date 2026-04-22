(require "../signatures/main.sl")
(provide p_ p_0 p11 p__0 p_11 p___0 p__11 p___11 n01 n?01 n??01)

(define-model p_
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_0
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p__0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_11
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p___0
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($3) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__11
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model p___11
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n?01
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n??01
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) ((_) false)))))
