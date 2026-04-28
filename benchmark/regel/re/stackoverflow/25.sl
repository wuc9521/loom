(require "../theories/so/25.sl")
(require "../../common/models/so/25.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_25_P1 0 (size (Pos so_25_P1)))))
(constraint (RegLan.Sem r (ReAsp so_25_P2 0 (size (Pos so_25_P2)))))
(constraint (RegLan.Sem r (ReAsp so_25_P3 0 (size (Pos so_25_P3)))))
(constraint (RegLan.Sem r (ReAsp so_25_P4 0 (size (Pos so_25_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_25_N1 0 (size (Pos so_25_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_25_N2 0 (size (Pos so_25_N2))))))

(check-synth)
