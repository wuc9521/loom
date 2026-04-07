;; ========== BENCHMARK 3: Grid Structures (10 models) ==========
;; Testing 2D grid-like structures with different connectivity patterns

(set-logic ALL)
(declare-sort U 0)
(declare-fun I () U)
(declare-fun R (U U) Bool)
(declare-fun a (U) Bool)
(declare-fun b (U) Bool)
(declare-fun c (U) Bool)

;; Grid1: 3x3 grid with 4-connectivity
(define-model Grid1
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8))
    (interp-fun R (match (
        ;; Row 1: $0-$1-$2
        (($0 $1) true) (($1 $0) true)
        (($1 $2) true) (($2 $1) true)
        ;; Row 2: $3-$4-$5  
        (($3 $4) true) (($4 $3) true)
        (($4 $5) true) (($5 $4) true)
        ;; Row 3: $6-$7-$8
        (($6 $7) true) (($7 $6) true)
        (($7 $8) true) (($8 $7) true)
        ;; Columns
        (($0 $3) true) (($3 $0) true)
        (($1 $4) true) (($4 $1) true)
        (($2 $5) true) (($5 $2) true)
        (($3 $6) true) (($6 $3) true)
        (($4 $7) true) (($7 $4) true)
        (($5 $8) true) (($8 $5) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true) (($2) true) (($6) true) (($8) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true) (($3) true) (($5) true) (($7) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($4) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Grid2-10: Similar grid structures with different labelings and connectivity
(define-model Grid2
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9))
    (interp-fun R (match (
        ;; 2x5 grid
        (($0 $1) true) (($1 $0) true)
        (($1 $2) true) (($2 $1) true)
        (($2 $3) true) (($3 $2) true)
        (($3 $4) true) (($4 $3) true)
        (($5 $6) true) (($6 $5) true)
        (($6 $7) true) (($7 $6) true)
        (($7 $8) true) (($8 $7) true)
        (($8 $9) true) (($9 $8) true)
        (($0 $5) true) (($5 $0) true)
        (($1 $6) true) (($6 $1) true)
        (($2 $7) true) (($7 $2) true)
        (($3 $8) true) (($8 $3) true)
        (($4 $9) true) (($9 $4) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true) (($4) true) (($5) true) (($9) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true) (($3) true) (($6) true) (($8) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true) (($7) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Grid3-Grid10: More grid variations (abbreviated for space)
(define-model Grid3
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
    (interp-fun R (match (
        ;; Complex connectivity pattern
        (($0 $1) true) (($1 $2) true) (($2 $3) true)
        (($3 $4) true) (($4 $5) true) (($5 $6) true)
        (($6 $7) true) (($7 $8) true) (($8 $9) true)
        (($9 $10) true) (($10 $0) true)
        (($0 $5) true) (($1 $6) true) (($2 $7) true)
        (($3 $8) true) (($4 $9) true) (($5 $10) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true) (($3) true) (($6) true) (($9) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true) (($4) true) (($7) true) (($10) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true) (($5) true) (($8) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Grid4-Grid10 similar pattern (abbreviated)
(define-model Grid4
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
    (interp-fun R (match (
        (($0 $1) true) (($1 $2) true) (($2 $3) true)
        (($3 $4) true) (($4 $5) true) (($5 $6) true)
        (($6 $7) true) (($7 $8) true) (($8 $9) true)
        (($9 $10) true) (($10 $0) true)
        (($1 $5) true) (($2 $6) true) (($3 $7) true)
        (($4 $8) true) (($5 $9) true) (($6 $10) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($1) true) (($4) true) (($7) true) (($10) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true) (($3) true) (($6) true) (($9) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true) (($5) true) (($8) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Continue with Grid5-Grid10 following similar patterns but with different labelings
(define-model Grid5
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
    (interp-fun R (match (
        (($0 $1) true) (($1 $2) true) (($2 $3) true)
        (($3 $4) true) (($4 $5) true) (($5 $6) true)
        (($6 $7) true) (($7 $8) true) (($8 $9) true)
        (($9 $10) true) (($10 $0) true)
        (($2 $5) true) (($3 $6) true) (($4 $7) true)
        (($5 $8) true) (($6 $9) true) (($7 $10) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($2) true) (($5) true) (($8) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true) (($1) true) (($9) true) (($10) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true) (($4) true) (($6) true) (($7) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Grid6-Grid10 (similar structure, different labelings)
(define-model Grid6
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
    (interp-fun R (match (
        (($0 $1) true) (($1 $2) true) (($2 $3) true)
        (($3 $4) true) (($4 $5) true) (($5 $6) true)
        (($6 $7) true) (($7 $8) true) (($8 $9) true)
        (($9 $10) true) (($10 $0) true)
        (($0 $4) true) (($1 $7) true) (($2 $8) true)
        (($3 $9) true) (($4 $10) true) (($5 $1) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true) (($5) true) (($10) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($2) true) (($4) true) (($6) true) (($8) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true) (($3) true) (($7) true) (($9) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Grid7
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10))
    (interp-fun R (match (
        (($0 $1) true) (($1 $2) true) (($2 $3) true)
        (($3 $4) true) (($4 $5) true) (($5 $6) true)
        (($6 $7) true) (($7 $8) true) (($8 $9) true)
        (($9 $10) true) (($10 $0) true)
        (($1 $4) true) (($2 $5) true) (($3 $6) true)
        (($7 $10) true) (($8 $1) true) (($9 $2) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($1) true) (($6) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true) (($2) true) (($4) true) (($8) true) (($10) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true) (($5) true) (($7) true) (($9) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)



(declare-term-types
 ((Bool 0))
 (((a) (b) (c) (v)
   (modal.not Bool)
   (modal.and Bool Bool)
   (modal.or Bool Bool)
   (modal.box Bool)
   (modal.dia Bool))))


(define-finite ModalAsp
  (record (model Model) (node U))
  :where true)

(define-semantics
  ((Modal.Sem (phi Bool) (asp ModalAsp)))
  ((match phi
     ((a) (a (node asp)))
     ((b) (b (node asp)))
     ((c) (c (node asp)))
     ((v) (v (node asp)))
     ((modal.not x) (dual (Modal.Sem x asp)))
     ((modal.and x y) (and (Modal.Sem x asp) (Modal.Sem y asp)))
     ((modal.or x y) (or (Modal.Sem x asp) (Modal.Sem y asp)))
     ((modal.box x)
      (forall ((next (domain (model asp))))
        (or (not (R (node asp) next))
            (Modal.Sem x (ModalAsp (model asp) next)))))
     ((modal.dia x)
      (exists ((next (domain (model asp))))
        (and (R (node asp) next)
             (Modal.Sem x (ModalAsp (model asp) next))))))))

(synth-fun phi () Bool
    ((Start Bool))
    (
        (Start Bool
            (
                a b c
                (modal.not Start)
                (modal.and Start Start)
                (modal.or Start Start)
                (modal.box Start)
                (modal.dia Start)
            )
        )
    )
)

(constraint (Modal.Sem phi (ModalAsp Grid1 $0)))
(constraint (Modal.Sem phi (ModalAsp Grid2 $0)))
(constraint (Modal.Sem phi (ModalAsp Grid3 $0)))
(constraint (Modal.Sem phi (ModalAsp Grid4 $0)))
(constraint (not (Modal.Sem phi (ModalAsp Grid5 $0))))
(constraint (not (Modal.Sem phi (ModalAsp Grid6 $0))))
(constraint (not (Modal.Sem phi (ModalAsp Grid7 $0))))

(check-synth)