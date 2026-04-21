(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_11: translated from the upstream exact 0/1 examples.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) false)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false)))))

(define-model p100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false)))))

(define-model p10010
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model n0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) true)))))

(define-model n10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) false) (($4) true)))))

(define-model n100101
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) false) (($4) true) (($5) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false) (($3) true) (($4) false) (($5) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p10 0 (size (Pos p10)))))
(constraint (RegLan.Sem r (ReAsp p010 0 (size (Pos p010)))))
(constraint (RegLan.Sem r (ReAsp p100 0 (size (Pos p100)))))
(constraint (RegLan.Sem r (ReAsp p1010 0 (size (Pos p1010)))))
(constraint (RegLan.Sem r (ReAsp p10010 0 (size (Pos p10010)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n110 0 (size (Pos n110))))))
(constraint (not (RegLan.Sem r (ReAsp n01 0 (size (Pos n01))))))
(constraint (not (RegLan.Sem r (ReAsp n0101 0 (size (Pos n0101))))))
(constraint (not (RegLan.Sem r (ReAsp n10101 0 (size (Pos n10101))))))
(constraint (not (RegLan.Sem r (ReAsp n100101 0 (size (Pos n100101))))))

(check-synth)
