(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_02: word ends with 01
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both char.0 and char.1.

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model p_01
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true)))))

(define-model p__01
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) false)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n_?00
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false)))))

(define-model n_?10
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n_?11
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (len (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p_01 0 (len (Pos p_01)))))
(constraint (RegLan.Sem r (ReAsp p__01 0 (len (Pos p__01)))))

(constraint (not (RegLan.Sem r (ReAsp n00 0 (len (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (len (Pos n10))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (len (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n_?00 0 (len (Pos n_?00))))))
(constraint (not (RegLan.Sem r (ReAsp n_?10 0 (len (Pos n_?10))))))
(constraint (not (RegLan.Sem r (ReAsp n_?11 0 (len (Pos n_?11))))))

(check-synth)
