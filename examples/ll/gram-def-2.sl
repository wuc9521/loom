(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		:grammar-mode replace ; or merge
		(Start Bool (Formula))
        (Formula Bool
			(
				true 						    ; provided by FOL
				false       			        ; provided by FOL
				(and Formula Formula)			; provided by FOL
				(or Formula Formula)  			; provided by FOL
				(not Formula)       			; provided by FOL
				(forall (Variable U) Formula)  	; provided by FOL
				(exists (Variable U) Formula)  	; provided by FOL
				(E Term Term)           	    ; provided by FOL
			)
		)   
		(Term U
			(
				(Variable U)                    ; provided by FOL
				(Constant U)                    ; provided by FOL
				(ITE Formula Term Term)         ; provided by FOL
			)
		)
	)
)