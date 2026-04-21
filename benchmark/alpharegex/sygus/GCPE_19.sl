(set-logic SLIA)

; Converted from benchmark/re/alpharegex semantic-format benchmarks.
; Source "_" positions are emitted as the string character "_";
; the grammar omits a literal "_" terminal, so only re.allchar can match them.
; Source "?" positions are expanded over both "0" and "1" concrete strings.

(define-fun w1 () String
    "_"
)
(define-fun w2 () String
    "__"
)
(define-fun w3 () String
    "0__"
)
(define-fun w4 () String
    "_1_"
)
(define-fun w5 () String
    "__1"
)
(define-fun w6 () String
    "_101"
)
(define-fun w7 () String
    "0_01_"
)
(define-fun w8 () String
    "__1_1_1"
)
(define-fun w9 () String
    "100"
)
(define-fun w10 () String
    "0100"
)
(define-fun w11 () String
    "1100"
)
(define-fun w12 () String
    "1000"
)
(define-fun w13 () String
    "1001"
)
(define-fun w14 () String
    "01000"
)
(define-fun w15 () String
    "01001"
)
(define-fun w16 () String
    "11000"
)
(define-fun w17 () String
    "11001"
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
(constraint (str.in_re w4 regex))
(constraint (str.in_re w5 regex))
(constraint (str.in_re w6 regex))
(constraint (str.in_re w7 regex))
(constraint (str.in_re w8 regex))
(constraint (not (str.in_re w9 regex)))
(constraint (not (str.in_re w10 regex)))
(constraint (not (str.in_re w11 regex)))
(constraint (not (str.in_re w12 regex)))
(constraint (not (str.in_re w13 regex)))
(constraint (not (str.in_re w14 regex)))
(constraint (not (str.in_re w15 regex)))
(constraint (not (str.in_re w16 regex)))
(constraint (not (str.in_re w17 regex)))

(check-synth)
