(provide FoAsp0 Term.0.quorum Term.0.quorum.Sem Term.0.round Term.0.round.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.quorum fol.0.eq.round fol.0.atom.irrelevant_msg fol.0.atom.le fol.0.atom.one_a fol.0.term.quorum_of_round fol.0.term.master_minrel fol.0.term.none)
(require "../../common/signatures/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")

(declare-term-types
 ((Formula0 0) (Term.0.quorum 0) (Term.0.round 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.quorum Term.0.quorum Term.0.quorum)
   (fol.0.eq.round Term.0.round Term.0.round)
   (fol.0.atom.irrelevant_msg Term.0.round)
   (fol.0.atom.le Term.0.round Term.0.round)
   (fol.0.atom.one_a Term.0.round Term.0.round))
  ((fol.0.term.quorum_of_round Term.0.round))
  ((fol.0.term.master_minrel)
   (fol.0.term.none))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.quorum.Sem (term Term.0.quorum) (asp FoAsp0)))
  :returns quorum
  ((match term
     ((fol.0.term.quorum_of_round a0) (quorum_of_round (Term.0.round.Sem a0 asp)))
  )))

(define-semantics
  ((Term.0.round.Sem (term Term.0.round) (asp FoAsp0)))
  :returns round
  ((match term
     ((fol.0.term.master_minrel) master_minrel)
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
     ((fol.0.eq.quorum lhs rhs)
      (= (Term.0.quorum.Sem lhs asp) (Term.0.quorum.Sem rhs asp)))
     ((fol.0.eq.round lhs rhs)
      (= (Term.0.round.Sem lhs asp) (Term.0.round.Sem rhs asp)))
     ((fol.0.atom.irrelevant_msg a0) (irrelevant_msg (Term.0.round.Sem a0 asp)))
     ((fol.0.atom.le a0 a1) (le (Term.0.round.Sem a0 asp) (Term.0.round.Sem a1 asp)))
     ((fol.0.atom.one_a a0 a1) (one_a (Term.0.round.Sem a0 asp) (Term.0.round.Sem a1 asp)))
  )))
