(set-logic FOL)
(declare-sort U 0)
(declare-fun s () U)
(declare-fun t () U)
(declare-fun E (U U) Bool)
(declare-qvar x0 U)
(declare-qvar x1 U)

(define-model G1
	(define-univ U ($0 $1 $2 $3 $4 $5 $6 $7))
	(interp-fun s ($7))
	(interp-fun t ($0))
	(interp-fun E (match (
		(($0 $1) true)
		(($0 $2) true)
		(($0 $3) true)
		(($1 $0) true)
		(($1 $4) true)
		(($1 $5) true)
		(($2 $0) true)
		(($3 $0) true)
		(($4 $1) true)
		(($4 $7) true)
		(($5 $1) true)
		(($5 $7) true)
		(($7 $5) true)
		((_ _) false)
	)))
)

(define-model G2
	(define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9))
	(interp-fun s ($8))
	(interp-fun t ($0))
	(interp-fun E (match (
		(($0 $1) true)
		(($0 $2) true)
		(($0 $3) true)
		(($1 $0) true)
		(($1 $4) true)
		(($2 $0) true)
		(($2 $5) true)
		(($2 $7) true)
		(($2 $9) true)
		(($3 $0) true)
		(($3 $6) true)
		(($4 $7) true)
		(($6 $9) true)
		(($7 $2) true)
		(($7 $4) true)
		(($9 $4) true)
		(($9 $6) true)
		((_ _) false)
	)))
)

(define-model G3
(define-univ U ($0 $1 $2 $3 $4 $5 $6 $7))
	(interp-fun s ($7))
	(interp-fun t ($0))
	(interp-fun E (match (
		(($0 $1) true)
		(($0 $2) true)
		(($0 $3) true)
		(($1 $0) true)
		(($1 $4) true)
		(($1 $5) true)
		(($2 $0) true)
		(($3 $0) true)
		(($4 $1) true)
		(($4 $7) true)
		(($5 $1) true)
		(($5 $7) true)
		(($6 $7) true)
		(($7 $5) true)
		(($7 $6) true)
		((_ _) false)
	)))
)

(define-model G4
(define-univ U ($0 $1 $2 $3 $4 $5 $6 $7))
	(interp-fun s ($7))
	(interp-fun t ($0))
	(interp-fun E (match (
		(($0 $1) true)
		(($0 $2) true)
		(($0 $3) true)
		(($1 $0) true)
		(($1 $4) true)
		(($1 $5) true)
		(($2 $0) true)
		(($3 $0) true)
		(($4 $1) true)
		(($4 $7) true)
		(($5 $1) true)
		(($5 $7) true)
		(($6 $7) true)
		(($7 $5) true)
		(($7 $6) true)
		((_ _) false)
	)))
)

(synth-fun phi () Bool
    (
		(Start Bool)
		(Term U)
	)
	(
		(Start Bool
			(
				true 						
				false 			
				(E Term Term) 	
				(= Term Term)
				(fol.not Start)
				(fol.and Start Start)
				(fol.or Start Start)
				(fol.forall ((x0 U)) Start)
				(fol.forall ((x1 U)) Start)
				(fol.exists ((x0 U)) Start)
				(fol.exists ((x1 U)) Start)
			)
		)
		(Term U
			(
				x0
				x1
				s
				t
			)
		)
	)
)

(constraint (models G1 phi))
(constraint (models G2 phi))
(constraint (not (models G3 phi)))
(constraint (not (models G4 phi)))

(check-synth)