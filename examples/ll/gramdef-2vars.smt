(show-grammar S)


(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		:override (UVar)
		(UVar U (x1 x2))
	)
)
