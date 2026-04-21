(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_13: at most two 0 symbols.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) ((_) false)))))

(define-model p11011011
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) ((_) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n11011010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($5) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) ((_) false)))))

(define-model n11000
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n00011
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) ((_) false)))))

(define-model n11010101
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($2) true) (($4) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) (($5) true) (($7) true) ((_) false)))))

(define-model n0000
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n101010101
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($1) true) (($3) true) (($5) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) (($6) true) (($8) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p1 0 (size (Pos p1)))))
(constraint (RegLan.Sem r (ReAsp p00 0 (size (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (size (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p10101 0 (size (Pos p10101)))))
(constraint (RegLan.Sem r (ReAsp p11011011 0 (size (Pos p11011011)))))
(constraint (RegLan.Sem r (ReAsp p111 0 (size (Pos p111)))))
(constraint (RegLan.Sem r (ReAsp p1111 0 (size (Pos p1111)))))

(constraint (not (RegLan.Sem r (ReAsp n000 0 (size (Pos n000))))))
(constraint (not (RegLan.Sem r (ReAsp n11011010 0 (size (Pos n11011010))))))
(constraint (not (RegLan.Sem r (ReAsp n11000 0 (size (Pos n11000))))))
(constraint (not (RegLan.Sem r (ReAsp n00011 0 (size (Pos n00011))))))
(constraint (not (RegLan.Sem r (ReAsp n11010101 0 (size (Pos n11010101))))))
(constraint (not (RegLan.Sem r (ReAsp n0000 0 (size (Pos n0000))))))
(constraint (not (RegLan.Sem r (ReAsp n101010101 0 (size (Pos n101010101))))))

(check-synth)
