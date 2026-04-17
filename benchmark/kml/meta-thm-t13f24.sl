(set-logic ALL) ;; propositional modal logic
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
)


(declare-term-types
 ((Modal 0))
 (((a) (c) (v)
   (modal.not Modal)
   (modal.and Modal Modal)
   (modal.or Modal Modal)
   (modal.box Modal)
   (modal.dia Modal))))


(define-finite ModalAsp
  (record (model Model) (node U))
  :where true)

(define-semantics
  ((Modal.Sem (phi Modal) (asp ModalAsp)))
  ((match phi
     ((a) (a (node asp)))
     ((c) (c (node asp)))
     ((v) (v (node asp)))
     ((modal.not x) (not (Modal.Sem x asp)))
     ((modal.and x y) (and (Modal.Sem x asp) (Modal.Sem y asp)))
     ((modal.or x y) (or (Modal.Sem x asp) (Modal.Sem y asp)))
     ((modal.box x)
      (forall ((next (domain (U (model asp)))))
        (or (not (R (node asp) next))
            (Modal.Sem x (ModalAsp (model asp) next)))))
     ((modal.dia x)
      (exists ((next (domain (U (model asp)))))
        (and (R (node asp) next)
             (Modal.Sem x (ModalAsp (model asp) next))))))))

(synth-fun phi () Modal
    ((Start Modal))
	(
		(Start Modal
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

(constraint (Modal.Sem phi (ModalAsp K1 $0)))
(constraint (not (Modal.Sem phi (ModalAsp K2 $0))))
(constraint (Modal.Sem phi (ModalAsp K3 $0)))
(constraint (not (Modal.Sem phi (ModalAsp K4 $0))))

(check-synth)
