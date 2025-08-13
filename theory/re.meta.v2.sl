(define-enum-sort Char ($a $b))
(define-fun a () Char $a)
(define-fun b () Char $b)


(declare-datatype RE (
    (re.empty) 
    (re.eps)
    (re.sym (sym Char))
    (re.union (lhs RE) (rhs RE))
    (re.++ (lhs RE) (rhs RE))
    (re.* (body RE))
    (re.comp (body RE))
    (re.inter (lhs RE) (rhs RE))
))

(declare-sort T 0)
(declare-fun toInt (T) Int)
(declare-fun charAt (T) Char)
(declare-fun fst () T) 
(declare-fun end () T)
(declare-datatype Interval (
    (mk-iv 
        (l-iv T) 
        (r-iv T)
    ))
)
(constraint (forall ((iv Interval)) 
    (<= (toInt (l-iv iv)) (toInt (r-iv iv))
)))

(define-fun iv-len ((iv Interval)) Int
        (- (toInt (r-iv iv)) (toInt (l-iv iv))))

(define-fun-rec eval ((phi RE) (iv Interval)) Bool
    (match phi (
        (re.empty false)
        (re.eps (= (iv-len iv) 0))
        ((re.sym c)
            (and (= (iv-len iv) 1)
                (exists ((p T))
                    (and (= (toInt p) (toInt (l-iv iv)))
                        (= (charAt p) c)
                        ))))
        ((re.union p q)
            (or (eval p iv)
                (eval q iv)))
        ((re.++ p q)
            (exists ((cut T))
                (and (and (<= (toInt (l-iv iv)) (toInt cut))
                    (<= (toInt cut) (toInt (r-iv iv))))
                    (and (eval p (mk-iv (l-iv iv) cut))
                        (eval q (mk-iv cut (r-iv iv)))))))
        ((re.* p)
            ((mu (Star ((seg Interval)) Bool) ; (mu name (param) return sort)
                (lambda ((seg Interval)) ; body
                    (or (= (iv-len seg) 0) ; empty interval
                        (exists ((cut T))
                            (and (and (<= (toInt (l-iv seg)) (toInt cut)) (<= (toInt cut) (toInt (r-iv seg))))
                                (and (eval p (mk-iv (l-iv seg) cut))
                                    (Star (mk-iv cut (r-iv seg)))))))))
                    iv))
        )
    )
)

(define-fun models ((phi RE)) Bool
    (eval phi (mk-iv fst end))
)

(define-enum-sort Pos_w1 ($0 $1))
(define-enum-sort Pos_w2 ($0 $1 $2 $3))
(define-enum-sort Pos_w4 ($0 $1 $2))

(define-model w1
    (interp-sort T () Pos_w1)
    (interp-fun fst () T $0)
    (interp-fun end () T $1)
    (interp-fun toInt ((p Pos_w1)) Int (match p (($0 0) ($1 1))))
    (interp-fun charAt ((p Pos_w1)) Char (match p (
        ($0 a) 
        ($1 b))))
)


(define-model w2
    (interp-sort T () Pos_w2)
    (interp-fun fst () T $0)
    (interp-fun end () T $3)
    (interp-fun toInt ((p Pos_w2)) Int
    (match p (($0 0) ($1 1) ($2 2) ($3 3))))
    (interp-fun charAt ((p Pos_w2)) Char
    (match p (
        ($0 a) 
        ($2 a)
        ($1 b) 
        ($3 b))))
)



(define-model w4
    (interp-sort T () Pos_w4)
    (interp-fun fst () T $0)
    (interp-fun end () T $2)
    (interp-fun toInt ((p Pos_w4)) Int (match p (($0 0) ($1 1) ($2 2))))
    (interp-fun charAt ((p Pos_w4)) Char
    (match p (
        ($0 b) 
        (_ a))))
)



(synth-fun regex () RE)

; switch symbol-map according to model name
(constraint (w1 (models regex)))    
(constraint (w2 (models regex)))
(constraint (not (w4 (models regex))))
(check-synth)
