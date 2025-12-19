(set-logic SLIA)
(define-fun wb () String
    "a"
)
(define-fun w2 () String
    "b"
)
(define-fun w3 () String
    "ab"
)
(define-fun w4 () String
    "ba"
)
(define-fun w5 () String
    "bab"
)
(define-fun wba () String
    "aa"
)
(define-fun wbb () String
    "bb"
)
(define-fun wb2 () String
    "abb"
)
(define-fun wb3 () String
    "aab"
)
(define-fun wb4 () String
    "bba"
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

(constraint (str.in_re wb regex))        
(constraint (str.in_re w2 regex))        
(constraint (str.in_re w3 regex))        
(constraint (str.in_re w4 regex))        
(constraint (str.in_re w5 regex))               
(constraint (not (str.in_re wba regex)))        
(constraint (not (str.in_re wbb regex)))        
(constraint (not (str.in_re wb2 regex)))        
(constraint (not (str.in_re wb3 regex)))        
(constraint (not (str.in_re wb4 regex)))             

(check-synth)