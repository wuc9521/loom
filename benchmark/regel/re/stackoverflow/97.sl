(require "../theories/so/97.sl")
(require "../../common/models/so/97.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_97_P1 0 (size (Pos so_97_P1)))))
(constraint (RegLan.Sem r (ReAsp so_97_P2 0 (size (Pos so_97_P2)))))
(constraint (RegLan.Sem r (ReAsp so_97_P3 0 (size (Pos so_97_P3)))))
(constraint (RegLan.Sem r (ReAsp so_97_P4 0 (size (Pos so_97_P4)))))
(constraint (RegLan.Sem r (ReAsp so_97_P5 0 (size (Pos so_97_P5)))))
(constraint (RegLan.Sem r (ReAsp so_97_P6 0 (size (Pos so_97_P6)))))
(constraint (not (RegLan.Sem r (ReAsp so_97_N1 0 (size (Pos so_97_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_97_N2 0 (size (Pos so_97_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_97_N3 0 (size (Pos so_97_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_97_N4 0 (size (Pos so_97_N4))))))

(check-synth)
