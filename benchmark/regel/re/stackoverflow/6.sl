(require "../theories/so/6.sl")
(require "../../common/models/so/6.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_6_P1 0 (size (Pos so_6_P1)))))
(constraint (RegLan.Sem r (ReAsp so_6_P2 0 (size (Pos so_6_P2)))))
(constraint (RegLan.Sem r (ReAsp so_6_P3 0 (size (Pos so_6_P3)))))
(constraint (RegLan.Sem r (ReAsp so_6_P4 0 (size (Pos so_6_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N1 0 (size (Pos so_6_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N2 0 (size (Pos so_6_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N3 0 (size (Pos so_6_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N4 0 (size (Pos so_6_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N5 0 (size (Pos so_6_N5))))))
(constraint (not (RegLan.Sem r (ReAsp so_6_N6 0 (size (Pos so_6_N6))))))

(check-synth)
