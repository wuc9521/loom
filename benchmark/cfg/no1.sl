(set-logic ALL)

(declare-term-types  
 ((CFG 0) (Rhs 0) (Rules 0))
 (
   (
      (cfg.top.S Rhs Rules)
    )
   (
      (a)      ; The leading `$`-signs are a naming convention with no special significance.
      (b) 
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

(declare-sort Pos 0)

; define signatures here.
(declare-fun a (Pos) Bool)
(declare-fun b (Pos) Bool)


; a
(define-model w0
  (define-univ Pos ($0))
  (interp-fun a (match (
      (($0) true)
      ((_) false)
  )))
  (interp-fun b (match (
      ((_) false)
  )))
)

; ab
(define-model w1
  (define-univ Pos ($0 $1))
  (interp-fun a (match (
      (($0) true)
      ((_) false)
  )))
  (interp-fun b (match (
      (($0) false)
      ((_) true)
  )))
)

; abab
(define-model w2
  (define-univ Pos ($0 $1 $2 $3))
  (interp-fun a (match (
      (($0) true)
      (($2) true)
      ((_) false)
  )))   
  (interp-fun b (match (
      (($1) true)
      (($3) true)
      ((_) false)
  )))
)

; aab
(define-model w5
  (define-univ Pos ($0 $1 $2))
  (interp-fun a (match (
      (($0) true)
      (($1) true)
      ((_) false)
  )))   
  (interp-fun b (match (
      (($2) true)
      ((_) false)
  )))
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
             (or (CFG.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) reset.T)
             (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T)))))

     ((a)
      (or
        (and (= (mode asp) base)
             (= (j asp) (+ (i asp) 1))
             (a (i asp)))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))

     ((b)
      (or
        (and (= (mode asp) base)
             (= (j asp) (+ (i asp) 1))
             (b (i asp)))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))

     ((cfg.++ x y)
      (or
        (and (= (mode asp) base)
             (exists ((k (int-range (i asp) (j asp))))
               (and (CFG.Sem x (CFG.Asp (model asp) (i asp) k base))
                    (CFG.Sem y (CFG.Asp (model asp) k (j asp) base)))))
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))))

     ((cfg.rhs.S)
      (and (= (mode asp) base)
           (CFG.Sem (parent n) (CFG.Asp (model asp) (i asp) (j asp) reset.S))))

     ((cfg.rhs.T)
      (and (= (mode asp) base)
           (CFG.Sem (parent n) (CFG.Asp (model asp) (i asp) (j asp) reset.T))))

     ((cfg.lhs.S x y)
      (or
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))
        (and (= (mode asp) find.S)
             (or (CFG.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S))))
        (and (= (mode asp) find.T)
             (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T)))))

     ((cfg.lhs.T x y)
      (or
        (and (or (= (mode asp) reset.S) (= (mode asp) reset.T))
             (CFG.Sem (parent n) asp))
        (and (= (mode asp) find.T)
             (or (CFG.Sem x (CFG.Asp (model asp) (i asp) (j asp) base))
                 (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.T))))
        (and (= (mode asp) find.S)
             (CFG.Sem y (CFG.Asp (model asp) (i asp) (j asp) find.S)))))

     ((end)
      false))))

(synth-fun g () CFG)
(constraint (CFG.Sem g (CFG.Asp w1 0 (size (Pos w1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp w2 0 (size (Pos w2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp w5 0 (size (Pos w5)) reset.S))))

(check-synth)
