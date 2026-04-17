(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_21: 00 1pair.

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model p010010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($4) true) ((_) false)))))

(define-model p1011001110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9))
  (interp-fun zero (match ((($1) true) (($4) true) (($5) true) (($9) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($3) true) (($6) true) (($7) true) (($8) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n00100
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p00 0 (len (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p1001 0 (len (Pos p1001)))))
(constraint (RegLan.Sem r (ReAsp p010010 0 (len (Pos p010010)))))
(constraint (RegLan.Sem r (ReAsp p1011001110 0 (len (Pos p1011001110)))))

(constraint (not (RegLan.Sem r (ReAsp n01 0 (len (Pos n01))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (len (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n000 0 (len (Pos n000))))))
(constraint (not (RegLan.Sem r (ReAsp n00100 0 (len (Pos n00100))))))

(check-synth)
