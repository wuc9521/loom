(set-logic SLIA)

; Converted from benchmark/re/alpharegex semantic-format benchmarks.
; Source "_" positions are emitted as the string character "_";
; the grammar omits a literal "_" terminal, so only re.allchar can match them.
; Source "?" positions are expanded over both "0" and "1" concrete strings.

(define-fun w1 () String
    "0101"
)
(define-fun w2 () String
    "_0101_"
)
(define-fun w3 () String
    "__0101__"
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
    "000"
)
(define-fun w11 () String
    "001"
)
(define-fun w12 () String
    "010"
)
(define-fun w13 () String
    "011"
)
(define-fun w14 () String
    "100"
)
(define-fun w15 () String
    "101"
)
(define-fun w16 () String
    "110"
)
(define-fun w17 () String
    "111"
)
(define-fun w18 () String
    "0000"
)
(define-fun w19 () String
    "0010"
)
(define-fun w20 () String
    "0100"
)
(define-fun w21 () String
    "0110"
)
(define-fun w22 () String
    "1000"
)
(define-fun w23 () String
    "1010"
)
(define-fun w24 () String
    "1100"
)
(define-fun w25 () String
    "1110"
)
(define-fun w26 () String
    "0000"
)
(define-fun w27 () String
    "0001"
)
(define-fun w28 () String
    "0010"
)
(define-fun w29 () String
    "0011"
)
(define-fun w30 () String
    "1000"
)
(define-fun w31 () String
    "1001"
)
(define-fun w32 () String
    "1010"
)
(define-fun w33 () String
    "1011"
)
(define-fun w34 () String
    "1000"
)
(define-fun w35 () String
    "1001"
)
(define-fun w36 () String
    "1010"
)
(define-fun w37 () String
    "1011"
)
(define-fun w38 () String
    "1100"
)
(define-fun w39 () String
    "1101"
)
(define-fun w40 () String
    "1110"
)
(define-fun w41 () String
    "1111"
)
(define-fun w42 () String
    "0010"
)
(define-fun w43 () String
    "0011"
)
(define-fun w44 () String
    "0110"
)
(define-fun w45 () String
    "0111"
)
(define-fun w46 () String
    "1010"
)
(define-fun w47 () String
    "1011"
)
(define-fun w48 () String
    "1110"
)
(define-fun w49 () String
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
                (re.? Start)         ; optional
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
(constraint (not (str.in_re w22 regex)))
(constraint (not (str.in_re w23 regex)))
(constraint (not (str.in_re w24 regex)))
(constraint (not (str.in_re w25 regex)))
(constraint (not (str.in_re w26 regex)))
(constraint (not (str.in_re w27 regex)))
(constraint (not (str.in_re w28 regex)))
(constraint (not (str.in_re w29 regex)))
(constraint (not (str.in_re w30 regex)))
(constraint (not (str.in_re w31 regex)))
(constraint (not (str.in_re w32 regex)))
(constraint (not (str.in_re w33 regex)))
(constraint (not (str.in_re w34 regex)))
(constraint (not (str.in_re w35 regex)))
(constraint (not (str.in_re w36 regex)))
(constraint (not (str.in_re w37 regex)))
(constraint (not (str.in_re w38 regex)))
(constraint (not (str.in_re w39 regex)))
(constraint (not (str.in_re w40 regex)))
(constraint (not (str.in_re w41 regex)))
(constraint (not (str.in_re w42 regex)))
(constraint (not (str.in_re w43 regex)))
(constraint (not (str.in_re w44 regex)))
(constraint (not (str.in_re w45 regex)))
(constraint (not (str.in_re w46 regex)))
(constraint (not (str.in_re w47 regex)))
(constraint (not (str.in_re w48 regex)))
(constraint (not (str.in_re w49 regex)))

(check-synth)
