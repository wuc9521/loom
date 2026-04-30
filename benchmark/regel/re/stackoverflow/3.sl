(require "../theories/so/3.sl")
(require "../../common/models/so/3.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_3_P1 0 (size (Pos so_3_P1)))))
(constraint (RegLan.Sem r (ReAsp so_3_P2 0 (size (Pos so_3_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_3_N1 0 (size (Pos so_3_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_3_N2 0 (size (Pos so_3_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_3_N3 0 (size (Pos so_3_N3))))))

(check-synth)
