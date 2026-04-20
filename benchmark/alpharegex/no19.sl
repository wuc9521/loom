(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_19: does not contain 100 as a substring.
; _ is matched only by re.any.
; ? is matched by both char.0 and char.1.

(define-model p_
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0__
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_1_
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p__1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p_101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match ((($1) true) (($3) true) ((_) false)))))

(define-model p0_01_
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) ((_) false)))))

(define-model p__1_1_1
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) (($4) true) (($6) true) ((_) false)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n?100
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n100?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($3) true) ((_) false)))))

(define-model n?100?
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($4) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p_ 0 (len (Pos p_)))))
(constraint (RegLan.Sem r (ReAsp p__ 0 (len (Pos p__)))))
(constraint (RegLan.Sem r (ReAsp p0__ 0 (len (Pos p0__)))))
(constraint (RegLan.Sem r (ReAsp p_1_ 0 (len (Pos p_1_)))))
(constraint (RegLan.Sem r (ReAsp p__1 0 (len (Pos p__1)))))
(constraint (RegLan.Sem r (ReAsp p_101 0 (len (Pos p_101)))))
(constraint (RegLan.Sem r (ReAsp p0_01_ 0 (len (Pos p0_01_)))))
(constraint (RegLan.Sem r (ReAsp p__1_1_1 0 (len (Pos p__1_1_1)))))

(constraint (not (RegLan.Sem r (ReAsp n100 0 (len (Pos n100))))))
(constraint (not (RegLan.Sem r (ReAsp n?100 0 (len (Pos n?100))))))
(constraint (not (RegLan.Sem r (ReAsp n100? 0 (len (Pos n100?))))))
(constraint (not (RegLan.Sem r (ReAsp n?100? 0 (len (Pos n?100?))))))

(check-synth)
