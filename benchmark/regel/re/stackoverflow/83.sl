(require "../theories/so/83.sl")
(require "../../common/models/so/83.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_83_P1 0 (size (Pos so_83_P1)))))
(constraint (RegLan.Sem r (ReAsp so_83_P2 0 (size (Pos so_83_P2)))))
(constraint (RegLan.Sem r (ReAsp so_83_P3 0 (size (Pos so_83_P3)))))
(constraint (RegLan.Sem r (ReAsp so_83_P4 0 (size (Pos so_83_P4)))))
(constraint (RegLan.Sem r (ReAsp so_83_P5 0 (size (Pos so_83_P5)))))
(constraint (not (RegLan.Sem r (ReAsp so_83_N1 0 (size (Pos so_83_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_83_N2 0 (size (Pos so_83_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_83_N3 0 (size (Pos so_83_N3))))))

(check-synth)
