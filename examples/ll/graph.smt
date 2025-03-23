(set-logic FOL)
(declare-sort U 0)
(define-signature S
	(declare-fun s () U)   
	(declare-fun t () U)   
	(declare-fun E (U U) Bool)
)

(define-model M 
	:signature S
	(define-universe
		(U {a b c d})         
	)
	(interpret
		(s a)
		(t b)
		(E (match
				((a a) True)
				((b a) True)
				((c a) True)
				((_ _) False)
			)
		)
	)                                           
)

(define-grammar G 
	;; Declare the non-terminals that would be used in the grammar
    ((I Int) (B Bool))
    ;; Define the grammar for allowed implementations.
    ((I Int (x y 0 1
             (+ I I) (- I I)
             (ite B I I)))
     (B Bool ((and B B) (or B B) (not B)
              (= I I) (<= I I) (>= I I))))
)

(synth-form f :signature S :grammar G)
(constraint (models M f)) 
(check-synth)              