(set-logic PML) ;; propositional modal logic
(declare-sort U 0)
(declare-fun I () U)
(declare-fun R (U U) Bool)
(declare-fun a (U) Bool)
(declare-fun c (U) Bool)
(declare-fun v (U) Bool)

;; in PML, define-model requires users to define

;; - a finite set of states S.
;; - a set of initial states I ⊆ S.
;; - a transition relation R ⊆ S × S such that R is left-total, i.e., ∀s ∈ S ∃s' ∈ S such that (s,s') ∈ R.
;; - a labeling (or interpretation) function L: S → 2AP.

(define-model K1 ;; Kripke Structure 1
    (define-univ U ($0 $1 $2 $3))
    (interp-fun R (match (; reachability relation.
        (($0 $2) true)
		(($0 $3) true)
		(($1 $0) true)
		(($2 $3) true)
		(($3 $1) true)
		(($3 $0) true)
		((_ _) false)
    )))
	(interp-fun a (match (
        (($0) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($2) true)
        ((_) false)
    )))
    (interp-fun v (match (
        (($3) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model K2 ;; Kripke Structure 2
    (define-univ U ($0 $1 $2 $3 $4 $5 $6))
    (interp-fun I ($0))
    (interp-fun R (match (; reachability relation.
		(($0 $1) true)
		(($0 $2) true)
        (($1 $3) true)
		(($1 $4) true)
		(($2 $5) true)
		(($2 $6) true)
		((_ _) false)
    )))
	(interp-fun a (match (
        (($5) true)
        ((_) false)
    )))
	(interp-fun c (match (
        (($3) false)
        (($5) false)
        ((_) true)
    )))
    (interp-fun v (match (
        (($3) true)
        ((_) false)
    )))
    (set-init I)
    (set-tran R)
)

(define-model K3 ;; Kripke Structure 3
    (define-univ U ($0 $1))
    (interp-fun I ($0))
    (interp-fun R  (match (; reachability relation.
		(($0 $1) true)
		((_ _) false)
    )))
	(interp-fun a (match (
        (($0) true)
        ((_) false)
    )))
	(interp-fun c (match (
        ((_) false)
    )))
    (interp-fun v (match (
        (($1) true)
        ((_) false)
    )))
    (set-init I)
    (set-tran R)
)

(define-model K4 ;; Kripke Structure 4
    (define-univ U ($0 $1 $2))
    (interp-fun I ($0))
    (interp-fun R (match (; reachability relation.
    	(($0 $2) true)
		(($1 $0) true)
		(($2 $1) true)
		((_ _) false)
    )))
	(interp-fun a (match (
        (($2) true)
        ((_) false)
    )))
	(interp-fun c (match (
        (($1) true)
        ((_) false)
    )))
	(interp-fun v (match (
        (($0) true)
        ((_) false)
    )))
    (set-init I)
    (set-tran R)
)

(synth-fun phi () Bool
    ((Start Bool))
	(
		(Start Bool
			(
				a
                c
                v		
				(modal.not Start)
				(modal.and Start Start)
				(modal.or Start Start)
				(modal.box Start)
				(modal.dia Start)
			)
		)
	)
)

(constraint (models K1 phi))
(constraint (not (models K1 phi)))

(check-synth)