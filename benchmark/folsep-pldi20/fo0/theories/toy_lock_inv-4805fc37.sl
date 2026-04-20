(provide FoAsp0 Term.0.epoch Term.0.epoch.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.epoch fol.0.atom.le fol.0.term.zero)
(require "../../common/signatures/toy_lock_inv-4805fc37.sl")

(declare-term-types
 ((Formula0 0) (Term.0.epoch 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.epoch Term.0.epoch Term.0.epoch)
   (fol.0.atom.le Term.0.epoch Term.0.epoch))
  ((fol.0.term.zero))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.epoch.Sem (term Term.0.epoch) (asp FoAsp0)))
  :returns epoch
  ((match term
     ((fol.0.term.zero) zero)
  )))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.eq.epoch lhs rhs)
      (= (Term.0.epoch.Sem lhs asp) (Term.0.epoch.Sem rhs asp)))
     ((fol.0.atom.le a0 a1) (le (Term.0.epoch.Sem a0 asp) (Term.0.epoch.Sem a1 asp)))
  )))
