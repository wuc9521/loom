(require "../theories/so/50.sl")
(require "../../common/models/so/50.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_50_P1 0 (size (Pos so_50_P1)))))
(constraint (RegLan.Sem r (ReAsp so_50_P2 0 (size (Pos so_50_P2)))))
(constraint (RegLan.Sem r (ReAsp so_50_P3 0 (size (Pos so_50_P3)))))
(constraint (RegLan.Sem r (ReAsp so_50_P4 0 (size (Pos so_50_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_50_N1 0 (size (Pos so_50_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_50_N2 0 (size (Pos so_50_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_50_N3 0 (size (Pos so_50_N3))))))

(check-synth)
