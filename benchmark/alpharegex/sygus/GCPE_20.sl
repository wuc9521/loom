(set-logic SLIA)

; Converted from benchmark/re/alpharegex semantic-format benchmarks.
; Source "_" positions are emitted as the string character "_";
; the grammar omits a literal "_" terminal, so only re.allchar can match them.
; Source "?" positions are expanded over both "0" and "1" concrete strings.

(define-fun w1 () String
    "0"
)
(define-fun w2 () String
    "0_"
)
(define-fun w3 () String
    "0_0"
)
(define-fun w4 () String
    "0_0_"
)
(define-fun w5 () String
    "0_0_0"
)
(define-fun w6 () String
    "0_0_0_"
)
(define-fun w7 () String
    "1"
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
    "1000"
)
(define-fun w15 () String
    "1001"
)
(define-fun w16 () String
    "1010"
)
(define-fun w17 () String
    "1011"
)
(define-fun w18 () String
    "1100"
)
(define-fun w19 () String
    "1101"
)
(define-fun w20 () String
    "1110"
)
(define-fun w21 () String
    "1111"
)
(define-fun w22 () String
    "0010"
)
(define-fun w23 () String
    "0011"
)
(define-fun w24 () String
    "0110"
)
(define-fun w25 () String
    "0111"
)
(define-fun w26 () String
    "1010"
)
(define-fun w27 () String
    "1011"
)
(define-fun w28 () String
    "1110"
)
(define-fun w29 () String
    "1111"
)
(define-fun w30 () String
    "10000"
)
(define-fun w31 () String
    "10001"
)
(define-fun w32 () String
    "10010"
)
(define-fun w33 () String
    "10011"
)
(define-fun w34 () String
    "10100"
)
(define-fun w35 () String
    "10101"
)
(define-fun w36 () String
    "10110"
)
(define-fun w37 () String
    "10111"
)
(define-fun w38 () String
    "11000"
)
(define-fun w39 () String
    "11001"
)
(define-fun w40 () String
    "11010"
)
(define-fun w41 () String
    "11011"
)
(define-fun w42 () String
    "11100"
)
(define-fun w43 () String
    "11101"
)
(define-fun w44 () String
    "11110"
)
(define-fun w45 () String
    "11111"
)
(define-fun w46 () String
    "00100"
)
(define-fun w47 () String
    "00101"
)
(define-fun w48 () String
    "00110"
)
(define-fun w49 () String
    "00111"
)
(define-fun w50 () String
    "01100"
)
(define-fun w51 () String
    "01101"
)
(define-fun w52 () String
    "01110"
)
(define-fun w53 () String
    "01111"
)
(define-fun w54 () String
    "10100"
)
(define-fun w55 () String
    "10101"
)
(define-fun w56 () String
    "10110"
)
(define-fun w57 () String
    "10111"
)
(define-fun w58 () String
    "11100"
)
(define-fun w59 () String
    "11101"
)
(define-fun w60 () String
    "11110"
)
(define-fun w61 () String
    "11111"
)
(define-fun w62 () String
    "00001"
)
(define-fun w63 () String
    "00011"
)
(define-fun w64 () String
    "00101"
)
(define-fun w65 () String
    "00111"
)
(define-fun w66 () String
    "01001"
)
(define-fun w67 () String
    "01011"
)
(define-fun w68 () String
    "01101"
)
(define-fun w69 () String
    "01111"
)
(define-fun w70 () String
    "10001"
)
(define-fun w71 () String
    "10011"
)
(define-fun w72 () String
    "10101"
)
(define-fun w73 () String
    "10111"
)
(define-fun w74 () String
    "11001"
)
(define-fun w75 () String
    "11011"
)
(define-fun w76 () String
    "11101"
)
(define-fun w77 () String
    "11111"
)
(define-fun w78 () String
    "000010"
)
(define-fun w79 () String
    "000011"
)
(define-fun w80 () String
    "000110"
)
(define-fun w81 () String
    "000111"
)
(define-fun w82 () String
    "001010"
)
(define-fun w83 () String
    "001011"
)
(define-fun w84 () String
    "001110"
)
(define-fun w85 () String
    "001111"
)
(define-fun w86 () String
    "010010"
)
(define-fun w87 () String
    "010011"
)
(define-fun w88 () String
    "010110"
)
(define-fun w89 () String
    "010111"
)
(define-fun w90 () String
    "011010"
)
(define-fun w91 () String
    "011011"
)
(define-fun w92 () String
    "011110"
)
(define-fun w93 () String
    "011111"
)
(define-fun w94 () String
    "100010"
)
(define-fun w95 () String
    "100011"
)
(define-fun w96 () String
    "100110"
)
(define-fun w97 () String
    "100111"
)
(define-fun w98 () String
    "101010"
)
(define-fun w99 () String
    "101011"
)
(define-fun w100 () String
    "101110"
)
(define-fun w101 () String
    "101111"
)
(define-fun w102 () String
    "110010"
)
(define-fun w103 () String
    "110011"
)
(define-fun w104 () String
    "110110"
)
(define-fun w105 () String
    "110111"
)
(define-fun w106 () String
    "111010"
)
(define-fun w107 () String
    "111011"
)
(define-fun w108 () String
    "111110"
)
(define-fun w109 () String
    "111111"
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
(constraint (not (str.in_re w50 regex)))
(constraint (not (str.in_re w51 regex)))
(constraint (not (str.in_re w52 regex)))
(constraint (not (str.in_re w53 regex)))
(constraint (not (str.in_re w54 regex)))
(constraint (not (str.in_re w55 regex)))
(constraint (not (str.in_re w56 regex)))
(constraint (not (str.in_re w57 regex)))
(constraint (not (str.in_re w58 regex)))
(constraint (not (str.in_re w59 regex)))
(constraint (not (str.in_re w60 regex)))
(constraint (not (str.in_re w61 regex)))
(constraint (not (str.in_re w62 regex)))
(constraint (not (str.in_re w63 regex)))
(constraint (not (str.in_re w64 regex)))
(constraint (not (str.in_re w65 regex)))
(constraint (not (str.in_re w66 regex)))
(constraint (not (str.in_re w67 regex)))
(constraint (not (str.in_re w68 regex)))
(constraint (not (str.in_re w69 regex)))
(constraint (not (str.in_re w70 regex)))
(constraint (not (str.in_re w71 regex)))
(constraint (not (str.in_re w72 regex)))
(constraint (not (str.in_re w73 regex)))
(constraint (not (str.in_re w74 regex)))
(constraint (not (str.in_re w75 regex)))
(constraint (not (str.in_re w76 regex)))
(constraint (not (str.in_re w77 regex)))
(constraint (not (str.in_re w78 regex)))
(constraint (not (str.in_re w79 regex)))
(constraint (not (str.in_re w80 regex)))
(constraint (not (str.in_re w81 regex)))
(constraint (not (str.in_re w82 regex)))
(constraint (not (str.in_re w83 regex)))
(constraint (not (str.in_re w84 regex)))
(constraint (not (str.in_re w85 regex)))
(constraint (not (str.in_re w86 regex)))
(constraint (not (str.in_re w87 regex)))
(constraint (not (str.in_re w88 regex)))
(constraint (not (str.in_re w89 regex)))
(constraint (not (str.in_re w90 regex)))
(constraint (not (str.in_re w91 regex)))
(constraint (not (str.in_re w92 regex)))
(constraint (not (str.in_re w93 regex)))
(constraint (not (str.in_re w94 regex)))
(constraint (not (str.in_re w95 regex)))
(constraint (not (str.in_re w96 regex)))
(constraint (not (str.in_re w97 regex)))
(constraint (not (str.in_re w98 regex)))
(constraint (not (str.in_re w99 regex)))
(constraint (not (str.in_re w100 regex)))
(constraint (not (str.in_re w101 regex)))
(constraint (not (str.in_re w102 regex)))
(constraint (not (str.in_re w103 regex)))
(constraint (not (str.in_re w104 regex)))
(constraint (not (str.in_re w105 regex)))
(constraint (not (str.in_re w106 regex)))
(constraint (not (str.in_re w107 regex)))
(constraint (not (str.in_re w108 regex)))
(constraint (not (str.in_re w109 regex)))

(check-synth)
