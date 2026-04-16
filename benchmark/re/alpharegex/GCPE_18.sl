(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_18: at least two 1 symbols between any two 0 symbols.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p0110
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p01110
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) ((_) false)))))

(define-model p011110
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))

(define-model p1111110
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n10101
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) (($4) true) ((_) false)))))

(define-model n010111
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($4) true) (($5) true) ((_) false)))))

(define-model n111010
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($3) true) (($5) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($4) true) ((_) false)))))

(define-model n0011111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) (($4) true) (($5) true) (($6) true) ((_) false)))))

(define-model n1111100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($5) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len p0))))
(constraint (RegLan.Sem r (ReAsp p0110 0 (len p0110))))
(constraint (RegLan.Sem r (ReAsp p01110 0 (len p01110))))
(constraint (RegLan.Sem r (ReAsp p011110 0 (len p011110))))
(constraint (RegLan.Sem r (ReAsp p111 0 (len p111))))
(constraint (RegLan.Sem r (ReAsp p01111 0 (len p01111))))
(constraint (RegLan.Sem r (ReAsp p1111110 0 (len p1111110))))

(constraint (not (RegLan.Sem r (ReAsp n00 0 (len n00)))))
(constraint (not (RegLan.Sem r (ReAsp n010 0 (len n010)))))
(constraint (not (RegLan.Sem r (ReAsp n10101 0 (len n10101)))))
(constraint (not (RegLan.Sem r (ReAsp n010111 0 (len n010111)))))
(constraint (not (RegLan.Sem r (ReAsp n111010 0 (len n111010)))))
(constraint (not (RegLan.Sem r (ReAsp n0011111 0 (len n0011111)))))
(constraint (not (RegLan.Sem r (ReAsp n1111100 0 (len n1111100)))))

(check-synth)
