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

; Exactly the same as SyGuS, except 2 things:
; 1. Should specify signature. 
;     - by default, user don't have to specify trivial grammar induced by the signature. eg. Relation(Term,.., Term) :: Bool. 
; 	  - we also allow user to ``restrict'' the usage of certain grammars.
;     - Question: Should we allow user define grammars that do not exist in the signa
; 2. 
(synth-fun 
	:signature S  ; use the signature S 
	form () Bool ; A boolean nullary function == A logic sentence.
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
; TODO: 如何将要合成的函数的参数和grammar中specify地参数对应起来. 
; TODO: 尤其是有量词问题. 

(constraint (models M form)) 
(synth-fun 
	:signature S 
	func (x U) U 
	(
		(U (s t x () )
	)
)
(check-synth)              