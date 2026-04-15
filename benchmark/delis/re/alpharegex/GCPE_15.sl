(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_15: accepts examples containing neither 0 nor 1.
; _ is matched only by re.any.

(define-model p__
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p___
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p____
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p__ 0 (len p__))))
(constraint (RegLan.Sem r (ReAsp p___ 0 (len p___))))
(constraint (RegLan.Sem r (ReAsp p____ 0 (len p____))))

(constraint (not (RegLan.Sem r (ReAsp n0 0 (len n0)))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (len n1)))))

(check-synth)
