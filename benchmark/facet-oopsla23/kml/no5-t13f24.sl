(require "../theories/main.sl")
(set-logic ALL) ;; propositional modal logic

;; in PML, define-model requires users to define

;; - a finite set of states S.
;; - a set of initial states I ⊆ S.
;; - a transition relation R ⊆ S × S such that R is left-total, i.e., ∀s ∈ S ∃s' ∈ S such that (s,s') ∈ R.
;; - a labeling (or interpretation) function L: S → 2AP.

(synth-fun phi () Modal)

(constraint (Modal.Sem phi (ModalAsp K1 $0)))
(constraint (not (Modal.Sem phi (ModalAsp K2 $0))))
(constraint (Modal.Sem phi (ModalAsp K3 $0)))
(constraint (not (Modal.Sem phi (ModalAsp K4 $0))))

(check-synth)
