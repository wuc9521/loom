(require "./theories/main.sl")
(require "../common/models/no15.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p__ 0 (size (Pos p__)))))
(constraint (RegLan.Sem r (ReAsp p___ 0 (size (Pos p___)))))
(constraint (RegLan.Sem r (ReAsp p____ 0 (size (Pos p____)))))
(constraint (not (RegLan.Sem r (ReAsp n0 0 (size (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))

(check-synth)
