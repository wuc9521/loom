(provide FoAsp0 Term.0.chord.node Term.0.chord.node.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.chord.node fol.0.atom.chord.bs.f fol.0.atom.chord.bs.p fol.0.atom.chord.live fol.0.atom.chord.prd fol.0.atom.chord.prd_proj fol.0.atom.chord.ring.btw fol.0.atom.chord.scheduled_rectify fol.0.atom.chord.scheduled_stabilize fol.0.atom.chord.stable fol.0.atom.chord.started fol.0.term.chord.tp1.n0)
(require "../../common/signatures/chord-2f2f7007.sl")

(declare-term-types
 ((Formula0 0) (Term.0.chord.node 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.chord.node Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.bs.f Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.bs.p Term.0.chord.node Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.live Term.0.chord.node)
   (fol.0.atom.chord.prd Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.prd_proj Term.0.chord.node)
   (fol.0.atom.chord.ring.btw Term.0.chord.node Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.scheduled_rectify Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.scheduled_stabilize Term.0.chord.node Term.0.chord.node)
   (fol.0.atom.chord.stable )
   (fol.0.atom.chord.started ))
  ((fol.0.term.chord.tp1.n0))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.chord.node.Sem (term Term.0.chord.node) (asp FoAsp0)))
  :returns chord.node
  ((match term
     ((fol.0.term.chord.tp1.n0) chord.tp1.n0)
  )))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.eq.chord.node lhs rhs)
      (= (Term.0.chord.node.Sem lhs asp) (Term.0.chord.node.Sem rhs asp)))
     ((fol.0.atom.chord.bs.f a0 a1) (chord.bs.f (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp)))
     ((fol.0.atom.chord.bs.p a0 a1 a2) (chord.bs.p (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp) (Term.0.chord.node.Sem a2 asp)))
     ((fol.0.atom.chord.live a0) (chord.live (Term.0.chord.node.Sem a0 asp)))
     ((fol.0.atom.chord.prd a0 a1) (chord.prd (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp)))
     ((fol.0.atom.chord.prd_proj a0) (chord.prd_proj (Term.0.chord.node.Sem a0 asp)))
     ((fol.0.atom.chord.ring.btw a0 a1 a2) (chord.ring.btw (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp) (Term.0.chord.node.Sem a2 asp)))
     ((fol.0.atom.chord.scheduled_rectify a0 a1) (chord.scheduled_rectify (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp)))
     ((fol.0.atom.chord.scheduled_stabilize a0 a1) (chord.scheduled_stabilize (Term.0.chord.node.Sem a0 asp) (Term.0.chord.node.Sem a1 asp)))
     ((fol.0.atom.chord.stable ) (chord.stable))
     ((fol.0.atom.chord.started ) (chord.started))
  )))
