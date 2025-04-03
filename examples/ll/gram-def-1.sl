; Generate a FOL formula (using signature S)
; If there's no limit, the syntax of a FOL formula looks like this. 
; all these could be automatically generated, given the logic is FOL and ranked signature.

(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		(Start Bool (Formula Bool))
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