(set-info :author "Chentian Wu")
(require "./theories/main.sl")

; GCPE_12: at least three 0 symbols and an even number of 1 symbols.

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p00011
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) true)))))

(define-model p0001111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true) (($4) true) (($5) true) (($6) true)))))

(define-model p000011
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true)))))

(define-model p00001111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false) (($6) false) (($7) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true) (($6) true) (($7) true)))))

(define-model p00000111111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) true) (($5) false) (($6) false) (($7) false) (($8) false) (($9) false) (($10) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) false) (($5) true) (($6) true) (($7) true) (($8) true) (($9) true) (($10) true)))))

(define-model n0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model n00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model n0001
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) true)))))

(define-model n00001
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true)))))

(define-model n0000111
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) (($3) true) (($4) false) (($5) false) (($6) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false) (($3) false) (($4) true) (($5) true) (($6) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p00011 0 (size (Pos p00011)))))
(constraint (RegLan.Sem r (ReAsp p0001111 0 (size (Pos p0001111)))))
(constraint (RegLan.Sem r (ReAsp p000011 0 (size (Pos p000011)))))
(constraint (RegLan.Sem r (ReAsp p00001111 0 (size (Pos p00001111)))))
(constraint (RegLan.Sem r (ReAsp p00000111111 0 (size (Pos p00000111111)))))

(constraint (not (RegLan.Sem r (ReAsp n0 0 (size (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n00 0 (size (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n0001 0 (size (Pos n0001))))))
(constraint (not (RegLan.Sem r (ReAsp n00001 0 (size (Pos n00001))))))
(constraint (not (RegLan.Sem r (ReAsp n0000111 0 (size (Pos n0000111))))))

(check-synth)
