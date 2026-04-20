(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_10: alternating symbols.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model p10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) false)))))

(define-model p101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false)))))

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false) (($4) true)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n011
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) true)))))

(define-model n001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true)))))

(define-model n101101
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) false) (($4) true) (($5) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) true) (($4) false) (($5) true)))))

(define-model n010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) true) (($4) false) (($5) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) false) (($4) true) (($5) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p1 0 (len (Pos p1)))))
(constraint (RegLan.Sem r (ReAsp p01 0 (len (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p10 0 (len (Pos p10)))))
(constraint (RegLan.Sem r (ReAsp p101 0 (len (Pos p101)))))
(constraint (RegLan.Sem r (ReAsp p010 0 (len (Pos p010)))))
(constraint (RegLan.Sem r (ReAsp p1010 0 (len (Pos p1010)))))
(constraint (RegLan.Sem r (ReAsp p0101 0 (len (Pos p0101)))))
(constraint (RegLan.Sem r (ReAsp p10101 0 (len (Pos p10101)))))

(constraint (not (RegLan.Sem r (ReAsp n00 0 (len (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (len (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n011 0 (len (Pos n011))))))
(constraint (not (RegLan.Sem r (ReAsp n001 0 (len (Pos n001))))))
(constraint (not (RegLan.Sem r (ReAsp n110 0 (len (Pos n110))))))
(constraint (not (RegLan.Sem r (ReAsp n1001 0 (len (Pos n1001))))))
(constraint (not (RegLan.Sem r (ReAsp n101101 0 (len (Pos n101101))))))
(constraint (not (RegLan.Sem r (ReAsp n010010 0 (len (Pos n010010))))))

(check-synth)
