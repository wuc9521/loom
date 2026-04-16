;; ========== BENCHMARK 1: Chain Structures (6 models) ==========
;; Testing linear chain structures with different lengths and properties

(set-logic ALL)
(declare-sort U 0)
(declare-fun I () U)
(declare-fun R (U U) Bool)
(declare-fun a (U) Bool)
(declare-fun b (U) Bool)
(declare-fun c (U) Bool)

;; Chain1: a->b->c->a->b (length 5)
(define-model Chain1
    (define-univ U ($0 $1 $2 $3 $4))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $0) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        (($3) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

;; Chain2: Similar structure but different labeling
(define-model Chain2
    (define-univ U ($0 $1 $2 $3 $4))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $0) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($1) true)
        (($2) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

;; Chain3: Linear chain (no cycle)
(define-model Chain3
    (define-univ U ($0 $1 $2 $3 $4 $5))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $5) true)
        (($5 $5) true) ;; self-loop at end
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        (($2) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true)
        (($3) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

;; Chain4-6: Similar structures with variations
(define-model Chain4
    (define-univ U ($0 $1 $2 $3 $4 $5))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $5) true)
        (($5 $0) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($2) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($3) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

(define-model Chain5
    (define-univ U ($0 $1 $2 $3 $4 $5))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $5) true)
        (($5 $5) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($1) true)
        (($3) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($2) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)

(define-model Chain6
    (define-univ U ($0 $1 $2 $3 $4 $5 $6))
    (interp-fun R (match (
        (($0 $1) true)
        (($1 $2) true)
        (($2 $3) true)
        (($3 $4) true)
        (($4 $5) true)
        (($5 $6) true)
        (($6 $6) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($2) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($3) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
)


(declare-term-types
 ((Modal 0))
 (((a) (b) (c) (v)
   (modal.not Modal)
   (modal.and Modal Modal)
   (modal.or Modal Modal)
   (modal.box Modal)
   (modal.dia Modal))))


(define-finite ModalAsp
  (record (model Model) (node U))
  :where true)

(define-semantics
  ((Modal.Sem (phi Modal) (asp ModalAsp)))
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

(synth-fun phi1 () Modal
    ((Start Modal))
    (
        (Start Modal
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

(constraint (Modal.Sem phi1 (ModalAsp Chain1 $0)))
(constraint (Modal.Sem phi1 (ModalAsp Chain2 $0)))
(constraint (Modal.Sem phi1 (ModalAsp Chain3 $0)))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain4 $0))))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain5 $0))))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain6 $0))))

(check-synth)