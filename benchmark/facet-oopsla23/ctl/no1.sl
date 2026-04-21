(set-logic ALL)

(declare-sort U 0)
(declare-fun p (U) Bool)
(declare-fun R (U U) Bool)

(define-model k-pos
  (define-univ U ($0 $1))
  (interp-fun p (match (
    (($1) true)
    ((_) false)
  )))
  (interp-fun R (match (
    (($0 $1) true)
    (($1 $1) true)
    ((_ _) false)
  ))))

(define-model k-neg
  (define-univ U ($0 $1))
  (interp-fun p (match (
    ((_) false)
  )))
  (interp-fun R (match (
    (($0 $1) true)
    (($1 $1) true)
    ((_ _) false)
  ))))

(declare-term-types
 ((CTL 0))
 (((p)
   (ctl.not CTL)
   (ctl.and CTL CTL)
   (ctl.or CTL CTL)
   (ctl.ex CTL)
   (ctl.ax CTL))))

(define-finite CtlAsp
  (record (model Model) (state U))
  :where true)

(define-semantics
  ((Ctl.Sem (phi CTL) (asp CtlAsp)))
  ((match phi
     ((p) (p (state asp)))
     ((ctl.not x) (not (Ctl.Sem x asp)))
     ((ctl.and x y) (and (Ctl.Sem x asp) (Ctl.Sem y asp)))
     ((ctl.or x y) (or (Ctl.Sem x asp) (Ctl.Sem y asp)))
     ((ctl.ex x)
      (exists ((next (domain (U (model asp)))))
        (and (R (state asp) next)
             (Ctl.Sem x (CtlAsp (model asp) next)))))
     ((ctl.ax x)
      (forall ((next (domain (U (model asp)))))
        (or (not (R (state asp) next))
            (Ctl.Sem x (CtlAsp (model asp) next))))))))

(synth-fun phi () CTL)
(constraint (Ctl.Sem phi (CtlAsp k-pos $0)))
(constraint (not (Ctl.Sem phi (CtlAsp k-neg $0))))
(check-synth)
