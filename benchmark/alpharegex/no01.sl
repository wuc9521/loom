(set-info :author "Chentian Wu")
(require "./theories/main.sl")

(define-model p0
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) true))))
  (interp-fun one (match ((($0) false)))))

(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))

(define-model p00
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) true))))
  (interp-fun one (match ((($0) false) (($1) false)))))

(define-model p011
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) true)))))

(define-model p000
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) false)))))

(define-model p010
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) false) (($1) true) (($2) false)))))

(define-model p001
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) false) (($1) false) (($2) true)))))

(define-model n1
  (define-univ Pos ($0))
  (interp-fun zero (match ((($0) false))))
  (interp-fun one (match ((($0) true)))))

(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))

(define-model n10
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) true))))
  (interp-fun one (match ((($0) true) (($1) false)))))

(define-model n111
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) true)))))

(define-model n100
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) false)))))

(define-model n110
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) false) (($2) true))))
  (interp-fun one (match ((($0) true) (($1) true) (($2) false)))))

(define-model n101
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) false) (($1) true) (($2) false))))
  (interp-fun one (match ((($0) true) (($1) false) (($2) true)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p01 0 (size (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p00 0 (size (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p011 0 (size (Pos p011)))))
(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p010 0 (size (Pos p010)))))
(constraint (RegLan.Sem r (ReAsp p001 0 (size (Pos p001)))))

(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (size (Pos n10))))))
(constraint (not (RegLan.Sem r (ReAsp n111 0 (size (Pos n111))))))
(constraint (not (RegLan.Sem r (ReAsp n100 0 (size (Pos n100))))))
(constraint (not (RegLan.Sem r (ReAsp n110 0 (size (Pos n110))))))
(constraint (not (RegLan.Sem r (ReAsp n101 0 (size (Pos n101))))))

(check-synth)
