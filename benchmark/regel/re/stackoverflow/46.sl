(require "../theories/so/46.sl")
(require "../../common/models/so/46.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_46_P1 0 (size (Pos so_46_P1)))))
(constraint (RegLan.Sem r (ReAsp so_46_P2 0 (size (Pos so_46_P2)))))
(constraint (RegLan.Sem r (ReAsp so_46_P3 0 (size (Pos so_46_P3)))))
(constraint (RegLan.Sem r (ReAsp so_46_P4 0 (size (Pos so_46_P4)))))
(constraint (RegLan.Sem r (ReAsp so_46_P5 0 (size (Pos so_46_P5)))))
(constraint (RegLan.Sem r (ReAsp so_46_P6 0 (size (Pos so_46_P6)))))
(constraint (not (RegLan.Sem r (ReAsp so_46_N1 0 (size (Pos so_46_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_46_N2 0 (size (Pos so_46_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_46_N3 0 (size (Pos so_46_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_46_N4 0 (size (Pos so_46_N4))))))

(check-synth)
