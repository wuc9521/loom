(require "../theories/so/75.sl")
(require "../../common/models/so/75.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_75_P1 0 (size (Pos so_75_P1)))))
(constraint (RegLan.Sem r (ReAsp so_75_P2 0 (size (Pos so_75_P2)))))
(constraint (RegLan.Sem r (ReAsp so_75_P3 0 (size (Pos so_75_P3)))))
(constraint (RegLan.Sem r (ReAsp so_75_P4 0 (size (Pos so_75_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_75_N1 0 (size (Pos so_75_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_75_N2 0 (size (Pos so_75_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_75_N3 0 (size (Pos so_75_N3))))))

(check-synth)
