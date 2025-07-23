(set-logic RE)

(declare-sort Pos 0)
(declare-sort Regex 0) 

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

; abab
(define-model w2
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun a (match (
      (($0) true)
      (($1) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($1) true)
      (($3) true)
      ((_) false)
  )))
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

(synth-fun regex () RegExp
    (Start RegExp)
    ( 
        (Start RegExp
            ( 
                re.nostr               ; \epsilon
                re.none                ; \emptyset
                (str.to.re "a")        ; regex of string "a"
                (str.to.re "b")        ; regex of string "b" 
                (re.++ Start Start)    ; concat
                (re.union Start Start) ; union
                (re.inter Start Start) ; intersection
                (re.opt Start)         ; optional
                (re.* Start)           ; Kleene star
                (re.loop Start Num Num); (re.loop r l u) repeat r for [l, u] times. If l \geq u, repeat exactly l times.
                (re.loop Start Num)    ; (re.loop r l) repeat r at least l times.
            )
        )
        (Num Int 
            (
                0 1 2 3 4 5 6
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
