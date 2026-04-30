(require "../theories/so/42.sl")
(require "../../common/models/so/42.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_42_P1 0 (size (Pos so_42_P1)))))
(constraint (RegLan.Sem r (ReAsp so_42_P2 0 (size (Pos so_42_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_42_N1 0 (size (Pos so_42_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_42_N2 0 (size (Pos so_42_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_42_N3 0 (size (Pos so_42_N3))))))

(check-synth)
