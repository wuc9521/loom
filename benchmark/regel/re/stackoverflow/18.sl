(require "../theories/so/18.sl")
(require "../../common/models/so/18.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_18_P1 0 (size (Pos so_18_P1)))))
(constraint (RegLan.Sem r (ReAsp so_18_P2 0 (size (Pos so_18_P2)))))
(constraint (RegLan.Sem r (ReAsp so_18_P3 0 (size (Pos so_18_P3)))))
(constraint (RegLan.Sem r (ReAsp so_18_P4 0 (size (Pos so_18_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_18_N1 0 (size (Pos so_18_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_18_N2 0 (size (Pos so_18_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_18_N3 0 (size (Pos so_18_N3))))))

(check-synth)
