(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_22: upstream label says len > 2 and not ending with 10; constraints include length-2 positives exactly.
; _ is matched only by re.any.
; ? is matched by both char.0 and char.1.

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p_00
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_01
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($2) true) ((_) false)))))

(define-model p_11
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p__00
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($2) true) (($3) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__01
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match ((($3) true) ((_) false)))))

(define-model p__11
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model n?
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) true)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n?10
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) ((_) false)))))

(define-model n??10
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p00 0 (len (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p01 0 (len (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (len (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p_00 0 (len (Pos p_00)))))
(constraint (RegLan.Sem r (ReAsp p_01 0 (len (Pos p_01)))))
(constraint (RegLan.Sem r (ReAsp p_11 0 (len (Pos p_11)))))
(constraint (RegLan.Sem r (ReAsp p__00 0 (len (Pos p__00)))))
(constraint (RegLan.Sem r (ReAsp p__01 0 (len (Pos p__01)))))
(constraint (RegLan.Sem r (ReAsp p__11 0 (len (Pos p__11)))))

(constraint (not (RegLan.Sem r (ReAsp n? 0 (len (Pos n?))))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (len (Pos n10))))))
(constraint (not (RegLan.Sem r (ReAsp n?10 0 (len (Pos n?10))))))
(constraint (not (RegLan.Sem r (ReAsp n??10 0 (len (Pos n??10))))))

(check-synth)
