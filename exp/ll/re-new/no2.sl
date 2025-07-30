(set-logic ALL)
(declare-sort Pos 0)
(declare-sort Char 0)
(declare-datatype RE (
    (re.empty) 
    (re.eps) 
    (re.sym (sym Char))
    (re.union (lhs RE) (rhs RE))
    (re.++ (lh RE) (rh RE))
    (re.* (body RE))
))
(declare-datatype Interval ((mk-iv (l Pos) (r Pos))))   
(declare-datatype ModelImpl (
  (mk-model-impl
    (size Int)
    (toInt (-> Pos Int))
    (charAt (-> Pos Char))
  )
))

(define-fun-rec eval ((impl ModelImpl) (iv Interval) (phi RE)) Bool
  (match phi (
    (re.empty false)
    (re.eps (= (- (toInt (r iv)) (toInt (l iv))) 0))
    ((re.sym c)
      (and (= (- (toInt (r iv)) (toInt (l iv))) 1)
           (exists ((p Pos))
             (and (= ((toInt impl) p) (toInt (l iv)))
                  (= ((charAt impl) p) c)))))
    ((re.union p q) (or (eval impl iv p) (eval impl iv q)))
    ((re.++ p q)
      (exists ((cut Pos))
        (and (<= (toInt (l iv)) (toInt cut)) (<= (toInt cut) (toInt (r iv)))
             (and (eval impl (mk-iv (l iv) cut) p)
                  (eval impl (mk-iv cut (r iv)) q)))))
  ))
)

(define-fun models ((impl ModelImpl) (phi RE)) Bool
  (eval impl (mk-iv 0 (mk-pos (size impl))) phi))


(define-sort-enum Pos_w1 ($w1_0 $w1_1))
(define-fun toInt_w1 ((p Pos_w1)) Int (match p (($w1_0 0) ($w1_1 1))))
(declare-const a_char Char)
(declare-const b_char Char)
(assert (distinct a_char b_char))
(define-fun charAt_w1 ((p Pos_w1)) Char (match p (($w1_0 a_char) ($w1_1 b_char))))
(define-fun model_w1 () ModelImpl
  (mk-model-impl 2 toInt_w1 charAt_w1))

(define-sort-enum Pos_w2 ($w2_0 $w2_1 $w2_2 $w2_3))
(define-fun toInt_w2 ((p Pos_w2)) Int (match p (($w2_0 0) ($w2_1 1) ($w2_2 2) ($w2_3 3))))
(define-fun charAt_w2 ((p Pos_w2)) Char
  (match p (
    ($w2_0 a_char) ($w2_2 a_char)
    ($w2_1 b_char) ($w2_3 b_char)
  )))
(define-fun model_w2 () ModelImpl
  (mk-model-impl 4 toInt_w2 charAt_w2))

(define-sort-enum Pos_w4 ($w4_0 $w4_1 $w4_2))
(define-fun toInt_w4 ((p Pos_w4)) Int (match p (($w4_0 0) ($w4_1 1) ($w4_2 2))))
(define-fun charAt_w4 ((p Pos_w4)) Char
  (match p (
      ($w4_0 b_char)
      (_ a_char)
  )))
(define-fun model_w4 () ModelImpl
  (mk-model-impl 3 toInt_w4 charAt_w4))
(synth-fun regex () RE
    ((Start RE))
    ( 
        (Start RE
            ( 
                (re.sym a_char)        ; a
                (re.sym b_char)        ; b
                (re.++ Start Start)    ; concat
                (re.union Start Start) ; union
                (re.inter Start Start) ; (intersection/complement can be added if defined in framework)
                (re.* Start)           ; Kleene star
            )
        )
    )
)

(constraint (models model_w1 regex))
(constraint (models model_w2 regex))
(constraint (not (models model_w4 regex)))

(check-synth)