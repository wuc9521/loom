(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_04: word begins with 0 and ends with 1.
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both char.0 and char.1.

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model p0_1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true)))))

(define-model p0__1
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n0?0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false)))))

(define-model n1?1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n1?0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (len (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p0_1 0 (len (Pos p0_1)))))
(constraint (RegLan.Sem r (ReAsp p0__1 0 (len (Pos p0__1)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (len (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n0 0 (len (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n00 0 (len (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n0?0 0 (len (Pos n0?0))))))
(constraint (not (RegLan.Sem r (ReAsp n1?1 0 (len (Pos n1?1))))))
(constraint (not (RegLan.Sem r (ReAsp n1?0 0 (len (Pos n1?0))))))

(check-synth)
