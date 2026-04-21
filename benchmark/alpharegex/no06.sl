(set-info :author "Chentian Wu")

; GCPE_06: length is a multiple of 3.
; '_' marks a position matched only by re.any.
; '?' marks a position matched by both re.0 and re.1.

; '?' marks a position matched by all character classes in the source benchmark.

(declare-sort Pos 0)

(declare-term-types
 ((RegLan 0))
 (((re.any)
   (re.? RegLan)
   (re.++ RegLan RegLan)
   (re.union RegLan RegLan)
    (re.* RegLan))))

(define-finite ReAsp                                             
  (record (model Model) (i Int) (j Int))                         
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))

(define-semantics
  ((RegLan.Sem (r RegLan) (asp ReAsp)))
  ((match r
     ((char.0)
      (and (= (j asp) (+ (i asp) 1)) (zero (i asp))))
     ((char.1)
      (and (= (j asp) (+ (i asp) 1)) (one (i asp))))
     ((re.any)
      (= (j asp) (+ (i asp) 1)))
     ((re.? x)
      (or (= (i asp) (j asp))
          (RegLan.Sem x asp)))
     ((re.++ x y)
      (exists ((k (int-range (i asp) (j asp))))
        (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
             (RegLan.Sem y (ReAsp (model asp) k (j asp))))))
     ((re.union x y)
      (or (RegLan.Sem x asp) (RegLan.Sem y asp)))
     ((re.* x)
      (or (= (i asp) (j asp))
          (exists ((k (int-range (+ (i asp) 1) (j asp))))
            (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
                 (RegLan.Sem r (ReAsp (model asp) k (j asp))))))))))

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
