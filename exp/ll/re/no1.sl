(set-logic RE)

(declare-sort Pos 0)

; define signatures here.
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)


; ab
(define-model w1 (T) ; the syntax is similar to define-sort.
  (define-sort-enum Pos ($0 $1))
  (set-univ T Pos)
  (define-fun a ((x Pos)) Bool (match x (
      ($0 true)
      (_ false)
  )))
  (define-fun b ((x Pos)) Bool (match x (
      ($0 false)
      (_ true)
  )))
)

; abab
(define-model w2 (T)
  (define-sort-enum Pos ($0 $1 $2 $3))
  (set-univ T Pos)
  (define-fun a ((x Pos)) Bool (match x (
      ($0 true)
      ($2 true)
      (_ false)
  )))
  (define-fun b ((x Pos)) Bool (match x (
      ($1 true)
      ($3 true)
      (_ false)
  )))
)

; ababab
(define-model w3 (T)
  (define-sort-enum Pos ($0 $1 $2 $3 $4 $5))
  (set-univ T Pos)
  (define-fun a ((x Pos)) Bool (match x (
      ($0 true)
      ($2 true)
      ($4 true)
      (_ false)
  )))
  (define-fun b ((x Pos)) Bool (match x (
      ($1 true)
      ($3 true)
      ($5 true)
      (_ false)
  )))
)


; baa
(define-model w4 (T)
  (define-sort-enum Pos ($0 $1 $2))
  (set-univ T Pos)
  (define-fun a ((x Pos)) Bool (match x (
      ($0 false)
      (_ true)
  )))
  (define-fun b ((x Pos)) Bool (match x (
      ($0 true)
      ($1 true)
      (_ false)
  )))
)

; aab
(define-model w5 (T)
  (define-sort-enum Pos ($0 $1 $2))
  (set-univ T Pos)
  (define-fun a ((x Pos)) Bool (match x (
      ($0 true)
      (_ false)
  )))   
  (define-fun b ((x Pos)) Bool (match x (
      ($0 true)
      ($1 true)
      (_ false)
  )))
)

(synth-fun regex () RegLan
    ((Start RegLan))
    ( 
        (Start RegLan
            ( 
                ; (str.to.re "a")        ; regex of string "a"
                ; (str.to.re "b")        ; regex of string "b" 
                a
                b
                (re.++ Start Start)    ; concat (re.++ r1 r2) = (re.union (re.concat r1 r2) (re.concat r2 r1))
                (re.union Start Start) ; union
                (re.inter Start Start) ; intersection
                (re.comp Start)        ; complement
                (re.* Start)           ; Kleene star
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
