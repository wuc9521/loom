(require "../theories/so/60.sl")
(require "../../common/models/so/60.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_60_P1 0 (size (Pos so_60_P1)))))
(constraint (RegLan.Sem r (ReAsp so_60_P2 0 (size (Pos so_60_P2)))))
(constraint (RegLan.Sem r (ReAsp so_60_P3 0 (size (Pos so_60_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_60_N1 0 (size (Pos so_60_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_60_N2 0 (size (Pos so_60_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_60_N3 0 (size (Pos so_60_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_60_N4 0 (size (Pos so_60_N4))))))

(check-synth)
