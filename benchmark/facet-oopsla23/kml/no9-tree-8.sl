;; ========== BENCHMARK 2: Tree Structures (8 models) ==========
;; Testing binary tree and more complex tree structures
(require "../theories/main.sl")
(set-logic ALL)

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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
        ((_) false)
    )))
    (interp-fun v (match (
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
)


(synth-fun phi2 () Modal)

(constraint (Modal.Sem phi2 (ModalAsp Tree1 $0)))
(constraint (Modal.Sem phi2 (ModalAsp Tree2 $0)))
(constraint (Modal.Sem phi2 (ModalAsp Tree3 $0)))
(constraint (Modal.Sem phi2 (ModalAsp Tree4 $0)))
(constraint (not (Modal.Sem phi2 (ModalAsp Tree5 $0))))
(constraint (not (Modal.Sem phi2 (ModalAsp Tree6 $0))))
(constraint (not (Modal.Sem phi2 (ModalAsp Tree7 $0))))
(constraint (not (Modal.Sem phi2 (ModalAsp Tree8 $0))))

(check-synth)
