(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_09: the fifth symbol from the right is 0.
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both char.0 and char.1.

(define-model p0____
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_0____
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__0____
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model n1????
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($1) true) (($2) true) (($3) true) (($4) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n?1????
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($2) true) (($3) true) (($4) true) (($5) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model n??1????
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($3) true) (($4) true) (($5) true) (($6) true) ((_) false))))
  (interp-fun one (match (((_) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0____ 0 (size (Pos p0____)))))
(constraint (RegLan.Sem r (ReAsp p_0____ 0 (size (Pos p_0____)))))
(constraint (RegLan.Sem r (ReAsp p__0____ 0 (size (Pos p__0____)))))

(constraint (not (RegLan.Sem r (ReAsp n1???? 0 (size (Pos n1????))))))
(constraint (not (RegLan.Sem r (ReAsp n?1???? 0 (size (Pos n?1????))))))
(constraint (not (RegLan.Sem r (ReAsp n??1???? 0 (size (Pos n??1????))))))

(check-synth)
