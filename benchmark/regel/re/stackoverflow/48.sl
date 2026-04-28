(require "../theories/so/48.sl")
(require "../../common/models/so/48.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_48_P1 0 (size (Pos so_48_P1)))))
(constraint (RegLan.Sem r (ReAsp so_48_P2 0 (size (Pos so_48_P2)))))
(constraint (RegLan.Sem r (ReAsp so_48_P3 0 (size (Pos so_48_P3)))))
(constraint (RegLan.Sem r (ReAsp so_48_P4 0 (size (Pos so_48_P4)))))
(constraint (RegLan.Sem r (ReAsp so_48_P5 0 (size (Pos so_48_P5)))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N1 0 (size (Pos so_48_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N2 0 (size (Pos so_48_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N3 0 (size (Pos so_48_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N4 0 (size (Pos so_48_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N5 0 (size (Pos so_48_N5))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N6 0 (size (Pos so_48_N6))))))
(constraint (not (RegLan.Sem r (ReAsp so_48_N7 0 (size (Pos so_48_N7))))))

(check-synth)
