(require "../theories/so/38.sl")
(require "../../common/models/so/38.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_38_P1 0 (size (Pos so_38_P1)))))
(constraint (RegLan.Sem r (ReAsp so_38_P2 0 (size (Pos so_38_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_38_N1 0 (size (Pos so_38_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_38_N2 0 (size (Pos so_38_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_38_N3 0 (size (Pos so_38_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_38_N4 0 (size (Pos so_38_N4))))))

(check-synth)
