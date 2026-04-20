(provide FoAsp0 Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or)
(require "../../common/signatures/leader_election_ring_btw_nonreflexive-c4d3b61d.sl")

(declare-term-types
 ((Formula0 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
  )))
