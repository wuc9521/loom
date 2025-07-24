; https://smt-lib.org/theories-UnicodeStrings.shtml
(set-logic SLIA) ; regular grammar

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

(synth-fun regex () RegLan
    ((Start RegLan))
    ( 
        (Start RegLan
            ( 
                (str.to_re "a")        ; regex of string "a"
                (str.to_re "b")        ; regex of string "b" 
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

(check-synth)
