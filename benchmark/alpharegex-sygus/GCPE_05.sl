(set-logic SLIA)

; Converted from benchmark/re/alpharegex semantic-format benchmarks.
; Source "_" positions are emitted as the string character "_";
; the grammar omits a literal "_" terminal, so only re.allchar can match them.
; Source "?" positions are expanded over both "0" and "1" concrete strings.

(define-fun w1 () String
    "__0"
)
(define-fun w2 () String
    "__0_"
)
(define-fun w3 () String
    "__0__"
)
(define-fun w4 () String
    "0"
)
(define-fun w5 () String
    "1"
)
(define-fun w6 () String
    "00"
)
(define-fun w7 () String
    "01"
)
(define-fun w8 () String
    "10"
)
(define-fun w9 () String
    "11"
)
(define-fun w10 () String
    "001"
)
(define-fun w11 () String
    "011"
)
(define-fun w12 () String
    "101"
)
(define-fun w13 () String
    "111"
)
(define-fun w14 () String
    "0010"
)
(define-fun w15 () String
    "0011"
)
(define-fun w16 () String
    "0110"
)
(define-fun w17 () String
    "0111"
)
(define-fun w18 () String
    "1010"
)
(define-fun w19 () String
    "1011"
)
(define-fun w20 () String
    "1110"
)
(define-fun w21 () String
    "1111"
)

(synth-fun regex () RegLan
    ((Start RegLan))
    (
        (Start RegLan
            (
                (str.to_re "0")        ; regex of string "0"
                (str.to_re "1")        ; regex of string "1"
                re.allchar             ; any single character, including "_"
                (re.opt Start)         ; optional
                (re.++ Start Start)    ; concat
                (re.union Start Start) ; union
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
(constraint (not (str.in_re w12 regex)))
(constraint (not (str.in_re w13 regex)))
(constraint (not (str.in_re w14 regex)))
(constraint (not (str.in_re w15 regex)))
(constraint (not (str.in_re w16 regex)))
(constraint (not (str.in_re w17 regex)))
(constraint (not (str.in_re w18 regex)))
(constraint (not (str.in_re w19 regex)))
(constraint (not (str.in_re w20 regex)))
(constraint (not (str.in_re w21 regex)))

(check-synth)
