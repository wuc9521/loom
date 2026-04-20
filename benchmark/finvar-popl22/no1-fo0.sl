(require "theories/common-t12f34.sl")

(declare-term-types
 ((FO0.Formula 0) (FO0.TermU 0))
 (((FO0.true)
   (FO0.false)
   (FO0.not FO0.Formula)
   (FO0.and FO0.Formula FO0.Formula)
   (FO0.or FO0.Formula FO0.Formula)
   (FO0.eq.U FO0.TermU FO0.TermU)
   (FO0.atom.E FO0.TermU FO0.TermU))
  ((FO0.term.s)
   (FO0.term.t))))

(define-finite FO0.Asp (record (model Model)))

(define-semantics
  ((FO0.TermU.Sem (term FO0.TermU) (asp FO0.Asp)))
  :returns U
  ((match term
     ((FO0.term.s) s)
     ((FO0.term.t) t))))

(define-semantics
  ((FO0.Sem (phi FO0.Formula) (asp FO0.Asp)))
  ((match phi
     ((FO0.true) true)
     ((FO0.false) false)
     ((FO0.not psi) (not (FO0.Sem psi asp)))
     ((FO0.and psi1 psi2) (and (FO0.Sem psi1 asp) (FO0.Sem psi2 asp)))
     ((FO0.or psi1 psi2) (or (FO0.Sem psi1 asp) (FO0.Sem psi2 asp)))
     ((FO0.eq.U lhs rhs)
      (= (FO0.TermU.Sem lhs asp) (FO0.TermU.Sem rhs asp)))
     ((FO0.atom.E lhs rhs)
      (E (FO0.TermU.Sem lhs asp) (FO0.TermU.Sem rhs asp))))))

(synth-fun phi () FO0.Formula)

(constraint (FO0.Sem phi (FO0.Asp G1)))
(constraint (FO0.Sem phi (FO0.Asp G2)))
(constraint (not (FO0.Sem phi (FO0.Asp G3))))
(constraint (not (FO0.Sem phi (FO0.Asp G4))))

(check-synth)
