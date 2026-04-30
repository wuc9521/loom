(require "../theories/so/58.sl")
(require "../../common/models/so/58.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_58_P1 0 (size (Pos so_58_P1)))))
(constraint (RegLan.Sem r (ReAsp so_58_P2 0 (size (Pos so_58_P2)))))
(constraint (RegLan.Sem r (ReAsp so_58_P3 0 (size (Pos so_58_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_58_N1 0 (size (Pos so_58_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_58_N2 0 (size (Pos so_58_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_58_N3 0 (size (Pos so_58_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_58_N4 0 (size (Pos so_58_N4))))))

(check-synth)
