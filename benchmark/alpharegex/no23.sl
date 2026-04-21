(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_23: translated from the upstream positive and negative examples exactly.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model p01010
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p1000100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($5) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($4) true) ((_) false)))))

(define-model p10010010001000
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13))
  (interp-fun zero (match ((($1) true) (($2) true) (($4) true) (($5) true) (($7) true) (($8) true) (($9) true) (($11) true) (($12) true) (($13) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) (($6) true) (($10) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model n001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model n01110
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) ((_) false)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p1010 0 (size (Pos p1010)))))
(constraint (RegLan.Sem r (ReAsp p01010 0 (size (Pos p01010)))))
(constraint (RegLan.Sem r (ReAsp p1000100 0 (size (Pos p1000100)))))
(constraint (RegLan.Sem r (ReAsp p10010010001000 0 (size (Pos p10010010001000)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n1111 0 (size (Pos n1111))))))
(constraint (not (RegLan.Sem r (ReAsp n101 0 (size (Pos n101))))))
(constraint (not (RegLan.Sem r (ReAsp n0101 0 (size (Pos n0101))))))
(constraint (not (RegLan.Sem r (ReAsp n001 0 (size (Pos n001))))))
(constraint (not (RegLan.Sem r (ReAsp n01110 0 (size (Pos n01110))))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (size (Pos n10))))))
(constraint (not (RegLan.Sem r (ReAsp n110 0 (size (Pos n110))))))
(constraint (not (RegLan.Sem r (ReAsp n100 0 (size (Pos n100))))))

(check-synth)
