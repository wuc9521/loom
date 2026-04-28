(require "../theories/so/10.sl")
(require "../../common/models/so/10.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_10_P1 0 (size (Pos so_10_P1)))))
(constraint (RegLan.Sem r (ReAsp so_10_P2 0 (size (Pos so_10_P2)))))
(constraint (RegLan.Sem r (ReAsp so_10_P3 0 (size (Pos so_10_P3)))))
(constraint (RegLan.Sem r (ReAsp so_10_P4 0 (size (Pos so_10_P4)))))
(constraint (RegLan.Sem r (ReAsp so_10_P5 0 (size (Pos so_10_P5)))))
(constraint (RegLan.Sem r (ReAsp so_10_P6 0 (size (Pos so_10_P6)))))
(constraint (RegLan.Sem r (ReAsp so_10_P7 0 (size (Pos so_10_P7)))))
(constraint (not (RegLan.Sem r (ReAsp so_10_N1 0 (size (Pos so_10_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_10_N2 0 (size (Pos so_10_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_10_N3 0 (size (Pos so_10_N3))))))

(check-synth)
