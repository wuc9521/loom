(set-logic FOL)
(declare-sort U 0)
(define-signature S
	(declare-fun s () U)   
	(declare-fun t () U)   
	(declare-fun E (U U) Bool)
	(declare-fun ITE (Bool U U) U)
)

(define-model M 
	:signature S
	(define-universe
		(U {a b c d})         
	)
	(interpret
		(s (() a))
		(t (() b))
		(E (match
				((b c) true)
				((a _) true)
				((_ b) true)
				((_ _) false)
			)
		)
		(ITE (match
				((true a _) a)
				((true b _) b)
				((true c _) c)
				((true d _) d)
				((false _ a) a)
				((false _ b) b)
				((false _ c) c)
				((false _ d) d)
			) 
		)
	)                                           
)

(synth-fun 
	:signature S  ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		(Start Bool 
			(true false       			    ; provided by FOL
				(and Start Start)			; provided by FOL
				(or Start Start)  			; provided by FOL
				(not Start)       			; provided by FOL
				(forall Var Start)  			; provided by FOL
				(exists Var Start)  			; provided by FOL
				(E Var Var)           			; provided by FOL
			)
		)   
		(Var U)
		(Term U
			(Var
				(ITE Start Term Term)     ; provided by FOL
			)
		)
	)
)
(constraint (models M myform))
(check-synth)              