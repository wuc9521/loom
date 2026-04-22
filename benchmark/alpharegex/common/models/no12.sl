(require "../signatures/main.sl")
(provide p000 p00011 p0001111 p000011 p00001111 p00000111111 n0 n00 n0001 n00001 n0000111)

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p00011
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) true)))))

(define-model p0001111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) true) (($5) true) (($6) true)))))

(define-model p000011
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true)))))

(define-model p00001111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false) (($6) false) (($7) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true) (($6) true) (($7) true)))))

(define-model p00000111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) false) (($6) false) (($7) false) (($8) false) (($9) false) (($10) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) true) (($6) true) (($7) true) (($8) true) (($9) true) (($10) true)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n0001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true)))))

(define-model n00001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true)))))

(define-model n0000111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true) (($6) true)))))
