(require "../theories/so/88.sl")
(require "../../common/models/so/88.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_88_P1 0 (size (Pos so_88_P1)))))
(constraint (RegLan.Sem r (ReAsp so_88_P2 0 (size (Pos so_88_P2)))))
(constraint (RegLan.Sem r (ReAsp so_88_P3 0 (size (Pos so_88_P3)))))
(constraint (RegLan.Sem r (ReAsp so_88_P4 0 (size (Pos so_88_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_88_N1 0 (size (Pos so_88_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_88_N2 0 (size (Pos so_88_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_88_N3 0 (size (Pos so_88_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_88_N4 0 (size (Pos so_88_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_88_N5 0 (size (Pos so_88_N5))))))

(check-synth)
