(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_24: translated from the upstream positive and negative examples exactly.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model p00111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p001100111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($1) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($6) true) (($7) true) (($8) true) ((_) false)))))

(define-model p100110011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($1) true) (($2) true) (($5) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) (($4) true) (($7) true) (($8) true) ((_) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model n0010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model n11001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($4) true) ((_) false)))))

(define-model n00101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($4) true) ((_) false)))))

(define-model n001100
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model n1001011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($1) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) (($5) true) (($6) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p1 0 (len (Pos p1)))))
(constraint (RegLan.Sem r (ReAsp p01 0 (len (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p010 0 (len (Pos p010)))))
(constraint (RegLan.Sem r (ReAsp p0101 0 (len (Pos p0101)))))
(constraint (RegLan.Sem r (ReAsp p1010 0 (len (Pos p1010)))))
(constraint (RegLan.Sem r (ReAsp p00111 0 (len (Pos p00111)))))
(constraint (RegLan.Sem r (ReAsp p001100111 0 (len (Pos p001100111)))))
(constraint (RegLan.Sem r (ReAsp p100110011 0 (len (Pos p100110011)))))

(constraint (not (RegLan.Sem r (ReAsp n00 0 (len (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n100 0 (len (Pos n100))))))
(constraint (not (RegLan.Sem r (ReAsp n001 0 (len (Pos n001))))))
(constraint (not (RegLan.Sem r (ReAsp n0010 0 (len (Pos n0010))))))
(constraint (not (RegLan.Sem r (ReAsp n1001 0 (len (Pos n1001))))))
(constraint (not (RegLan.Sem r (ReAsp n11001 0 (len (Pos n11001))))))
(constraint (not (RegLan.Sem r (ReAsp n00101 0 (len (Pos n00101))))))
(constraint (not (RegLan.Sem r (ReAsp n001100 0 (len (Pos n001100))))))
(constraint (not (RegLan.Sem r (ReAsp n1001011 0 (len (Pos n1001011))))))

(check-synth)
