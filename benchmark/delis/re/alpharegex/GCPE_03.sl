(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_03: word contains 0101
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both re.0 and re.1.

(define-model p0101
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false) (($3) true)))))

(define-model p_0101_
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false) (($3) true) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) true) (($5) false)))))

(define-model p__0101__
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true) (($3) false) (($4) true) (($5) false) (($6) false) (($7) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) false) (($5) true) (($6) false) (($7) false)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model n??
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n???
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n???0
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) false)))))

(define-model n?0??
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true) (($3) true)))))

(define-model n1???
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(define-model n??1?
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false) (($3) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true) (($3) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0101 0 (len p0101))))
(constraint (RegLan.Sem r (ReAsp p_0101_ 0 (len p_0101_))))
(constraint (RegLan.Sem r (ReAsp p__0101__ 0 (len p__0101__))))

(constraint (not (RegLan.Sem r (ReAsp n0 0 (len n0)))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (len n1)))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (len n??)))))
(constraint (not (RegLan.Sem r (ReAsp n??? 0 (len n???)))))
(constraint (not (RegLan.Sem r (ReAsp n???0 0 (len n???0)))))
(constraint (not (RegLan.Sem r (ReAsp n?0?? 0 (len n?0??)))))
(constraint (not (RegLan.Sem r (ReAsp n1??? 0 (len n1???)))))
(constraint (not (RegLan.Sem r (ReAsp n??1? 0 (len n??1?)))))

(check-synth)
