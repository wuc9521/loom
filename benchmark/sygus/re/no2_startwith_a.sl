(set-logic SLIA)
(define-fun wb () String
    "a"
)
(define-fun w2 () String
    "aX"
)
(define-fun w3 () String
    "aXX"
)
(define-fun w4 () String
    "b"
)
(define-fun w5 () String
    "bX"
)
(define-fun w6 () String
    "bXX"
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

(constraint (str.in_re wb regex))        
(constraint (str.in_re w2 regex))        
(constraint (str.in_re w3 regex))        
(constraint (not (str.in_re w4 regex)))        
(constraint (not (str.in_re w5 regex)))        
(constraint (not (str.in_re w6 regex)))             

(check-synth)