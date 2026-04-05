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
    ; w1: ab
    (
      RegLan.Sem ((regex RegLan) (l Int) (r Int)) Bool
      :finite
    )  ; Each relation is declared as a function Sem : (members) -> Bool
  )
  ;; Bodies
  (
    (! ; The `!` expression attaches attributes to its first argument: in this case, the `:in` and `:out` labels below
      (match regex ; E.Sem defined inductively through matching
        (
          ($a (and (= 0 l) (= r 1))) ; evaluating $a yields the value of a in the relation
          ($b (and (= l 1) (= r 2))) ; evaluating $b yields the value of b in the relation
;;; ...
          (
            ($++ r1 r2)            ; et1 and et2 identify the child terms
            (exists ; this part is meta. 
              ((m Int))   ; r1 and r2 are auxiliary variables
              (and
                (<= l m) (<= m r)) ; the value of m is between l and r
                (RegLan.Sem r1 l m)  ; the value of r1 is determined by the values of l and m
                (RegLan.Sem r2 m r)  ; the value of r2 is determined by the values of m and r
              )                     ; the value of ($++ r1 r2) is the value of r1 plus the r2
          )                       ; when evaluated on the same inputs x and y
          (
            ($union r1 r2)           ; the value of ($union r1 r2) is the value of r1 or the value of r2
            (or
              (RegLan.Sem r1 l r)
              (RegLan.Sem r2 l r) 
            )
          )
          (
            ($* r1)
            (exists ((m Int))
              (and
                (<= l m) (<= m r)
                (or
                  (and (= l m) (= r m)) ; the value of ($* r1) is 1 if the value of r1 is 1 on the empty string
                  (and (RegLan.Sem r1 l m) (RegLan.Sem ($* r1) m r)) ; the value of ($* r1) is 1 if there exists a split of the input such that the value of r1 is 1 on the first part and the value of ($* r1) is 1 on the second part
                )
              )
            )
          )
        )
;;; ...
      )
    ) ; end `match regex`
    :input (x y) :output (r)      ; Indicate that the value of `r` is intended to be determined
  )                               
)

; ab
(define-model w1
  (define-univ Pos ($0 $1)) ; same as (define-datatype Pos ((pos0) (pos1)))
  (interp-fun a (match (
      (($0) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($0) false)
      ((_) true)
  )))
  (declare-semasp 3 3)
;   (define-semasp 
;   #[
;     #[0 0 0]
;     #[0 0 0]
;     #[0 0 0]
;   ])
)

; abab
(define-model w2
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun a (match (
      (($0) true)
      (($2) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($1) true)
      (($3) true)
      ((_) false)
  )))
  (declare-semasp 5 5)
)

; ababab
(define-model w3
  (define-univ Pos ($0 $1 $2 $3 $4 $5))
  (interp-fun a (match (
      (($0) true)
      (($2) true)
      (($4) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($1) true)
      (($3) true)
      (($5) true)
      ((_) false)
  )))
)


; baa
(define-model w4
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (
      (($0) false)
      ((_) true)
  )))
  (interp-fun b (match (
      (($0) true)
      (($1) true)
      ((_) false)
  )))
  (declare-semasp 4 4)
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
  (declare-semasp 4 4)
)

; some way to define syntax 
; some way to define semantics (similar to semgus)

(synth-fun regex () RegLan
    ((Start RegLan))
    ( 
        (Start RegLan
            ( 
                a
                b
                ($++ Start Start)    ; concat ($++ r1 r2) = ($union ($concat r1 r2) ($concat r2 r1))
                ($union Start Start) ; union
                ($inter Start Start) ; intersection
                ($comp Start)        ; complement
                ($* Start)           ; Kleene star
            )
        )
    )
)



(constraint (models w1 regex))        
(constraint (models w2 regex))        
(constraint (models w3 regex))        
(constraint (not (models w4 regex)))        
(constraint (not (models w5 regex)))        

(check-synth)
