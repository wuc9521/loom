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
   (ctl.EX CTL)
   (ctl.AX CTL))))

(define-finite CTL.Asp
  (record (model Model) (state U))
  :where true)

(define-semantics
  ((CTL.Sem (phi CTL) (asp CTL.Asp)))
  ((match phi
     ((p) (p (state asp)))
     ((ctl.not x) (not (CTL.Sem x asp)))
     ((ctl.and x y) (and (CTL.Sem x asp) (CTL.Sem y asp)))
     ((ctl.or x y) (or (CTL.Sem x asp) (CTL.Sem y asp)))
     ((ctl.EX x)
      (exists ((next (domain (U (model asp)))))
        (and (R (state asp) next)
             (CTL.Sem x (CTL.Asp (model asp) next)))))
     ((ctl.AX x)
      (forall ((next (domain (U (model asp)))))
        (or (not (R (state asp) next))
            (CTL.Sem x (CTL.Asp (model asp) next))))))))

(synth-fun phi () CTL)
(constraint (CTL.Sem phi (CTL.Asp k-pos $0)))
(constraint (not (CTL.Sem phi (CTL.Asp k-neg $0))))
(check-synth)
