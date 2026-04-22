(require "../signatures/main.sl")
(provide p0____ p_0____ p__0____ n1???? n?1???? n??1????)

(define-model p0____
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_0____
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__0____
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model n1????
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n?1????
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n??1????
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($5) true) (($6) true) ((_) false))))
  (interp-fun one (match (((_) true)))))
