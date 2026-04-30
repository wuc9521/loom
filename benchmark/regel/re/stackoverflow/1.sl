(require "../theories/so/1.sl")
(require "../../common/models/so/1.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_1_P1 0 (size (Pos so_1_P1)))))
(constraint (RegLan.Sem r (ReAsp so_1_P2 0 (size (Pos so_1_P2)))))
(constraint (RegLan.Sem r (ReAsp so_1_P3 0 (size (Pos so_1_P3)))))
(constraint (RegLan.Sem r (ReAsp so_1_P4 0 (size (Pos so_1_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_1_N1 0 (size (Pos so_1_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_1_N2 0 (size (Pos so_1_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_1_N3 0 (size (Pos so_1_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_1_N4 0 (size (Pos so_1_N4))))))

(check-synth)
