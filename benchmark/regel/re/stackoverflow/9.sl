(require "../theories/so/9.sl")
(require "../../common/models/so/9.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_9_P1 0 (size (Pos so_9_P1)))))
(constraint (RegLan.Sem r (ReAsp so_9_P2 0 (size (Pos so_9_P2)))))
(constraint (RegLan.Sem r (ReAsp so_9_P3 0 (size (Pos so_9_P3)))))
(constraint (RegLan.Sem r (ReAsp so_9_P4 0 (size (Pos so_9_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_9_N1 0 (size (Pos so_9_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_9_N2 0 (size (Pos so_9_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_9_N3 0 (size (Pos so_9_N3))))))

(check-synth)
