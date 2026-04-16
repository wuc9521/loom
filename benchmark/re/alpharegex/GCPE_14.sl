(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_14: starts with 0 at odd length, or starts with 1 at even length.
; _ is matched only by re.any.
; ? is matched by both char.0 and char.1.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p0__
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0____
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p1_
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model p1___
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($0) true) ((_) false)))))

(define-model n0?
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n0???
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match ((($1) true) (($2) true) (($3) true) ((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1??
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($1) true) (($2) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1????
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (len p0))))
(constraint (RegLan.Sem r (ReAsp p0__ 0 (len p0__))))
(constraint (RegLan.Sem r (ReAsp p0____ 0 (len p0____))))
(constraint (RegLan.Sem r (ReAsp p1_ 0 (len p1_))))
(constraint (RegLan.Sem r (ReAsp p1___ 0 (len p1___))))

(constraint (not (RegLan.Sem r (ReAsp n0? 0 (len n0?)))))
(constraint (not (RegLan.Sem r (ReAsp n0??? 0 (len n0???)))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (len n1)))))
(constraint (not (RegLan.Sem r (ReAsp n1?? 0 (len n1??)))))
(constraint (not (RegLan.Sem r (ReAsp n1???? 0 (len n1????)))))

(check-synth)
