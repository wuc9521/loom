(require "../theories/so/8.sl")
(require "../../common/models/so/8.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_8_P1 0 (size (Pos so_8_P1)))))
(constraint (RegLan.Sem r (ReAsp so_8_P2 0 (size (Pos so_8_P2)))))
(constraint (RegLan.Sem r (ReAsp so_8_P3 0 (size (Pos so_8_P3)))))
(constraint (RegLan.Sem r (ReAsp so_8_P4 0 (size (Pos so_8_P4)))))
(constraint (RegLan.Sem r (ReAsp so_8_P5 0 (size (Pos so_8_P5)))))
(constraint (RegLan.Sem r (ReAsp so_8_P6 0 (size (Pos so_8_P6)))))
(constraint (RegLan.Sem r (ReAsp so_8_P7 0 (size (Pos so_8_P7)))))
(constraint (RegLan.Sem r (ReAsp so_8_P8 0 (size (Pos so_8_P8)))))
(constraint (not (RegLan.Sem r (ReAsp so_8_N1 0 (size (Pos so_8_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_8_N2 0 (size (Pos so_8_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_8_N3 0 (size (Pos so_8_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_8_N4 0 (size (Pos so_8_N4))))))

(check-synth)
