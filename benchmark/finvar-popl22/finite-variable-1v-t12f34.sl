(require "theories/common-t12f34.sl")

(declare-term-types
 ((FO1.Formula 0) (FO1.Formula.x0 0) (FO1.TermU 0) (FO1.TermU.x0 0))
 (((FO1.true)
   (FO1.false)
   (FO1.not FO1.Formula)
   (FO1.and FO1.Formula FO1.Formula)
   (FO1.or FO1.Formula FO1.Formula)
   (FO1.forall.x0.U FO1.Formula.x0)
   (FO1.exists.x0.U FO1.Formula.x0)
   (FO1.eq.U FO1.TermU FO1.TermU)
   (FO1.atom.E FO1.TermU FO1.TermU))
  ((FO1.ctx-x0.true)
   (FO1.ctx-x0.false)
   (FO1.ctx-x0.not FO1.Formula.x0)
   (FO1.ctx-x0.and FO1.Formula.x0 FO1.Formula.x0)
   (FO1.ctx-x0.or FO1.Formula.x0 FO1.Formula.x0)
   (FO1.ctx-x0.forall.x0.U FO1.Formula.x0)
   (FO1.ctx-x0.exists.x0.U FO1.Formula.x0)
   (FO1.ctx-x0.eq.U FO1.TermU.x0 FO1.TermU.x0)
   (FO1.ctx-x0.atom.E FO1.TermU.x0 FO1.TermU.x0))
  ((FO1.term.s)
   (FO1.term.t))
  ((FO1.ctx-x0.var)
   (FO1.ctx-x0.term.s)
   (FO1.ctx-x0.term.t))))

(define-finite FO1.Asp (record (model Model)))
(define-finite FO1.Asp.x0 (record (model Model) (x0 U)))

(define-semantics
  ((FO1.TermU.Sem (term FO1.TermU) (asp FO1.Asp)))
  :returns U
  ((match term
     ((FO1.term.s) s)
     ((FO1.term.t) t))))

(define-semantics
  ((FO1.TermU.x0.Sem (term FO1.TermU.x0) (asp FO1.Asp.x0)))
  :returns U
  ((match term
     ((FO1.ctx-x0.var) (x0 asp))
     ((FO1.ctx-x0.term.s) s)
     ((FO1.ctx-x0.term.t) t))))

(define-semantics
  ((FO1.Sem (phi FO1.Formula) (asp FO1.Asp)))
  ((match phi
     ((FO1.true) true)
     ((FO1.false) false)
     ((FO1.not psi) (not (FO1.Sem psi asp)))
     ((FO1.and psi1 psi2) (and (FO1.Sem psi1 asp) (FO1.Sem psi2 asp)))
     ((FO1.or psi1 psi2) (or (FO1.Sem psi1 asp) (FO1.Sem psi2 asp)))
     ((FO1.forall.x0.U psi)
      (forall ((v (domain (U (model asp)))))
        (FO1.ctx-x0.Sem psi (FO1.Asp.x0 (model asp) v))))
     ((FO1.exists.x0.U psi)
      (exists ((v (domain (U (model asp)))))
        (FO1.ctx-x0.Sem psi (FO1.Asp.x0 (model asp) v))))
     ((FO1.eq.U lhs rhs)
      (= (FO1.TermU.Sem lhs asp) (FO1.TermU.Sem rhs asp)))
     ((FO1.atom.E lhs rhs)
      (E (FO1.TermU.Sem lhs asp) (FO1.TermU.Sem rhs asp))))))

(define-semantics
  ((FO1.ctx-x0.Sem (phi FO1.Formula.x0) (asp FO1.Asp.x0)))
  ((match phi
     ((FO1.ctx-x0.true) true)
     ((FO1.ctx-x0.false) false)
     ((FO1.ctx-x0.not psi) (not (FO1.ctx-x0.Sem psi asp)))
     ((FO1.ctx-x0.and psi1 psi2) (and (FO1.ctx-x0.Sem psi1 asp) (FO1.ctx-x0.Sem psi2 asp)))
     ((FO1.ctx-x0.or psi1 psi2) (or (FO1.ctx-x0.Sem psi1 asp) (FO1.ctx-x0.Sem psi2 asp)))
     ((FO1.ctx-x0.forall.x0.U psi)
      (forall ((v (domain (U (model asp)))))
        (FO1.ctx-x0.Sem psi (FO1.Asp.x0 (model asp) v))))
     ((FO1.ctx-x0.exists.x0.U psi)
      (exists ((v (domain (U (model asp)))))
        (FO1.ctx-x0.Sem psi (FO1.Asp.x0 (model asp) v))))
     ((FO1.ctx-x0.eq.U lhs rhs)
      (= (FO1.TermU.x0.Sem lhs asp) (FO1.TermU.x0.Sem rhs asp)))
     ((FO1.ctx-x0.atom.E lhs rhs)
      (E (FO1.TermU.x0.Sem lhs asp) (FO1.TermU.x0.Sem rhs asp))))))

(synth-fun phi () FO1.Formula)

(constraint (FO1.Sem phi (FO1.Asp G1)))
(constraint (FO1.Sem phi (FO1.Asp G2)))
(constraint (not (FO1.Sem phi (FO1.Asp G3))))
(constraint (not (FO1.Sem phi (FO1.Asp G4))))

(check-synth)
