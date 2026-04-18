(provide Pos LTL LtlAsp Ltl.Sem
p0
p1
ltl.p0
ltl.p1
ltl.not
ltl.X
ltl.G
ltl.F
ltl.and
ltl.or
ltl.=>
ltl.U)
(set-logic ALL)

(declare-sort Pos 0)
(declare-fun p0 (Pos) Bool)
(declare-fun p1 (Pos) Bool)

(declare-term-types
 ((LTL 0))
 (((ltl.p0)
   (ltl.p1)
   (ltl.not LTL)
   (ltl.X LTL)
   (ltl.G LTL)
   (ltl.F LTL)
   (ltl.and LTL LTL)
   (ltl.or LTL LTL)
   (ltl.=> LTL LTL)
   (ltl.U LTL LTL))))

(define-finite LtlAsp
  (record (model Model) (t Int))
  :where (and (<= 0 t) (< t (len (Pos model)))))

(define-semantics
  ((Ltl.Sem (phi LTL) (asp LtlAsp)))
  ((match phi
     ((ltl.p0) (p0 (t asp)))
     ((ltl.p1) (p1 (t asp)))
     ((ltl.not x)
      (not (Ltl.Sem x asp)))
     ((ltl.X x)
      (exists ((k (int-range (+ (t asp) 1) (- (len (Pos (model asp))) 1))))
        (and (= k (+ (t asp) 1))
             (Ltl.Sem x (LtlAsp (model asp) k)))))
     ((ltl.G x)
      (forall ((k (int-range (t asp) (- (len (Pos (model asp))) 1))))
        (Ltl.Sem x (LtlAsp (model asp) k))))
     ((ltl.F x)
      (exists ((k (int-range (t asp) (- (len (Pos (model asp))) 1))))
        (Ltl.Sem x (LtlAsp (model asp) k))))
     ((ltl.and x y)
      (and (Ltl.Sem x asp) (Ltl.Sem y asp)))
     ((ltl.or x y)
      (or (Ltl.Sem x asp) (Ltl.Sem y asp)))
     ((ltl.=> x y)
      (or (not (Ltl.Sem x asp)) (Ltl.Sem y asp)))
     ((ltl.U x y)
      (exists ((k (int-range (t asp) (- (len (Pos (model asp))) 1))))
        (and
          (Ltl.Sem y (LtlAsp (model asp) k))
          (forall ((j (int-range (t asp) (- k 1))))
            (Ltl.Sem x (LtlAsp (model asp) j)))))))))
