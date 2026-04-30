(require "../theories/so/40.sl")
(require "../../common/models/so/40.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_40_P1 0 (size (Pos so_40_P1)))))
(constraint (RegLan.Sem r (ReAsp so_40_P2 0 (size (Pos so_40_P2)))))
(constraint (RegLan.Sem r (ReAsp so_40_P3 0 (size (Pos so_40_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_40_N1 0 (size (Pos so_40_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_40_N2 0 (size (Pos so_40_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_40_N3 0 (size (Pos so_40_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_40_N4 0 (size (Pos so_40_N4))))))

(check-synth)
