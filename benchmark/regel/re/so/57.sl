(require "../theories/so/57.sl")
(require "../../common/models/so/57.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_57_P1 0 (size (Pos so_57_P1)))))
(constraint (RegLan.Sem r (ReAsp so_57_P2 0 (size (Pos so_57_P2)))))
(constraint (RegLan.Sem r (ReAsp so_57_P3 0 (size (Pos so_57_P3)))))
(constraint (RegLan.Sem r (ReAsp so_57_P4 0 (size (Pos so_57_P4)))))
(constraint (RegLan.Sem r (ReAsp so_57_P5 0 (size (Pos so_57_P5)))))
(constraint (not (RegLan.Sem r (ReAsp so_57_N1 0 (size (Pos so_57_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_57_N2 0 (size (Pos so_57_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_57_N3 0 (size (Pos so_57_N3))))))

(check-synth)
