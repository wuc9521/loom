(require "../theories/so/120.sl")
(require "../../common/models/so/120.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_120_P1 0 (size (Pos so_120_P1)))))
(constraint (RegLan.Sem r (ReAsp so_120_P2 0 (size (Pos so_120_P2)))))
(constraint (RegLan.Sem r (ReAsp so_120_P3 0 (size (Pos so_120_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_120_N1 0 (size (Pos so_120_N1))))))

(check-synth)
