(set-logic SLIA) 

; w contains the substring 0101
; ++
; 0101
(define-fun w1 () String
    "abab"
)
; X0101X
(define-fun w2 () String
    "XababX"
)
; XX0101XX
(define-fun w3 () String
    "XXababXX"
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
; XX
(define-fun w6 () String
    "XX"
)
; XXX
(define-fun w7 () String
    "XXX"
)
; XXX0
(define-fun w8 () String
    "XXXa"
)
; X0XX
(define-fun w9 () String
    "XbXX"
)
; 1XXX
(define-fun w10 () String
    "bXXX"
)
; XX1X
(define-fun w11 () String
    "XXbX"
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
(constraint (not (str.in_re w10 regex)))
(constraint (not (str.in_re w11 regex)))

(check-synth)