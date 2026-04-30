(require "../theories/so/11.sl")
(require "../../common/models/so/11.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_11_P1 0 (size (Pos so_11_P1)))))
(constraint (RegLan.Sem r (ReAsp so_11_P2 0 (size (Pos so_11_P2)))))
(constraint (RegLan.Sem r (ReAsp so_11_P3 0 (size (Pos so_11_P3)))))
(constraint (RegLan.Sem r (ReAsp so_11_P4 0 (size (Pos so_11_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_11_N1 0 (size (Pos so_11_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_11_N2 0 (size (Pos so_11_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_11_N3 0 (size (Pos so_11_N3))))))

(check-synth)
