(set-logic RE) ; regular grammar

(define-fun w1 () String 
    "ab"
)
(define-fun w2 () String
    "abab"
)
(define-fun w3 () String
    "ababab"
) 
(define-fun w4 () String
    "baa"
)
(define-fun w5 () String
    "aab"
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
