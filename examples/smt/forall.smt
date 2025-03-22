(set-logic AUFLIA)  ; 允许量词的整数算术
(declare-const x Int)
(declare-const y Int)
(declare-const z Int)

(assert (forall ((v Int)) 
  (=> (or (= v x) (= v y)) 
      (<= v 10))))  ; 只允许使用 x, y，不允许使用 z
(check-sat)