(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_07: the number of 1 symbols is a multiple of 3.
; This follows the upstream constraints; the upstream comment says "0 count = 3k".

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model p111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false) (($4) true)))))

(define-model p1001001
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true) (($5) true) (($6) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false) (($5) false) (($6) true)))))

(define-model p100100110010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13 $14))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true) (($5) true) (($6) false) (($7) false) (($8) true) (($9) true) (($10) false) (($11) true) (($12) true) (($13) false) (($14) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false) (($5) false) (($6) true) (($7) true) (($8) false) (($9) false) (($10) true) (($11) false) (($12) false) (($13) true) (($14) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n011
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) true)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(define-model n11111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true)))))

(define-model n1111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true) (($6) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p00 0 (size (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p111 0 (size (Pos p111)))))
(constraint (RegLan.Sem r (ReAsp p111111 0 (size (Pos p111111)))))
(constraint (RegLan.Sem r (ReAsp p10101 0 (size (Pos p10101)))))
(constraint (RegLan.Sem r (ReAsp p1001001 0 (size (Pos p1001001)))))
(constraint (RegLan.Sem r (ReAsp p100100110010010 0 (size (Pos p100100110010010)))))

(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n011 0 (size (Pos n011))))))
(constraint (not (RegLan.Sem r (ReAsp n101 0 (size (Pos n101))))))
(constraint (not (RegLan.Sem r (ReAsp n110 0 (size (Pos n110))))))
(constraint (not (RegLan.Sem r (ReAsp n1111 0 (size (Pos n1111))))))
(constraint (not (RegLan.Sem r (ReAsp n11111 0 (size (Pos n11111))))))
(constraint (not (RegLan.Sem r (ReAsp n1111111 0 (size (Pos n1111111))))))

(check-synth)
