(provide CFG Rhs Rules CFG.Asp CFG.Sem Rhs.Sem Rules.Sem cfg.top.S cfg.++ cfg.rhs.S cfg.rhs.T cfg.lhs.S cfg.lhs.T end char.0 char.1)
(require "../../common/signatures/main.sl")

(declare-term-types
 ((CFG 0) (Rhs 0) (Rules 0))
 (
   (
      (cfg.top.S Rhs Rules)
    )
   (
      (char.0)
      (char.1)
      (cfg.empty)
      (cfg.++ Rhs Rhs)
      (cfg.rhs.S)
      (cfg.rhs.T)
    )
   (
      (cfg.lhs.S Rhs Rules)
      (cfg.lhs.T Rhs Rules)
      (end)
   )
  )
)

(define-finite CFG.Asp
  (record
    (model Model)
    (i Int)
    (j Int)
    (mode (enum base reset.S find.S reset.T find.T)))
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))

(define-semantics
  ((CFG.Sem (n CFG) (asp CFG.Asp)))
  ((match n
     ((cfg.top.S x y)
      (or
        (and (= (mode asp) reset.S)
             (or (Rhs.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) reset.T)
             (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T))))))))

(define-semantics
  ((Rhs.Sem (n Rhs) (asp CFG.Asp)))
  ((match n
     ((char.0)
      (or
        (and (= (mode asp) base)
             (= (j asp) (+ (i asp) 1))
             (zero (i asp)))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))

     ((char.1)
      (or
        (and (= (mode asp) base)
             (= (j asp) (+ (i asp) 1))
             (one (i asp)))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))
     
     ((cfg.empty)
      (and (= (mode asp) base)
           (= (i asp) (j asp))))

     ((cfg.++ x y)
      (or
        (and (= (mode asp) base)
             (exists ((k (int-range (i asp) (j asp))))
               (and (Rhs.Sem x (CFG.Asp (model asp) (i asp) k base))
                    (Rhs.Sem y (CFG.Asp (model asp) k (j asp) base)))))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))

     ((cfg.rhs.S)
      (and (= (mode asp) base)
           (CFG.Sem (parent n) (CFG.Asp (model asp) (i asp) (j asp) reset.S))))

     ((cfg.rhs.T)
      (and (= (mode asp) base)
           (CFG.Sem (parent n) (CFG.Asp (model asp) (i asp) (j asp) reset.T)))))))

(define-semantics
  ((Rules.Sem (n Rules) (asp CFG.Asp)))
  ((match n
     ((cfg.lhs.S x y)
      (or
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))
        (and (= (mode asp) find.S)
             (or (Rhs.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) find.T)
             (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T)))))

     ((cfg.lhs.T x y)
      (or
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))
        (and (= (mode asp) find.T)
             (or (Rhs.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T))))
        (and (= (mode asp) find.S)
             (Rules.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S)))))

     ((end)
      false))))
