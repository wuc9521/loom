(set-logic HORN)

;; --- Term datatype: a | f(t,t)
(declare-datatypes ()
  ((Term
     a
     (f (l Term) (r Term)))))

;; --- Predicates
(declare-fun P1 (Term) Bool)
(declare-fun P2 (Term) Bool)
(declare-fun P3 (Term) Bool)
(declare-fun P4 (Term) Bool)

;; --- Clauses from Example 7.6.2

;; (2) P2(a).
(assert (P2 a))

;; (1) P1(f(f(x1,x2),x3)) ← P2(x1),P2(x2),P2(x3)
(assert
  (forall ((x1 Term) (x2 Term) (x3 Term))
    (=> (and (P2 x1) (P2 x2) (P2 x3))
        (P1 (f (f x1 x2) x3)))))

;; (3) P1(f(a,x)) ← P2(x)
(assert
  (forall ((x Term))
    (=> (P2 x)
        (P1 (f a x)))))

;; (4) P3(f(x,y)) ← P1(x),P2(y)
(assert
  (forall ((x Term) (y Term))
    (=> (and (P1 x) (P2 y))
        (P3 (f x y)))))

;; (5) P4(x) ← P3(x),P1(x)
(assert
  (forall ((x Term))
    (=> (and (P3 x) (P1 x))
        (P4 x))))

;; (6) P2(x) ← P4(f(x,y))
(assert
  (forall ((x Term) (y Term))
    (=> (P4 (f x y))
        (P2 x))))

;; (7) P1(y) ← P4(f(x,y))
(assert
  (forall ((x Term) (y Term))
    (=> (P4 (f x y))
        (P1 y))))

(check-sat)
(get-model)