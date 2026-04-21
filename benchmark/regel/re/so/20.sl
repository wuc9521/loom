(require "../theories/so/20.sl")
(require "../../common/models/so/20.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_20_P1 0 (size (Pos so_20_P1)))))
(constraint (RegLan.Sem r (ReAsp so_20_P2 0 (size (Pos so_20_P2)))))
(constraint (RegLan.Sem r (ReAsp so_20_P3 0 (size (Pos so_20_P3)))))
(constraint (RegLan.Sem r (ReAsp so_20_P4 0 (size (Pos so_20_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_20_N1 0 (size (Pos so_20_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_20_N2 0 (size (Pos so_20_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_20_N3 0 (size (Pos so_20_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_20_N4 0 (size (Pos so_20_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_20_N5 0 (size (Pos so_20_N5))))))

(check-synth)
