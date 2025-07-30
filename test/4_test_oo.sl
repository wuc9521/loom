(set-logic RE)

(declare-sort Char 0)
(declare-sort Pos 0)
(declare-fun a_char () Char)
(declare-fun b_char () Char)


(define-class (Model [Pos])
  ([size Int]
   [toInt (-> Pos Int)]
   [charAt (-> Pos Char)]
   [succ (-> Pos Pos)]
   [leq (-> Pos Pos Bool)])
  (define-fun-rec eval ((self Model) (iv Interval) (phi RE)) Bool
    (match phi (
      (re.empty false)
      (re.eps (= (iv-len iv) 0))
      ((re.sym c) (and (= (iv-len iv) 1) (= c (charAt (l iv)))))
      ((re.union p q) (or (eval self iv p) (eval self iv q)))
      ((re.++ p q) 
        (exists ((cut Pos))
          (and (leq (l iv) cut) (leq cut (r iv))
               (eval self (mk-iv (l iv) cut) p)
               (eval self (mk-iv cut (r iv)) q))))
      ; ((re.* p)
      ;   (mu (Star Bool Interval)
      ;     (lambda ((seg Interval))
      ;       (or (= (l seg) (r seg))
      ;           (exists ((cut Pos))
      ;             (and (leq (l seg) cut) (leq cut (r seg))
      ;                  (eval self (mk-iv (l seg) cut) p)
      ;                  (Star (mk-iv cut (r seg)))))))
      ;     iv))
      )
    )
  )
)

(define-sort-enum Pos_w1 ($0 $1))
; "define-object" ~ Symbol ~ "(" ~ Symbol ~ Sort* ~ ")" ~ MethodDef* ~ ")"  }
(define-object w1 (Model Pos_w1)
  (define-fun size () Int 2)
  (define-fun toInt ((p Pos_w1)) Int 
    (match p (
      ($0 0) 
      ($1 1)))
    )
  (define-fun charAt ((p Pos_w1)) Char 
    (match p (($0 a_char) ($1 b_char))))
  (define-fun succ ((p Pos_w1)) Pos_w1
    (match p (($0 $1) ($1 $0))))
  (define-fun leq ((p1 Pos_w1) (p2 Pos_w1)) Bool
    (match p1 (($0 true) ($1 (match p2 (($0 false) ($1 true)))))))
)

(echo "Testing object-oriented syntax...")
(echo w1.size)
(echo (w1.toInt $0))
(echo (w1.charAt $0)) 