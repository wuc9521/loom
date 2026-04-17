(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_17: at least one 0 symbol and at most one 1 symbol.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p0001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) ((_) false)))))

(define-model p0010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p01000
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p100000
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n0110
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model n1010
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n1001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model n10001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($4) true) ((_) false)))))

(define-model n111000
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) ((_) false)))))

(define-model n010000100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) (($5) true) (($7) true) (($8) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($6) true) ((_) false)))))

(define-model n000101001
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($4) true) (($6) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($3) true) (($5) true) (($8) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p001 0 (len (Pos p001)))))
(constraint (RegLan.Sem r (ReAsp p0001 0 (len (Pos p0001)))))
(constraint (RegLan.Sem r (ReAsp p0010 0 (len (Pos p0010)))))
(constraint (RegLan.Sem r (ReAsp p01000 0 (len (Pos p01000)))))
(constraint (RegLan.Sem r (ReAsp p100000 0 (len (Pos p100000)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (len (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n101 0 (len (Pos n101))))))
(constraint (not (RegLan.Sem r (ReAsp n0110 0 (len (Pos n0110))))))
(constraint (not (RegLan.Sem r (ReAsp n1010 0 (len (Pos n1010))))))
(constraint (not (RegLan.Sem r (ReAsp n1001 0 (len (Pos n1001))))))
(constraint (not (RegLan.Sem r (ReAsp n10001 0 (len (Pos n10001))))))
(constraint (not (RegLan.Sem r (ReAsp n111000 0 (len (Pos n111000))))))
(constraint (not (RegLan.Sem r (ReAsp n010000100 0 (len (Pos n010000100))))))
(constraint (not (RegLan.Sem r (ReAsp n000101001 0 (len (Pos n000101001))))))

(check-synth)
