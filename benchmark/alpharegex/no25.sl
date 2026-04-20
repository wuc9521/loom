(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_25: at most one 11 substring.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p101010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($1) true) (($3) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) ((_) false)))))

(define-model p00011000
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($5) true) (($6) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) ((_) false)))))

(define-model p00100110001
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($7) true) (($8) true) (($9) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($5) true) (($6) true) (($10) true) ((_) false)))))

(define-model n111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n110011
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($4) true) (($5) true) ((_) false)))))

(define-model n0110110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($3) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($4) true) (($5) true) ((_) false)))))

(define-model n00011001100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($5) true) (($6) true) (($9) true) (($10) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) (($7) true) (($8) true) ((_) false)))))

(define-model n011100011110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11))
  (interp-fun zero (match ((($0) true) (($4) true) (($5) true) (($6) true) (($11) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) (($7) true) (($8) true) (($9) true) (($10) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (len (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p101010 0 (len (Pos p101010)))))
(constraint (RegLan.Sem r (ReAsp p00011000 0 (len (Pos p00011000)))))
(constraint (RegLan.Sem r (ReAsp p00100110001 0 (len (Pos p00100110001)))))

(constraint (not (RegLan.Sem r (ReAsp n111 0 (len (Pos n111))))))
(constraint (not (RegLan.Sem r (ReAsp n110011 0 (len (Pos n110011))))))
(constraint (not (RegLan.Sem r (ReAsp n0110110 0 (len (Pos n0110110))))))
(constraint (not (RegLan.Sem r (ReAsp n00011001100 0 (len (Pos n00011001100))))))
(constraint (not (RegLan.Sem r (ReAsp n011100011110 0 (len (Pos n011100011110))))))

(check-synth)
