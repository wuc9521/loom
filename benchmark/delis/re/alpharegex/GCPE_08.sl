(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_08: translated from the upstream example constraints.
; The upstream comment says "0 count = 2k", but the listed examples are not
; exactly the even-zero-count language, so this file follows the constraints.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p01010
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p00100100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($5) true) ((_) false)))))

(define-model p1111
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p010101010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) (($6) true) (($8) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($5) true) (($7) true) ((_) false)))))

(define-model p00100100100100
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11 $12 $13))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($6) true) (($7) true) (($9) true) (($10) true) (($12) true) (($13) true) ((_) false))))
  (interp-fun one (match ((($2) true) (($5) true) (($8) true) (($11) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n0101010
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) (($6) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) (($5) true) ((_) false)))))

(define-model n11111
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len p0))))
(constraint (RegLan.Sem r (ReAsp p11 0 (len p11))))
(constraint (RegLan.Sem r (ReAsp p01010 0 (len p01010))))
(constraint (RegLan.Sem r (ReAsp p00100100 0 (len p00100100))))
(constraint (RegLan.Sem r (ReAsp p1111 0 (len p1111))))
(constraint (RegLan.Sem r (ReAsp p010101010 0 (len p010101010))))
(constraint (RegLan.Sem r (ReAsp p00100100100100 0 (len p00100100100100))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (len n1)))))
(constraint (not (RegLan.Sem r (ReAsp n01 0 (len n01)))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (len n10)))))
(constraint (not (RegLan.Sem r (ReAsp n100 0 (len n100)))))
(constraint (not (RegLan.Sem r (ReAsp n111 0 (len n111)))))
(constraint (not (RegLan.Sem r (ReAsp n0101010 0 (len n0101010)))))
(constraint (not (RegLan.Sem r (ReAsp n11111 0 (len n11111)))))

(check-synth)
