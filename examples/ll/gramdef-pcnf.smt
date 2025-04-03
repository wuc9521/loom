; Generate all FOL S-formula, which are Prenex Conjunctive Normal Form (PCNF):
; forall x1 x2 ... xn. (phi1 and phi2 and ... and phim)
; where phi1, phi2, ..., phim are disjunctions of literals


(synth-fun 
    :signature S
    :override (Start)
    myform () Bool
    (
        (Start Bool (
            PNF
            CNF
        ))
        
        (PNF Bool (
            CNF
            (forall (Variable U) PNF)
            (exists (Variable U) PNF)
        ))
        
        (CNF Bool (
            Clause
            (and CNF Clause)
        ))
        
        (Clause Bool (
            Literal
            (or Clause Literal)
        ))
        
        (Literal Bool (
            AtomicFormula
            (not AtomicFormula)
        ))
        
        (AtomicFormula Bool (
            true
            false
            (E UTerm UTerm)
            (F UTerm UTerm)
            (G UTerm UTerm)
        ))
        
        (UTerm U (
            s
            t
            (Variable U)
            (Constant U)
        ))
    )
)