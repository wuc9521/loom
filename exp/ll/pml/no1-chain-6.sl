;; ========== BENCHMARK 1: Chain Structures (6 models) ==========
;; Testing linear chain structures with different lengths and properties

(set-logic PML)
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
    (set-init I)
    (set-tran R)
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
    (set-init I)
    (set-tran R)
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
    (set-init I)
    (set-tran R)
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
    (set-init I)
    (set-tran R)
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
    (set-init I)
    (set-tran R)
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
    (set-init I)
    (set-tran R)
)

(synth-fun phi1 () Bool
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

(constraint (models Chain1 phi1))
(constraint (models Chain2 phi1))
(constraint (models Chain3 phi1))
(constraint (not (models Chain4 phi1)))
(constraint (not (models Chain5 phi1)))
(constraint (not (models Chain6 phi1)))

(check-synth)