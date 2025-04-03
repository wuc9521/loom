(set-logic FOL)

(declare-sort U 0) ; decalre sort

(define-signature S
	(declare-fun s () U)   
	(declare-fun t () U)   
	(declare-fun E (U U) Bool)
	(declare-fun F (U U) Bool)
	(declare-fun G (U U) Bool)
	(declare-fun IET (Bool U U) U)
	(declare-qvar x1 U)
)

; (show-grammar S)

; operations allowed in interpretations.
;	1. ``and'', 
;   2. ``or'', 
;   3. ``not'', 
;   4. ``ite'' /// ????

(define-model M  
	:signature S
	(define-universe
		(U {a b c d})       ;;? universe seems a bit strange    
	)
	(interpret S
		(s (a))                 ;; symbol name and an (arity+1)- tuple
		(t (b))
		(E  					;; check if pattern matching is exhaustive at compile time
			(					
				(b c true)    	; if 
				(a $_ true)		; else if
				($_ b false)	; else if
				($_ $_ false) 	; else
			)
		)
		(F  
			(
				($x $y (not (E $x $y)))  ;; $x and $y are variables
			)
		)
		(G 	
			(
				($x $y (or (E $x a) (F b $y)))  ;; Variables and elements clearly distinguished
			)
		)
		(IET ; the complement of ITE 
			(
				(true $_ a a)
				(true $_ b b)
				(true $_ c c)
				(true $_ d d)
				(false a $_ a)
				(false b $_ b)
				(false c $_ c)
				(false d $_ d)
			) 
		)
		; (IET 
		; 	(
		; 		(($b $x $y) (ite $b $y $x))
		; 	) 
		; )   
	)         
)

; You can specify grammars explicitly, or you can extend the signature. 
; (define-grammar G 
; 	;; Declare the non-terminals that would be used in the grammar
;     ((B Bool))
;     ;; Define the grammar for allowed implementations.
;     (
; 		(U (x (s) (t) (ite B U U))
; 		(B ((= U U) (and B B) (or B B) (not B) (forall x B) (exists x B) (E U U)))
; 	)
; )

(synth-fun 
	:signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	(
		:override (UVar)
		(UVar U (x1 x2))
	)
)

(constraint (models M form)) 
(check-synth)              
