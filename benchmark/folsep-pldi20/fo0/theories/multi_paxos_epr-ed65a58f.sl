(provide FoAsp0 Term.0.round Term.0.round.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.round fol.0.atom.active fol.0.atom.le fol.0.atom.one_a fol.0.term.none)
(require "../../common/signatures/multi_paxos_epr-ed65a58f.sl")

(declare-term-types
 ((Formula0 0) (Term.0.round 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.round Term.0.round Term.0.round)
   (fol.0.atom.active Term.0.round)
   (fol.0.atom.le Term.0.round Term.0.round)
   (fol.0.atom.one_a Term.0.round))
  ((fol.0.term.none))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.round.Sem (term Term.0.round) (asp FoAsp0)))
  :returns round
  ((match term
     ((fol.0.term.none) none)
  )))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.eq.round lhs rhs)
      (= (Term.0.round.Sem lhs asp) (Term.0.round.Sem rhs asp)))
     ((fol.0.atom.active a0) (active (Term.0.round.Sem a0 asp)))
     ((fol.0.atom.le a0 a1) (le (Term.0.round.Sem a0 asp) (Term.0.round.Sem a1 asp)))
     ((fol.0.atom.one_a a0) (one_a (Term.0.round.Sem a0 asp)))
  )))
