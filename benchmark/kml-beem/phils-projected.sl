;; Generated from BEEM family `phils`, label source synthetic modal-K split: balanced separator mined from the shared modal atom vocabulary
;; Found modal-K separator: (modal.box fork_0_is_0)

(set-logic ALL)
(declare-sort U 0)
(declare-fun I () U)
(declare-fun R (U U) Bool)
(declare-fun fork_0_is_0 (U) Bool)

(define-model phils_1
    (define-univ U ($0 $1 $2 $3 $4))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)
        (($0 $4) true)
        ((_ _) false)
    )))
    (interp-fun fork_0_is_0 (match (
        (($0) true)
        (($2) true)
        (($3) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)
(define-model phils_2
    (define-univ U ($0 $1 $2 $3 $4 $5))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)
        (($0 $4) true)
        (($0 $5) true)
        (($1 $0) true)
        (($2 $0) true)
        (($3 $0) true)
        (($4 $0) true)
        (($5 $0) true)
        ((_ _) false)
    )))
    (interp-fun fork_0_is_0 (match (
        (($0) true)
        (($1) true)
        (($2) true)
        (($3) true)
        (($4) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

(declare-term-types
 ((Modal 0))
 (((modal.not Modal)
   (modal.and Modal Modal)
   (modal.or Modal Modal)
   (modal.box Modal)
   (modal.dia Modal)
   (fork_0_is_0))))

(define-finite ModalAsp
  (record (model Model) (node U))
  :where true)

(define-semantics
  ((Modal.Sem (phi Modal) (asp ModalAsp)))
  ((match phi
     ((fork_0_is_0) (fork_0_is_0 (node asp)))
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

(synth-fun phi () Modal)
(constraint (not (Modal.Sem phi (ModalAsp phils_1 $0))))
(constraint (Modal.Sem phi (ModalAsp phils_2 $0)))

(check-synth)
