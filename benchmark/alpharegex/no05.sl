(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_05: length is at least 3 and the third symbol is 0.
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both char.0 and char.1.

(define-model p__0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p__0_
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false)))))

(define-model p__0__
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false)))))

(define-model n?
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) true)))))

(define-model n??
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n??1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n??1?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p__0 0 (len (Pos p__0)))))
(constraint (RegLan.Sem r (ReAsp p__0_ 0 (len (Pos p__0_)))))
(constraint (RegLan.Sem r (ReAsp p__0__ 0 (len (Pos p__0__)))))

(constraint (not (RegLan.Sem r (ReAsp n? 0 (len (Pos n?))))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (len (Pos n??))))))
(constraint (not (RegLan.Sem r (ReAsp n??1 0 (len (Pos n??1))))))
(constraint (not (RegLan.Sem r (ReAsp n??1? 0 (len (Pos n??1?))))))

(check-synth)
