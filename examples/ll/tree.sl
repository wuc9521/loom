(set-logic FOL)
(declare-sort U 0) ; universe 
(declare-sort N 0) ; nil
(define-signature S
	(declare-fun root () U)   		; constant
	(declare-fun nil () N)   		; constant
	(declare-fun hasNext () Bool)   ; predicate
	(declare-fun prec (U) U) 		; function
)

(define-model M 
	:signature S
	(define-universe
		(U {a b c})     
		(N {nil})    
	)
	(interpret
		(root (() a))
		(prec 
			((b) a)
			((c) a)
		)
		(hasNext 
			(match
				((a) true)
				((_) false)
			)
		)
	)                                           
)

(define-grammar G 
	;; Declare the non-terminals that would be used in the grammar
    ((B Bool))
    ;; Define the grammar for allowed implementations.
    ; TODO
)

(synth-form f :signature S :grammar G)
(constraint (models M f)) 
(check-synth)              