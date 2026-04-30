(require "../theories/so/102.sl")
(require "../../common/models/so/102.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_102_P1 0 (size (Pos so_102_P1)))))
(constraint (RegLan.Sem r (ReAsp so_102_P2 0 (size (Pos so_102_P2)))))
(constraint (RegLan.Sem r (ReAsp so_102_P3 0 (size (Pos so_102_P3)))))
(constraint (RegLan.Sem r (ReAsp so_102_P4 0 (size (Pos so_102_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_102_N1 0 (size (Pos so_102_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_102_N2 0 (size (Pos so_102_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_102_N3 0 (size (Pos so_102_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_102_N4 0 (size (Pos so_102_N4))))))

(check-synth)
