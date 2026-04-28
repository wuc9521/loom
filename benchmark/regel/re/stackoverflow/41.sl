(require "../theories/so/41.sl")
(require "../../common/models/so/41.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_41_P1 0 (size (Pos so_41_P1)))))
(constraint (RegLan.Sem r (ReAsp so_41_P2 0 (size (Pos so_41_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_41_N1 0 (size (Pos so_41_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_41_N2 0 (size (Pos so_41_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_41_N3 0 (size (Pos so_41_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_41_N4 0 (size (Pos so_41_N4))))))

(check-synth)
