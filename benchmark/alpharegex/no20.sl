(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_20: the constrained positions are 0 in positive examples.
; _ is matched only by re.any.
; ? is matched by both char.0 and char.1.

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) true))))
  (interp-fun one (match (((_) false)))))

(define-model p0_
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0_0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0_0_
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0_0_0
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p0_0_0_
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1?
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n??1
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1???
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n??1?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n1????
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n??1??
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n????1
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n????1?
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($5) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p0_ 0 (size (Pos p0_)))))
(constraint (RegLan.Sem r (ReAsp p0_0 0 (size (Pos p0_0)))))
(constraint (RegLan.Sem r (ReAsp p0_0_ 0 (size (Pos p0_0_)))))
(constraint (RegLan.Sem r (ReAsp p0_0_0 0 (size (Pos p0_0_0)))))
(constraint (RegLan.Sem r (ReAsp p0_0_0_ 0 (size (Pos p0_0_0_)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n1? 0 (size (Pos n1?))))))
(constraint (not (RegLan.Sem r (ReAsp n??1 0 (size (Pos n??1))))))
(constraint (not (RegLan.Sem r (ReAsp n1??? 0 (size (Pos n1???))))))
(constraint (not (RegLan.Sem r (ReAsp n??1? 0 (size (Pos n??1?))))))
(constraint (not (RegLan.Sem r (ReAsp n1???? 0 (size (Pos n1????))))))
(constraint (not (RegLan.Sem r (ReAsp n??1?? 0 (size (Pos n??1??))))))
(constraint (not (RegLan.Sem r (ReAsp n????1 0 (size (Pos n????1))))))
(constraint (not (RegLan.Sem r (ReAsp n????1? 0 (size (Pos n????1?))))))

(check-synth)
