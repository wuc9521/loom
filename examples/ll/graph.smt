(set-logic FOL)
; TODO: 我们会允许用户自己定义logic吗? 还是只能用已有的logic

(declare-sort U 0) ; decalre sort
(declare-sort V 0) ; 

; TODO: 是否需要 define-signature 这样的东西? 看起来只会用到一个 signature ?
; (define-signature S
(declare-fun s () U)   
(declare-fun t () U)   
(declare-fun E (U U) Bool)
(declare-fun F (U U) Bool)
(declare-fun G (U U) Bool)
(declare-fun IET (Bool U U) U)
(declare-qvar qv U)    ; TODO: why even qvar? Why cannot just var?
; )

; (show-grammar S)

; operations allowed in interpretations.
;	1. ``and'', 
;   2. ``or'', 
;   3. ``not'', 
;   4. ``ite'' /// ????


(define-model M  
	; :signature S
	(define-universe U ($a $b $c $d))   ; everything in the universe starts with `$` to distinguish from normal symbols
	;? universe seems a bit strange    
	; (define-universe (V ($c $d)))     ; allow elements of same name in U, because we can do type inference.
	; (interpret S					   ;? if we can ignore define-signature, it will look much more neat.
	(interp-fun s (($a)))              ; symbol name and an (arity+1)- tuple
	(interp-fun t ($b))                ; a shortcut: remove the inner parenthensis
	(interp-fun E  					  ;; check if pattern matching is exhaustive at compile time
		(					
			($b $c true)    ; if 
			($a _ true)		; else if
			(_ b false)		; else if
			(_ _ false) 	; else
		)
	)
	(interp-fun F  
		(
			(x y (not (E x y)))  ;; $x and $y are variables
		)
	)
	(interp-fun G 	
		(
			(x y (or (E x $a) (F $b y)))  ;; Variables and elements clearly distinguished
		)
	)
	(interp-fun IET (
			(b x y (ite b y x))
		)
	)
)

(synth-fun 
	; :signature S   ; use the signature S 
	myform () Bool ; A boolean nullary function == A logic sentence.
	; I want a first order sentence of the pattern: exists <v>. and a conjunction of literals
	(
		(Start Bool (
			(exists qv Conjunction)
		))
		(Conjunction Bool (
			Literal
			(and Conjunction Literal)
		))
		(Literal Bool (
			Atom
			(not Atom)
		))
        (Atom Bool
			(
				true 						    
				false       			        
				(E Term Term)           	    
				(F Term Term)
				(G Term Term)
				(= Term Term)
			)
		)   
		(Term U
			(
				qv  
				s 
				t                            
				(ITE Literal Term Term)         
			)
		)
	)
)

(constraint (models M myform)) 
(check-synth)              

