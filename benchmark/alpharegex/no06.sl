(set-info :author "Chentian Wu")
(require "../theories/main.sl")

; GCPE_06: length is a multiple of 3.
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both re.0 and re.1.

; '?' marks a position matched by all character classes in the source benchmark.

(declare-sort Pos 0)

(define-model p___ (define-univ Pos ($0 $1 $2)))

(define-model p______ (define-univ Pos ($0 $1 $2 $3 $4 $5)))

(define-model n? (define-univ Pos ($0)))

(define-model n?? (define-univ Pos ($0 $1)) )

(define-model n???? (define-univ Pos ($0 $1 $2 $3)) )

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p___ 0 (size (Pos p___)))))
(constraint (RegLan.Sem r (ReAsp p______ 0 (size (Pos p______)))))

(constraint (not (RegLan.Sem r (ReAsp n? 0 (size (Pos n?))))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (size (Pos n??))))))
(constraint (not (RegLan.Sem r (ReAsp n???? 0 (size (Pos n????))))))

(check-synth)
