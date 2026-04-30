(require "./theories/main.sl")
(require "../common/models/no02.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (size (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p_01 0 (size (Pos p_01)))))
(constraint (RegLan.Sem r (ReAsp p__01 0 (size (Pos p__01)))))
(constraint (not (RegLan.Sem r (ReAsp n00 0 (size (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n10 0 (size (Pos n10))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n_?00 0 (size (Pos n_?00))))))
(constraint (not (RegLan.Sem r (ReAsp n_?10 0 (size (Pos n_?10))))))
(constraint (not (RegLan.Sem r (ReAsp n_?11 0 (size (Pos n_?11))))))

(check-synth)
