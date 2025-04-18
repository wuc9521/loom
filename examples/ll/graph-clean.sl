(set-logic FOL)
(declare-sort U 0)
(declare-sort V 0)
(declare-fun s () U)   
(declare-fun t () U)   
(declare-fun E (U U) Bool)
(declare-fun F (U U) Bool)
(declare-fun G (U U) Bool)
(declare-fun IET (Bool U U) U)
(declare-qvar qv U)    
(define-model M  
    (define-universe U ($a $b $c $d))
    (interp-fun s ($a))                   ; Simple 0-ary function
    (interp-fun t ($b))                   ; Simple 0-ary function
    (interp-fun E (match (           ; Using match with pattern-value pairs
        (($b $c) true)
        (($a _) true) 
		((_ $b) false)
        ((_ _) false)
    )))
    (interp-fun F (match (
        ((x y) (not (E x y)))
    )))
    (interp-fun G (match (
        ((x y) (or (E x $a) (F $b y)))
    )))
    (interp-fun IET (match (
        ((b x y) (ite b y x))
    )))
)
(synth-fun 
	myform () Bool
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