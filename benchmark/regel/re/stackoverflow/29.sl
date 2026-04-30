(require "../theories/so/29.sl")
(require "../../common/models/so/29.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_29_P1 0 (size (Pos so_29_P1)))))
(constraint (RegLan.Sem r (ReAsp so_29_P2 0 (size (Pos so_29_P2)))))
(constraint (RegLan.Sem r (ReAsp so_29_P3 0 (size (Pos so_29_P3)))))
(constraint (RegLan.Sem r (ReAsp so_29_P4 0 (size (Pos so_29_P4)))))
(constraint (RegLan.Sem r (ReAsp so_29_P5 0 (size (Pos so_29_P5)))))
(constraint (RegLan.Sem r (ReAsp so_29_P6 0 (size (Pos so_29_P6)))))
(constraint (RegLan.Sem r (ReAsp so_29_P7 0 (size (Pos so_29_P7)))))
(constraint (RegLan.Sem r (ReAsp so_29_P8 0 (size (Pos so_29_P8)))))
(constraint (not (RegLan.Sem r (ReAsp so_29_N1 0 (size (Pos so_29_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_29_N2 0 (size (Pos so_29_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_29_N3 0 (size (Pos so_29_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_29_N4 0 (size (Pos so_29_N4))))))

(check-synth)
