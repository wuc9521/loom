(set-info :author "Chentian Wu")

(declare-sort Pos 0)

(declare-term-types
 ((RegLan 0))
 (((re.any)
   (re.? RegLan)
   (re.++ RegLan RegLan)
   (re.union RegLan RegLan)
    (re.* RegLan))))

(define-finite ReAsp                                             
  (record (model Model) (i Int) (j Int))                         
  :where (and (<= 0 i) (<= i j) (<= j (size (Pos model)))))

(define-semantics
  ((RegLan.Sem (r RegLan) (asp ReAsp)))
  ((match r
     ((re.any)
      (= (j asp) (+ (i asp) 1)))
     ((re.? x)
      (or (= (i asp) (j asp))
          (RegLan.Sem x asp)))
     ((re.++ x y)
      (exists ((k (int-range (i asp) (j asp))))
        (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
             (RegLan.Sem y (ReAsp (model asp) k (j asp))))))
     ((re.union x y)
      (or (RegLan.Sem x asp) (RegLan.Sem y asp)))
     ((re.* x)
      (or (= (i asp) (j asp))
          (exists ((k (int-range (+ (i asp) 1) (j asp))))
            (and (RegLan.Sem x (ReAsp (model asp) (i asp) k))
                 (RegLan.Sem r (ReAsp (model asp) k (j asp))))))))))


(define-model p01
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) true) (($1) false))))
  (interp-fun one (match ((($0) false) (($1) true)))))


(define-model n11
  (define-univ Pos ($0 $1))
  (interp-fun zero (match ((($0) false) (($1) false))))
  (interp-fun one (match ((($0) true) (($1) true)))))


(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (size (Pos p01)))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))

(check-synth)
