;! in the meta language, (forall ...) and (exists ...) are essentially quantified over semantic aspect.
;! maybe we should come up with a better abstraction other than semantic aspect
;! maybe about the termination of recursions. (see guarded recursion)
(set-logic RE)

(declare-term-types  
 ((RegLan 0))  ; Each term type corresponds to a nonterminal in the default grammar
 (
   ( ; Productions for term type E
      ($a)      ; The leading `$`-signs are a naming convention with no special significance.
      ($b) 
      ($empty)  ; non-recursive, constant production
      ($++ RegLan RegLan)  ; The $+ production has two child terms of type RegLan
      ($union RegLan RegLan)
      ($inter RegLan RegLan)
      ($comp RegLan)
      ($* RegLan)
    )
  )
)

; (declare-semasp RegSemAsp 0)  ; Semantic aspect for regular languages

; ab
(define-model w1 (r RegLan) (asp (SemAsp 3 3)) ; (define-model <NAME> (<VAR> <SYNTAX>) (<SEMASP>))
    (match r
        (
            ($a [
                [0 1 0]
                [0 0 0]
                [0 0 0]
            ])  ; The value of the production $a is 1 in the semantics
            ($b [
                [0 0 0]
                [⊥ 0 1]
                [⊥ ⊥ 0]
            ])
            ($empty [
                [0 0 0]
                [0 0 0]
                [0 0 0]
            ])
            ($++ r1 r2)  ; The value of the production $++ is the sum of the values of its children
                (exists ((v1 Int) (v2 Int))
                    (and
                        (SemAsp r1 v1)  ; The value of the child term r1 is v1
                        (SemAsp r2 v2)  ; The value of the child term r2 is v2
                        (= v (+ v1 v2)) ; The value of the parent term is the sum of the values of the child terms
                    )
                )
        )
    )
)

(define-funs-rec
  ;; Semantic relations and their types
  ; In the setting of logic learning, all aux vairables and ret should have Bool type.
  ; we need a meta language to describe semantic aspects. Using everything as a bool variable is not elegant.
  (
    (RegLan.Sem ((r RegLan) (X (array Bool))) Bool)  
    ; X = ["", "a", "ab", "", "b", ""] denoting all substrings.
  )
  ;; Bodies
  (
    (! ; The `!` expression attaches attributes to its first argument: in this case, the `:in` and `:out` labels below
      (match r ; E.Sem defined inductively through matching
        (
          ($a 
            (and 
              X[1]
                (and (not X[0]) 
                  (and (not X[2])
                    (and (not X[3])
                      (and (not X[4])
                        (not X[5])) ; the value of $a is 1 on "a" and 0 on "" and "b"
              ))))
          ) ; evaluating $a yields the value of a in the relation
          ($b 
            (and 
              X[4]
                (and (not X[0]) 
                  (and (not X[1])
                    (and (not X[2])
                      (and (not X[3])
                        (not X[5])) ; the value of $b is 1 on "b" and 0 on "" and "a"
              ))))
          ) 
          (
            ($++ r1 r2)            ; et1 and et2 identify the child terms
            (exists ; this part is meta. 
              ((m Int))   ; r1 and r2 are auxiliary variables
              (and
                (<= (l asp) m) (<= m (r asp))) ; the value of m is between l and r
                (RegLan.Sem r1 (l asp) m)  ; the value of r1 is determined by the values of l and m
                (RegLan.Sem r2 m (r asp))  ; the value of r2 is determined by the values of m and r
              )                     ; the value of ($++ r1 r2) is the value of r1 plus the r2
          )                       ; when evaluated on the same inputs x and y
          (
            ($union r1 r2)           ; the value of ($union r1 r2) is the value of r1 or the value of r2
            (or
              (RegLan.Sem r1 (l asp) (r asp))
              (RegLan.Sem r2 (l asp) (r asp)) 
            )
          )
          (
            ($* r1)
            (exists ((m Int))
              (and
                (<= (l asp) m) (<= m (r asp))
                (or
                  (and (= (l asp) m) (= (r asp) m)) ; the value of ($* r1) is 1 if the value of r1 is 1 on the empty string
                  (and (RegLan.Sem r1 (l asp) m) (RegLan.Sem ($* r1) m (r asp))) ; the value of ($* r1) is 1 if there exists a split of the input such that the value of r1 is 1 on the first part and the value of ($* r1) is 1 on the second part
                )
              )
            )
          )
        )
;;; ...
      )
    ) ; end `match r`
    :aspects(X)  
  )                               
)

; ab
; abab
; ababab
; baa
; aab

; some way to define syntax 
; some way to define semantics (similar to semgus)

(synth-fun r () RegLan)

(constraint (models w1 r))        
(constraint (models w2 r))        
(constraint (models w3 r))        
(constraint (not (models w4 r)))        
(constraint (not (models w5 r)))        

(check-synth)
