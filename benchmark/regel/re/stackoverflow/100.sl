(require "../theories/so/100.sl")
(require "../../common/models/so/100.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_100_P1 0 (size (Pos so_100_P1)))))
(constraint (RegLan.Sem r (ReAsp so_100_P2 0 (size (Pos so_100_P2)))))
(constraint (RegLan.Sem r (ReAsp so_100_P3 0 (size (Pos so_100_P3)))))
(constraint (RegLan.Sem r (ReAsp so_100_P4 0 (size (Pos so_100_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_100_N1 0 (size (Pos so_100_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_100_N2 0 (size (Pos so_100_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_100_N3 0 (size (Pos so_100_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_100_N4 0 (size (Pos so_100_N4))))))

(check-synth)
