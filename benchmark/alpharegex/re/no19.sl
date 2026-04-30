(require "./theories/main.sl")
(require "../common/models/no19.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p_ 0 (size (Pos p_)))))
(constraint (RegLan.Sem r (ReAsp p__ 0 (size (Pos p__)))))
(constraint (RegLan.Sem r (ReAsp p0__ 0 (size (Pos p0__)))))
(constraint (RegLan.Sem r (ReAsp p_1_ 0 (size (Pos p_1_)))))
(constraint (RegLan.Sem r (ReAsp p__1 0 (size (Pos p__1)))))
(constraint (RegLan.Sem r (ReAsp p_101 0 (size (Pos p_101)))))
(constraint (RegLan.Sem r (ReAsp p0_01_ 0 (size (Pos p0_01_)))))
(constraint (RegLan.Sem r (ReAsp p__1_1_1 0 (size (Pos p__1_1_1)))))
(constraint (not (RegLan.Sem r (ReAsp n100 0 (size (Pos n100))))))
(constraint (not (RegLan.Sem r (ReAsp n?100 0 (size (Pos n?100))))))
(constraint (not (RegLan.Sem r (ReAsp n100? 0 (size (Pos n100?))))))
(constraint (not (RegLan.Sem r (ReAsp n?100? 0 (size (Pos n?100?))))))

(check-synth)
