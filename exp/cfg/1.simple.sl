; This file is under consideration.

(set-logic RE) ; regular grammar

; define signatures here.
(declare-char 'a')
(declare-char 'b')
(define-string w1 (a b))
(define-string w2 (a b a b))
(define-string w3 (a b a b a b))
(define-string w4 (b a a))
(define-string w5 (a a b))

(synth-fun regex () Regex
    (Start Regex)
    ( 
        (Start Regex
            ( 
                re.empty               ; non-recursive, const
                re.none                ; ∅
                re.a                   ; 单字符 'a', const
                re.b                   ; 单字符 'b', const
                (Concat Start Start)
                (Or Start Start)
                (And Start Start)
                (Optional Start)
                (KleeneStar Start)
                (Repeat Start Num)
                (RepeatAtLeast Start Num)
                (RepeatRange Start Num Num)
            )
        )
        (Num Int 
            (
                0 1 2 3
            )
        )
    )
)


(constraint (models w1 regex))        
(constraint (models w2 regex))        
(constraint (models w3 regex))        
(constraint (not (models w4 regex)))        
(constraint (not (models w5 regex)))        

(check-synth)
