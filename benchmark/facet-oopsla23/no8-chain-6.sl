;; ========== BENCHMARK 1: Chain Structures (6 models) ==========
;; Testing linear chain structures with different lengths and properties
(require "./theories/main.sl")
(set-logic ALL)


;; Chain1: a->v->c->a->v (length 5)
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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
    (interp-fun v (match (
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

(synth-fun phi1 () Modal)

(constraint (Modal.Sem phi1 (ModalAsp Chain1 $0)))
(constraint (Modal.Sem phi1 (ModalAsp Chain2 $0)))
(constraint (Modal.Sem phi1 (ModalAsp Chain3 $0)))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain4 $0))))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain5 $0))))
(constraint (not (Modal.Sem phi1 (ModalAsp Chain6 $0))))

(check-synth)
