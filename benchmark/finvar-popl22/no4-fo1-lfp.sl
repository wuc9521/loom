(require "theories/common-t12f34.sl")

; A hand-written FO1-LFP-style encoding over the same G1..G4 structures as no1/no2/no3.
;
; The intended recursive predicate is a unary least fixed point:
;   R(x) := (x = t) or exists y. (E x y and R(y))
;
; and the synthesized top-level formula can use that recursive notion at one bound variable.
;
; The point of this file is to make the "where does parent-navigation appear?" question obvious:
; it appears only in FO1L.ctx-x0.FindLfp below, where a recursive occurrence climbs from
; `(FO1L.ctx-x0.atom.rec a0)` to the nearest enclosing `(FO1L.ctx-x0.lfp p)`.

(declare-term-types
 ((FO1L.Formula 0)
  (FO1L.Formula.x0 0)
  (FO1L.TermU 0)
  (FO1L.TermU.x0 0))
 (((FO1L.true)
   (FO1L.false)
   (FO1L.not FO1L.Formula)
   (FO1L.and FO1L.Formula FO1L.Formula)
   (FO1L.or FO1L.Formula FO1L.Formula)
   (FO1L.forall.x0.U FO1L.Formula.x0)
   (FO1L.exists.x0.U FO1L.Formula.x0)
   (FO1L.eq.U FO1L.TermU FO1L.TermU)
   (FO1L.atom.E FO1L.TermU FO1L.TermU))
  ((FO1L.ctx-x0.true)
   (FO1L.ctx-x0.false)
   (FO1L.ctx-x0.not FO1L.Formula.x0)
   (FO1L.ctx-x0.and FO1L.Formula.x0 FO1L.Formula.x0)
   (FO1L.ctx-x0.or FO1L.Formula.x0 FO1L.Formula.x0)
   (FO1L.ctx-x0.forall.x0.U FO1L.Formula.x0)
   (FO1L.ctx-x0.exists.x0.U FO1L.Formula.x0)
   (FO1L.ctx-x0.eq.U FO1L.TermU.x0 FO1L.TermU.x0)
   (FO1L.ctx-x0.atom.E FO1L.TermU.x0 FO1L.TermU.x0)
   (FO1L.ctx-x0.lfp FO1L.Formula.x0)
   (FO1L.ctx-x0.atom.rec FO1L.TermU.x0))
  ((FO1L.term.s)
   (FO1L.term.t))
  ((FO1L.ctx-x0.var)
   (FO1L.ctx-x0.term.s)
   (FO1L.ctx-x0.term.t))))

(define-finite FO1L.Asp (record (model Model)))
(define-finite FO1L.Asp.x0 (record (model Model) (x0 U)))

; `fuel` is the finite semantic aspect that approximates the least fixed point.
(define-finite FO1L.Asp.x0.lfp
  (record (model Model) (x0 U) (fuel Int))
  :where (<= fuel (size (U model))))

(define-semantics
  ((FO1L.TermU.Sem (term FO1L.TermU) (asp FO1L.Asp)))
  :returns U
  ((match term
     ((FO1L.term.s) s)
     ((FO1L.term.t) t))))

(define-semantics
  ((FO1L.TermU.x0.Sem (term FO1L.TermU.x0) (asp FO1L.Asp.x0)))
  :returns U
  ((match term
     ((FO1L.ctx-x0.var) (x0 asp))
     ((FO1L.ctx-x0.term.s) s)
     ((FO1L.ctx-x0.term.t) t))))

(define-semantics
  ((FO1L.Sem (phi FO1L.Formula) (asp FO1L.Asp)))
  ((match phi
     ((FO1L.true) true)
     ((FO1L.false) false)
     ((FO1L.not psi) (not (FO1L.Sem psi asp)))
     ((FO1L.and psi1 psi2) (and (FO1L.Sem psi1 asp) (FO1L.Sem psi2 asp)))
     ((FO1L.or psi1 psi2) (or (FO1L.Sem psi1 asp) (FO1L.Sem psi2 asp)))
     ((FO1L.forall.x0.U psi)
      (forall ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.Sem psi (FO1L.Asp.x0 (model asp) v))))
     ((FO1L.exists.x0.U psi)
      (exists ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.Sem psi (FO1L.Asp.x0 (model asp) v))))
     ((FO1L.eq.U lhs rhs)
      (= (FO1L.TermU.Sem lhs asp) (FO1L.TermU.Sem rhs asp)))
     ((FO1L.atom.E lhs rhs)
      (E (FO1L.TermU.Sem lhs asp) (FO1L.TermU.Sem rhs asp))))))

(define-semantics
  ((FO1L.ctx-x0.Sem (phi FO1L.Formula.x0) (asp FO1L.Asp.x0)))
  ((match phi
     ((FO1L.ctx-x0.true) true)
     ((FO1L.ctx-x0.false) false)
     ((FO1L.ctx-x0.not psi) (not (FO1L.ctx-x0.Sem psi asp)))
     ((FO1L.ctx-x0.and psi1 psi2) (and (FO1L.ctx-x0.Sem psi1 asp) (FO1L.ctx-x0.Sem psi2 asp)))
     ((FO1L.ctx-x0.or psi1 psi2) (or (FO1L.ctx-x0.Sem psi1 asp) (FO1L.ctx-x0.Sem psi2 asp)))
     ((FO1L.ctx-x0.forall.x0.U psi)
      (forall ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.Sem psi (FO1L.Asp.x0 (model asp) v))))
     ((FO1L.ctx-x0.exists.x0.U psi)
      (exists ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.Sem psi (FO1L.Asp.x0 (model asp) v))))
     ((FO1L.ctx-x0.eq.U lhs rhs)
      (= (FO1L.TermU.x0.Sem lhs asp) (FO1L.TermU.x0.Sem rhs asp)))
     ((FO1L.ctx-x0.atom.E lhs rhs)
      (E (FO1L.TermU.x0.Sem lhs asp) (FO1L.TermU.x0.Sem rhs asp)))
     ((FO1L.ctx-x0.lfp psi)
      (FO1L.ctx-x0.LfpSem psi
        (FO1L.Asp.x0.lfp (model asp) (x0 asp) (size (U (model asp))))))
     ((FO1L.ctx-x0.atom.rec a0) false))))

(define-semantics
  ((FO1L.ctx-x0.LfpSem (phi FO1L.Formula.x0) (asp FO1L.Asp.x0.lfp)))
  ((match phi
     ((FO1L.ctx-x0.true) true)
     ((FO1L.ctx-x0.false) false)
     ((FO1L.ctx-x0.not psi) (not (FO1L.ctx-x0.LfpSem psi asp)))
     ((FO1L.ctx-x0.and psi1 psi2) (and (FO1L.ctx-x0.LfpSem psi1 asp) (FO1L.ctx-x0.LfpSem psi2 asp)))
     ((FO1L.ctx-x0.or psi1 psi2) (or (FO1L.ctx-x0.LfpSem psi1 asp) (FO1L.ctx-x0.LfpSem psi2 asp)))
     ((FO1L.ctx-x0.forall.x0.U psi)
      (forall ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.LfpSem psi (FO1L.Asp.x0.lfp (model asp) v (fuel asp)))))
     ((FO1L.ctx-x0.exists.x0.U psi)
      (exists ((v (domain (U (model asp)))))
        (FO1L.ctx-x0.LfpSem psi (FO1L.Asp.x0.lfp (model asp) v (fuel asp)))))
     ((FO1L.ctx-x0.eq.U lhs rhs)
      (= (FO1L.TermU.x0.Sem lhs (FO1L.Asp.x0 (model asp) (x0 asp)))
         (FO1L.TermU.x0.Sem rhs (FO1L.Asp.x0 (model asp) (x0 asp)))))
     ((FO1L.ctx-x0.atom.E lhs rhs)
      (E (FO1L.TermU.x0.Sem lhs (FO1L.Asp.x0 (model asp) (x0 asp)))
         (FO1L.TermU.x0.Sem rhs (FO1L.Asp.x0 (model asp) (x0 asp)))))
     ((FO1L.ctx-x0.lfp psi)
      (FO1L.ctx-x0.LfpSem psi
        (FO1L.Asp.x0.lfp (model asp) (x0 asp) (size (U (model asp))))))
     ((FO1L.ctx-x0.atom.rec a0)
      (and (not (= (fuel asp) 0))
           (FO1L.ctx-x0.FindLfp (parent phi)
             (FO1L.Asp.x0.lfp
               (model asp)
               (FO1L.TermU.x0.Sem a0 (FO1L.Asp.x0 (model asp) (x0 asp)))
               (- (fuel asp) 1))))))))

(define-semantics
  ((FO1L.ctx-x0.FindLfp (phi FO1L.Formula.x0) (asp FO1L.Asp.x0.lfp)))
  ((match phi
     ((FO1L.ctx-x0.true) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.false) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.not psi) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.and psi1 psi2) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.or psi1 psi2) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ; This is the only place that really needs `parent`.
     ; We are at a recursive occurrence and search upward for the nearest enclosing binder.
     ((FO1L.ctx-x0.lfp psi)
      (FO1L.ctx-x0.LfpSem psi
        (FO1L.Asp.x0.lfp (model asp) (x0 asp) (fuel asp))))
     ((FO1L.ctx-x0.forall.x0.U psi) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.exists.x0.U psi) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.eq.U lhs rhs) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.atom.E lhs rhs) (FO1L.ctx-x0.FindLfp (parent phi) asp))
     ((FO1L.ctx-x0.atom.rec a0) (FO1L.ctx-x0.FindLfp (parent phi) asp)))))

(synth-fun phi () FO1L.Formula)

(constraint (FO1L.Sem phi (FO1L.Asp G1)))
(constraint (FO1L.Sem phi (FO1L.Asp G2)))
(constraint (not (FO1L.Sem phi (FO1L.Asp G3))))
(constraint (not (FO1L.Sem phi (FO1L.Asp G4))))

(check-synth)
