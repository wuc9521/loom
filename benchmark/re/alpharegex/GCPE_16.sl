(set-info :author "Chentian Wu")
(require "../main.sl")

; GCPE_16: does not end with 01.
; _ is matched only by re.any.
; ? is matched by both char.0 and char.1.

(define-model p_
  (define-univ Pos ($0))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_0
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($1) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match (((_) true)))))

(define-model p__0
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($2) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p_11
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($1) true) (($2) true) ((_) false)))))

(define-model p___0
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($3) true) ((_) false))))
  (interp-fun one (match (((_) false)))))

(define-model p__11
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($2) true) (($3) true) ((_) false)))))

(define-model p___11
  (define-univ Pos ($0 $1 $2 $3 $4))
  (interp-fun zero (match (((_) false))))
  (interp-fun one (match ((($3) true) (($4) true) ((_) false)))))

(define-model n01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) ((_) false))))
  (interp-fun one (match ((($1) true) ((_) false)))))

(define-model n?01
  (define-univ Pos ($0 $1 $2))
  (interp-fun zero (match ((($0) true) (($1) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($2) true) ((_) false)))))

(define-model n??01
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun zero (match ((($0) true) (($1) true) (($2) true) ((_) false))))
  (interp-fun one (match ((($0) true) (($1) true) (($3) true) ((_) false)))))

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p_ 0 (len (Pos p_)))))
(constraint (RegLan.Sem r (ReAsp p_0 0 (len (Pos p_0)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (len (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p__0 0 (len (Pos p__0)))))
(constraint (RegLan.Sem r (ReAsp p_11 0 (len (Pos p_11)))))
(constraint (RegLan.Sem r (ReAsp p___0 0 (len (Pos p___0)))))
(constraint (RegLan.Sem r (ReAsp p__11 0 (len (Pos p__11)))))
(constraint (RegLan.Sem r (ReAsp p___11 0 (len (Pos p___11)))))

(constraint (not (RegLan.Sem r (ReAsp n01 0 (len (Pos n01))))))
(constraint (not (RegLan.Sem r (ReAsp n?01 0 (len (Pos n?01))))))
(constraint (not (RegLan.Sem r (ReAsp n??01 0 (len (Pos n??01))))))

(check-synth)
