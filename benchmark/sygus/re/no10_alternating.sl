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
(define-fun w6 () String
    "aba"
)
(define-fun w7 () String
    "baba"
)
(define-fun w8 () String
    "abab"
)
(define-fun w9 () String
    "babab"
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
(define-fun wb5 () String
    "baab"
)
(define-fun wb6 () String
    "babbab"
)
(define-fun wb7 () String
    "abaaba"
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
(constraint (str.in_re w6 regex))        
(constraint (str.in_re w7 regex))        
(constraint (str.in_re w8 regex))        
(constraint (str.in_re w9 regex))        
(constraint (not (str.in_re wba regex)))        
(constraint (not (str.in_re wbb regex)))        
(constraint (not (str.in_re wb2 regex)))        
(constraint (not (str.in_re wb3 regex)))        
(constraint (not (str.in_re wb4 regex)))        
(constraint (not (str.in_re wb5 regex)))        
(constraint (not (str.in_re wb6 regex)))        
(constraint (not (str.in_re wb7 regex)))        

(check-synth)