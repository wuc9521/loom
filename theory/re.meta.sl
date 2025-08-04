(declare-sort Char 0)
(declare-fun a () Char)
(declare-fun b () Char)


(declare-datatype RE (
    (re.empty) (re.eps)
    (re.sym (sym Char))
    (re.union (lhs RE) (rhs RE))
    (re.++ (lhs RE) (rhs RE))
    (re.* (body RE))
    (re.inter (lhs RE) (rhs RE))
    (re.comp (body RE))
))


(define-class (Model [T])
    (
        [size Int]
        [toInt (-> T Int)]
        [charAt (-> T Char)]
    )

    (declare-datatype Interval
        ((mk-iv (l T) (r T))))

    (define-fun iv-len ((iv Interval)) Int
        (- (toInt (r iv)) (toInt (l iv))))
    
    (define-fun-rec eval ((m Model) (iv Interval) (phi RE)) Bool
        (match phi (
            (re.empty false)
            (re.eps (= (iv-len iv) 0))
            ((re.sym c)
                (and (= (iv-len iv) 1)
                    (exists ((p T))
                        (and (= (toInt p) (toInt (l iv)))
                            (= (charAt p) c)
                            ))))
            ((re.union p q)
            (or (eval m iv p)
                (eval m iv q)))
            ((re.++ p q)
                (exists ((cut T))
                    (and (and (<= (toInt (l iv)) (toInt cut))
                        (<= (toInt cut) (toInt (r iv))))
                        (and (eval m (mk-iv (l iv) cut) p)
                            (eval m (mk-iv cut (r iv)) q)))))
            ((re.* p)
            ((mu (Star ((seg Interval)) Bool) ; (mu name (param) return sort)
                (lambda ((seg Interval)) ; body
                    (or (= (toInt (l seg)) (toInt (r seg))) ; empty interval
                        (exists ((cut T))
                            (and (and (<= (toInt (l seg)) (toInt cut)) (<= (toInt cut) (toInt (r seg))))
                                (and (eval m (mk-iv (l seg) cut) p)
                                    (Star (mk-iv cut (r seg)))))))))
                    iv))
            )
        )
    )
    (define-fun models ((m Model) (phi RE)) Bool
        (eval m (mk-iv (toInt (l iv)) (toInt (r iv))) phi)
    )
)


(define-enum-sort Pos_w1 ($0 $1))
(define-object w1 (Model Pos_w1)
    (define-fun size () Int 2)
    (define-fun toInt ((p Pos_w1)) Int (match p (($0 0) ($1 1))))
    (define-fun charAt ((p Pos_w1)) Char (match p (
        ($0 a) 
        ($1 b))))
)

(define-enum-sort Pos_w2 ($0 $1 $2 $3))
(define-object w2 (Model Pos_w2)
    (define-fun size () Int 4)
    (define-fun toInt ((p Pos_w2)) Int
    (match p (($0 0) ($1 1) ($2 2) ($3 3))))
    (define-fun charAt ((p Pos_w2)) Char
    (match p (
        ($0 a) 
        ($2 a)
        ($1 b) 
        ($3 b))))
)



(define-enum-sort Pos_w4 ($0 $1 $2))
(define-object w4 (Model Pos_w4)
    (define-fun size () Int 3)
    (define-fun toInt ((p Pos_w4)) Int (match p (($0 0) ($1 1) ($2 2))))
    (define-fun charAt ((p Pos_w4)) Char
    (match p (
        ($0 b) 
        (_ a))))
)



(synth-fun regex () RE)

(constraint (models w1 regex))
(constraint (models w2 regex))
(constraint (not (models w4 regex)))
(check-synth)
