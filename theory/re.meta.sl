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
))

(model-spec 
    (declare-sort T 0)
    (declare-fun-inline toInt () (-> T Int))
    (declare-fun-inline charAt () (-> T Char))
    (declare-fun-inline fst () T) 
    (declare-fun-inline end () T)
    (declare-datatype Interval (
        (mk-iv 
            (l-iv T) 
            (r-iv T)
        ))
    )
    (define-fun-inline iv-len () (-> Interval Int)
        (lambda ((iv Interval))
            (- (toInt (r-iv iv)) (toInt (l-iv iv)))
        )
    )
    (define-fun iv-valid () (-> Interval Bool)
        (lambda ((iv Interval))
            (<= (iv-len iv) 0)
        )
    )
    (define-fun-rec eval ((phi RE)) (-> Interval Bool)
        (match phi (
            (re.empty 
                (lambda ((iv Interval)) false) ; constant bitvec #b0...0
            )
            (re.eps 
                (lambda ((iv Interval)) (= (iv-len iv) 0)) ; constant bitvec 
            )
            ((re.sym c) ; (define-fun re.sym ((c (_ BitVec ...)) (_ BitVec ...)) .....)
                (lambda ((iv Interval))
                    (and (= (iv-len iv) 1)
                        (exists ((p T))
                            (and (= (toInt p) (toInt (l-iv iv)))
                                (= (charAt p) c)
                                )))
                )
            )
            ((re.union p q) ; bvor
                (lambda ((iv Interval))
                    (or (eval p iv)
                        (eval q iv))
                )
            )
            ((re.++ p q) 
                (lambda ((iv Interval))
                    (exists ((cut T))
                        (and (and (<= (toInt (l-iv iv)) (toInt cut))
                            (<= (toInt cut) (toInt (r-iv iv))))
                            (and (eval p (mk-iv (l-iv iv) cut))
                                (eval q (mk-iv cut (r-iv iv))))))
                )
            )
            ((re.* p)
                (lambda ((iv Interval))
                    ((mu (Star ((seg Interval)) Bool) ; (mu name (param) return sort)
                        (lambda ((seg Interval)) ; body
                            (or (= (iv-len seg) 0) ; empty interval
                                (exists ((cut T))
                                    (and (and (<= (toInt (l-iv seg)) (toInt cut)) (<= (toInt cut) (toInt (r-iv seg))))
                                        (and (eval p (mk-iv (l-iv seg) cut))
                                            (Star (mk-iv cut (r-iv seg)))))))))
                            iv)
                )
            )
        ))
    )
    (define-fun models ((phi RE)) Bool
        (let ((iv (mk-iv fst end)))
            (and (iv-valid iv)
                (eval phi iv)))
    )
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
