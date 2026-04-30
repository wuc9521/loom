(require "../theories/so/12.sl")
(require "../../common/models/so/12.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_12_P1 0 (size (Pos so_12_P1)))))
(constraint (RegLan.Sem r (ReAsp so_12_P2 0 (size (Pos so_12_P2)))))
(constraint (RegLan.Sem r (ReAsp so_12_P3 0 (size (Pos so_12_P3)))))
(constraint (RegLan.Sem r (ReAsp so_12_P4 0 (size (Pos so_12_P4)))))
(constraint (RegLan.Sem r (ReAsp so_12_P5 0 (size (Pos so_12_P5)))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N1 0 (size (Pos so_12_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N2 0 (size (Pos so_12_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N3 0 (size (Pos so_12_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N4 0 (size (Pos so_12_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N5 0 (size (Pos so_12_N5))))))
(constraint (not (RegLan.Sem r (ReAsp so_12_N6 0 (size (Pos so_12_N6))))))

(check-synth)
