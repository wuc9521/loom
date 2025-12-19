;; ========== BENCHMARK 2: Tree Structures (8 models) ==========
;; Testing binary tree and more complex tree structures

(set-logic PML)
(declare-sort U 0)
(declare-fun I () U)
(declare-fun R (U U) Bool)
(declare-fun a (U) Bool)
(declare-fun b (U) Bool)
(declare-fun c (U) Bool)

;; Tree1: Complete binary tree depth 3
(define-model Tree1
    (define-univ U ($0 $1 $2 $3 $4 $5 $6))
    (interp-fun R (match (
        (($0 $1) true)  ;; root to left
        (($0 $2) true)  ;; root to right
        (($1 $3) true)  ;; left to left-left
        (($1 $4) true)  ;; left to left-right
        (($2 $5) true)  ;; right to right-left
        (($2 $6) true)  ;; right to right-right
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true)
        (($2) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true)
        (($4) true)
        (($5) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Tree2-8: Similar tree structures with different labelings
(define-model Tree2
    (define-univ U ($0 $1 $2 $3 $4 $5 $6))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($1 $3) true)
        (($1 $4) true)
        (($2 $5) true)
        (($2 $6) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($3) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($4) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($2) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Tree3
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($1 $3) true)
        (($1 $4) true)
        (($2 $5) true)
        (($2 $6) true)
        (($3 $7) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        (($7) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true)
        (($3) true)
        (($4) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Tree4
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($1 $3) true)
        (($1 $4) true)
        (($2 $5) true)
        (($2 $6) true)
        (($4 $7) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($4) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($3) true)
        (($7) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($2) true)
        (($5) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

;; Models Tree5-Tree8 with different structures
(define-model Tree5
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)  ;; ternary branch
        (($1 $4) true)
        (($1 $5) true)
        (($2 $6) true)
        (($3 $7) true)
        (($3 $8) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($0) true)
        (($4) true)
        (($8) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($1) true)
        (($2) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true)
        (($5) true)
        (($7) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Tree6
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)
        (($1 $4) true)
        (($1 $5) true)
        (($2 $6) true)
        (($3 $7) true)
        (($3 $8) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($2) true)
        (($5) true)
        (($7) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($1) true)
        (($8) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($3) true)
        (($4) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Tree7
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)
        (($1 $4) true)
        (($1 $5) true)
        (($2 $6) true)
        (($3 $7) true)
        (($3 $8) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($1) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($4) true)
        (($7) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($2) true)
        (($3) true)
        (($5) true)
        (($8) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(define-model Tree8
    (define-univ U ($0 $1 $2 $3 $4 $5 $6 $7 $8))
    (interp-fun R (match (
        (($0 $1) true)
        (($0 $2) true)
        (($0 $3) true)
        (($1 $4) true)
        (($1 $5) true)
        (($2 $6) true)
        (($3 $7) true)
        (($3 $8) true)
        ((_ _) false)
    )))
    (interp-fun a (match (
        (($3) true)
        (($4) true)
        (($_) false)
    )))
    (interp-fun b (match (
        (($0) true)
        (($2) true)
        (($5) true)
        (($6) true)
        ((_) false)
    )))
    (interp-fun c (match (
        (($1) true)
        (($7) true)
        (($8) true)
        ((_) false)
    )))
    (interp-fun I ($0))
    (set-init I)
    (set-tran R)
)

(synth-fun phi2 () Bool
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

(constraint (models Tree1 phi2))
(constraint (models Tree2 phi2))
(constraint (models Tree3 phi2))
(constraint (models Tree4 phi2))
(constraint (not (models Tree5 phi2)))
(constraint (not (models Tree6 phi2)))
(constraint (not (models Tree7 phi2)))
(constraint (not (models Tree8 phi2)))

(check-synth)