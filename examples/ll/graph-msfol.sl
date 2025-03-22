(set-logic FOL)

; define a ranked signature: only declare without concrete definitions.
(define-signature Sigma
  (declare-sort U 0)         
  (declare-sort V 0)
  (declare-sort W 1)
  (declare-fun c1 () U)       ; const s: U
  (declare-fun c2 () U)       ; const t: U
  (declare-fun E (U U) Bool) ; pred  E: U × U -> Bool
)

; define a model by: specifying the domain + giving interpretations to everything in the signature. automatically check types.

(define-model G1 :extends Sigma
  (define-universe 
  	(U {a b c d})
	  ; if there are other sorts in the signature add here. 
  ) ; M1 的 U = {a, b, c, d}
  (interpret s a)             ; s := a
  (interpret t d)             ; t := d
  (interpret E
    (lambda (x y) (or (= x b) (= y c)))) ; E := such a function of U^2 -> Bool
)


; 1. additionally provide lambda syntax. automatically check types. minimal design. 
; 2. universe should formally be a (possibly partial) map: \N -> Objects
;   {an} is a syntax sugar for the map (n mapsto an)
;   if it's infinite, provide it as a generating function like ``lambda (n) n''
; 3. additionally add ``interpret'' keyword.
; 4. (declare-pred <name> ____) is an alias for (declare-fun <name> ____ Bool)
; theory ~= signature + axioms


; (define-model G2 :extends Sigma
;  ......
; )
