(require "../theories/so/84.sl")
(require "../../common/models/so/84.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_84_P1 0 (size (Pos so_84_P1)))))
(constraint (RegLan.Sem r (ReAsp so_84_P2 0 (size (Pos so_84_P2)))))
(constraint (RegLan.Sem r (ReAsp so_84_P3 0 (size (Pos so_84_P3)))))
(constraint (RegLan.Sem r (ReAsp so_84_P4 0 (size (Pos so_84_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_84_N1 0 (size (Pos so_84_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_84_N2 0 (size (Pos so_84_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_84_N3 0 (size (Pos so_84_N3))))))

(check-synth)
