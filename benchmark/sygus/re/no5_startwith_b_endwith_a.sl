(set-logic SLIA) 
; w begins with a 1 and ends with a 0
; ++
; 10
(define-fun w1 () String
    "ba"
)
; 1X0
(define-fun w2 () String
    "bXa"
)
; 1XX0
(define-fun w3 () String
    "bXXa"
)
; --
; 0
(define-fun w4 () String
    "a"
)
; 1
(define-fun w5 () String
    "b"
)
; 11
(define-fun w6 () String
    "bb"
)
; 1X1
(define-fun w7 () String
    "bXb"
)
; 0X0
(define-fun w8 () String
    "aXa"
)
; 0X1
(define-fun w9 () String
    "aXb"
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
(constraint (not (str.in_re w6 regex)))
(constraint (not (str.in_re w7 regex)))
(constraint (not (str.in_re w8 regex)))
(constraint (not (str.in_re w9 regex)))

(check-synth)