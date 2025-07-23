(set-logic BV)
; --- Helper Functions ---
(define-fun bool_to_bv1 ((b Bool)) (_ BitVec 1) (ite b #b1 #b0))
(define-fun b1_to_bool ((b (_ BitVec 1))) Bool (= b #b1))

; Concatenated bitvector length: 16
(define-fun a () (_ BitVec 16) #b1000100010100000)
(define-fun c () (_ BitVec 16) #b0100001100000000)
(define-fun v () (_ BitVec 16) #b0011010000000000)

; --- Logical Operations ---
(define-fun modal.and ((x (_ BitVec 16)) (y (_ BitVec 16))) (_ BitVec 16) (bvand x y))
(define-fun modal.or ((x (_ BitVec 16)) (y (_ BitVec 16))) (_ BitVec 16) (bvor x y))
(define-fun modal.not ((x (_ BitVec 16))) (_ BitVec 16) (bvnot x))

; --- Modal Operators ---
(define-fun modal.box ((phi_val (_ BitVec 16))) (_ BitVec 16)
  (let (
    (seg0 ((_ extract 6 0) phi_val))
    (seg1 ((_ extract 10 7) phi_val))
    (seg2 ((_ extract 12 11) phi_val))
    (seg3 ((_ extract 15 13) phi_val))
  ) ; end bindings
    (concat (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 1 1) seg3)))) (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 0 0) seg3)))) (bool_to_bv1 (and (b1_to_bool ((_ extract 2 2) seg3)))))) (concat (concat #b1 (bool_to_bv1 (and (b1_to_bool ((_ extract 1 1) seg2))))) (concat (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 0 0) seg1)) (b1_to_bool ((_ extract 1 1) seg1)))) (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 3 3) seg1)))) (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 0 0) seg1)))) (bool_to_bv1 (and (b1_to_bool ((_ extract 2 2) seg1)) (b1_to_bool ((_ extract 3 3) seg1))))))) (concat #b1 (concat #b1 (concat #b1 (concat #b1 (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 5 5) seg0)) (b1_to_bool ((_ extract 6 6) seg0)))) (concat (bool_to_bv1 (and (b1_to_bool ((_ extract 3 3) seg0)) (b1_to_bool ((_ extract 4 4) seg0)))) (bool_to_bv1 (and (b1_to_bool ((_ extract 1 1) seg0)) (b1_to_bool ((_ extract 2 2) seg0))))))))))))) ; body of let
  ) ; end let
)
(define-fun modal.dia ((phi_val (_ BitVec 16))) (_ BitVec 16)
  (let (
    (seg0 ((_ extract 6 0) phi_val))
    (seg1 ((_ extract 10 7) phi_val))
    (seg2 ((_ extract 12 11) phi_val))
    (seg3 ((_ extract 15 13) phi_val))
  ) ; end bindings
    (concat (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 1 1) seg3)))) (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 0 0) seg3)))) (bool_to_bv1 (or (b1_to_bool ((_ extract 2 2) seg3)))))) (concat (concat #b0 (bool_to_bv1 (or (b1_to_bool ((_ extract 1 1) seg2))))) (concat (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 0 0) seg1)) (b1_to_bool ((_ extract 1 1) seg1)))) (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 3 3) seg1)))) (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 0 0) seg1)))) (bool_to_bv1 (or (b1_to_bool ((_ extract 2 2) seg1)) (b1_to_bool ((_ extract 3 3) seg1))))))) (concat #b0 (concat #b0 (concat #b0 (concat #b0 (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 5 5) seg0)) (b1_to_bool ((_ extract 6 6) seg0)))) (concat (bool_to_bv1 (or (b1_to_bool ((_ extract 3 3) seg0)) (b1_to_bool ((_ extract 4 4) seg0)))) (bool_to_bv1 (or (b1_to_bool ((_ extract 1 1) seg0)) (b1_to_bool ((_ extract 2 2) seg0))))))))))))) ; body of let
  ) ; end let
)

; --- Synthesis Task ---
(synth-fun separator () (_ BitVec 16)
  (
    (Start (_ BitVec 16))
  )
  (
    (Start (_ BitVec 16)
    ( (modal.and Start Start)
      (modal.or Start Start)
      (modal.not Start)
      (modal.box Start)
      (modal.dia Start)
      a
      c
      v
    )
  ))
)

(constraint (= ((_ extract 0 0 ) separator) #b1)) ; Model K2
(constraint (= ((_ extract 7 7 ) separator) #b1)) ; Model K1
(constraint (= ((_ extract 11 11 ) separator) #b0)) ; Model K3
(constraint (= ((_ extract 13 13 ) separator) #b0)) ; Model K4

(check-synth)