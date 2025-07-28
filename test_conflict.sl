(set-logic RE)

(declare-fun f (Int) Bool)
(declare-fun f (Int) Bool)  ; 这应该报错，因为f已经被声明了

(define-fun g ((x Int)) Bool true)
(define-fun g ((x Int)) Bool false)  ; 这应该报错，因为g已经被定义了

(define-sort-enum Color ($1 $2 $3))
(define-sort-enum Color ($1 $2 $3))  ; 这应该报错，因为Color已经被定义了

(check-synth) 