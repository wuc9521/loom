(require "../theories/so/101.sl")
(require "../../common/models/so/101.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_101_P1 0 (size (Pos so_101_P1)))))
(constraint (RegLan.Sem r (ReAsp so_101_P2 0 (size (Pos so_101_P2)))))
(constraint (RegLan.Sem r (ReAsp so_101_P3 0 (size (Pos so_101_P3)))))
(constraint (RegLan.Sem r (ReAsp so_101_P4 0 (size (Pos so_101_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_101_N1 0 (size (Pos so_101_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_101_N2 0 (size (Pos so_101_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_101_N3 0 (size (Pos so_101_N3))))))

(check-synth)
