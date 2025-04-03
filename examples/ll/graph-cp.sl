; .sl: synth lib;
(set-logic FOL) ; only consider FOL so far.

(declare-sort U 0) ; declare-sort <name> <arity>. By default only ``='' is allowed on the sort.

; TODO: only allow arity == 0 for now.
; define a ranked signature: only declare without concrete definitions.
(define-signature Sigma
(declare-fun s () U)       ; const s: U
(declare-fun t () U)       ; const t: U
(declare-fun E (U U) Bool) ; pred  E: U × U -> Bool
)

; define a model by: specifying the domain + giving interpretations to everything in the signature. automatically check types.

(define-model G1 Sigma
(define-universe
	(U {a b c d})         ; if there are other sorts in the signature add here.
)                      ;TODO: name collision
(interpret s a)        ; s := a
(interpret t d)        ; t := d
(interpret E
(match
	((a a) true)
	((b a) true)
	((c a) true)
	((_ _) false)
)
)                      ; E := such a function of U^2 -> Bool
; TODO: how to interpret ``='' on symbolic values?
; We know ``='' is an equivalence relation
; (interpret E
;   (a, b) (a, c)) ; E := such a function of U^2 -> Bool
)
;TODO: explicitly define functions and relations by enumerating?
;TODO: define the complement of a predicate; define set operations.

; Paul's paper handles grammars.

; (define-grammar <name>
;   ((<non-terminal> <sort>)    ; define the type of the non-terminal
;    (<non-terminal> <expr1> <expr2> ... <exprN>))) ; define generation rules

(define-grammar G
((Term U)                                                               ; 定义 Term 类型，它是 sort `U` 中的变量
(Term s t x y)                                                          ; Term 只能是 s, t, x, y 中的某个变量
(Fml Bool)                                                              ; 定义 Fml 类型，它是 Bool 类型的公式
(Fml (and Fml Fml) (or Fml Fml) (not Fml) (E Term Term) (= Term Term)))
)

; This syntax refers to SemGuS.
; TODO: (k) This should be a grammar thing.
; (synth-form f Sigma (x1 U) (x2 U))
(synth-form f Sigma G)
; synthesize a Sigma-formula f using only variables x1, x2 (can be reused)
; of sort U in signature Sigma.
(constraint (models G1 f)) ; f holds in G1
(check-synth)              ; check the synthesis problem.

; TODO: Question:
; 1. How to define a infinite universe (is that necessary?)
; 2. Where should we constaint the usage of variables? Let's say we only allow the return formula only uses 1 variable,
;     where should I impose this constraint?
;     Should it be set in the logic?

; 1. additionally provide ``lambda'' syntax (distinguish from define-fun and decalre fun). automatically check types. minimal design.
; 2. universe should formally be a (possibly partial) map: \Nat -> \Objects
;   {an} is a syntax sugar for the map (n mapsto an)
;   if it's infinite, provide it as a generating function like ``lambda (n) n''
; 3. additionally add ``interpret'' keyword.
; theory ~~ signature + axioms

; TODO: I don't know if this design is reasonable. It feels reasonable from a math perspective, but idk if solvers like it.
; There are a lot of things remaining to be explicitly defined in the logic.
; 1. axioms of FOL theory. Like transitivity, reflexivity, symmetry of ``=''.
; 2. what does ``models'' mean?
