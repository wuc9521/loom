(define-signature S
	(declare-fun s () U)   
	(declare-fun t () U)   
	(declare-fun E (U U) Bool)
	(declare-fun F (U U) Bool)
	(declare-fun G (U U) Bool)
	(declare-fun IET (Bool U U) U)
)


(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		(Start Bool (Formula))
        (Formula Bool
			(
                Atom
				(and Formula Formula)			; provided by FOL
				(or Formula Formula)  			; provided by FOL
				(not Formula)       			; provided by FOL
				(forall (Variable U) Formula)  	; provided by FOL
				(exists (Variable U) Formula)  	; provided by FOL
			)
		) 
        (Atom Bool
            (
                true 						    ; provided by FOL
				false       			        ; provided by FOL
                (E Term Term)           	    ; provided by FOL
                (F Term Term)                   ; provided by FOL
                (G Term Term)                   ; provided by FOL
            )
        )  
		(Term U
			(
				(Variable U)                    ; provided by FOL
				(Constant U)                    ; provided by FOL
				(IET Formula Term Term)         ; provided by FOL
			)
		)
	)
)