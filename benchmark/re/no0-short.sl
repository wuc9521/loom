(set-logic ALL)

(declare-term-types  
 ((RegLan 0))  ; Each term type corresponds to a nonterminal in the default grammar
 (
   ( ; Productions for term type E
      (re.a)      ; The leading `$`-signs are a naming convention with no special significance.
      (re.b) 
      (re.++ RegLan RegLan)  ; The $+ production has two child terms of type RegLan
      (re.union RegLan RegLan)
      (re.inter RegLan RegLan)
      (re.comp RegLan)
      (re.* RegLan)
    )
  )
)

(declare-sort Pos 0)

; define signatures here.
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)


; ab
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (
      (($0) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($0) false)
      ((_) true)
  )))
)

; aab
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (
      (($0) true)
      ((_) false)
  )))   
  (interp-fun b (match (
      (($0) true)
      (($1) true)
      ((_) false)
  )))
)


(define-finite ReAsp
  (record (model Model) (i Int) (j Int))
  :where (and (<= 0 i) (<= i j) (<= j (len (Pos model)))))

(define-semantics
  ((RegLan.Sem (r RegLan) (asp ReAsp)))
  ((match r
     ((re.a) (and (= (j asp) (+ (i asp) 1)) (a (i asp))))
     ((re.b) (and (= (j asp) (+ (i asp) 1)) (b (i asp))))
     ((re.++ x y)
      (exists ((k (int-range (i asp) (j asp))))
        (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
             (RegLan.Sem y (ReAsp (model asp) k (j asp))))))
     ((re.union x y)
      (or (RegLan.Sem x asp) (RegLan.Sem y asp)))
     ((re.inter x y)
      (and (RegLan.Sem x asp) (RegLan.Sem y asp)))
     ((re.comp x)
      (not (RegLan.Sem x asp)))
     ((re.* x)
      (or (= (i asp) (j asp))
          (exists ((k (int-range (+ (i asp) 1) (j asp))))
            (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
                 (RegLan.Sem r (ReAsp (model asp) k (j asp))))))))))

(synth-fun r () RegLan)
(constraint (RegLan.Sem r (ReAsp w1 0 (len (Pos w1)))))              
(constraint (not (RegLan.Sem r (ReAsp w5 0 (len (Pos w5))))))        

(check-synth)
