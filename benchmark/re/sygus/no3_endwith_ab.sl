(set-logic SLIA) 

(define-fun w1 () String
    "ab"
)
(define-fun w2 () String
    "Xab"
)
(define-fun w3 () String
    "XXab"
)
(define-fun w4 () String
    "aa"
)
(define-fun w5 () String
    "ba"
)
(define-fun w6 () String
    "bb"
)
(define-fun w7 () String 
    "Xaa"
)
(define-fun w8 () String 
    "Xba"
)
(define-fun w9 () String 
    "Xbb"
)

(synth-fun regex () RegLan
    ((Start RegLan))
    ( 
        (Start RegLan
            ( 
                (str.to_re "a")        ; regex of string "a"
                (str.to_re "b")        ; regex of string "b" 
                (str.to_re "X")        ; regex of string "X"
                (re.++ Start Start)    ; concat
                (re.union Start Start) ; union
                (re.inter Start Start) ; intersection
                (re.* Start)           ; Kleene star
            )
        )
    )
)

(constraint (str.in_re w1 regex))
(constraint (str.in_re w2 regex))
(constraint (str.in_re w3 regex))
(constraint (not (str.in_re w4 regex)))
(constraint (not (str.in_re w5 regex)))
(constraint (not (str.in_re w6 regex)))
(constraint (not (str.in_re w7 regex)))
(constraint (not (str.in_re w8 regex)))
(constraint (not (str.in_re w9 regex)))

(check-synth)