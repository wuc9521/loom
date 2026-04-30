(require "../theories/so/15.sl")
(require "../../common/models/so/15.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_15_P1 0 (size (Pos so_15_P1)))))
(constraint (RegLan.Sem r (ReAsp so_15_P2 0 (size (Pos so_15_P2)))))
(constraint (RegLan.Sem r (ReAsp so_15_P3 0 (size (Pos so_15_P3)))))
(constraint (RegLan.Sem r (ReAsp so_15_P4 0 (size (Pos so_15_P4)))))
(constraint (RegLan.Sem r (ReAsp so_15_P5 0 (size (Pos so_15_P5)))))
(constraint (RegLan.Sem r (ReAsp so_15_P6 0 (size (Pos so_15_P6)))))
(constraint (not (RegLan.Sem r (ReAsp so_15_N1 0 (size (Pos so_15_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_15_N2 0 (size (Pos so_15_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_15_N3 0 (size (Pos so_15_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_15_N4 0 (size (Pos so_15_N4))))))

(check-synth)
