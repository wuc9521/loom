(set-logic FOL)

(declare-sort U 0) ; decalre sort
(declare-sort V 0) 

(define-signature S
	(declare-fun s () U)   
	(declare-fun t () U)   
	(declare-fun E (U U) Bool)
	(declare-fun F (U U) Bool)
	(declare-fun G (U U) Bool)
	(declare-fun IET (Bool V V) V)
)

(show-grammar S)

(define-model M  
	(define-universe
		(U () {a b c d})    
		(V () {a b c})     
	)
	(interpret S
		(s (() a))
		(t (() b))
		(E  					;; check if pattern matching is exhaustive at compile time
			(					
				((b c) true)    ; if 
				((a $_) true)	; else if
				(($_ b) false)	; else if
				(($_ $_) false) ; else
			)
		)
		(F  
			(
				(($x $y) (not (E $x $y)))  ;; $x and $y are variables
			)
		)
		(G 	
			(
				(($x $y) (or (E $x a) (F b $y)))  ;; Variables and elements clearly distinguished
			)
		)
		(IET ; the complement of ITE 
			(
				((true $_ a) a)
				((true $_ b) b)
				((true $_ c) c)
				((false a $_) a)
				((false b $_) b)
				((false c $_) c)
			) 
		)
		; (ITE 
		; 	(
		; 		(($b $x $y) (not (ite $b $x $y)))
		; 	) 
		; )   
	)         
)

; synthesis a first order sentense using signature S. 
; no restriction at all.
(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	()
)

(constraint (= (M myform) true)) ; myform in true in model M
(synth-fun 
	:signature S 
	:override
	myfunc (x U) U 
	(
		(U (s t x () ))
	)
)
(constraint 
	(= ((M myfunc) a) a) ; myfunc(a) = a in model M. 
)
(check-synth)              
