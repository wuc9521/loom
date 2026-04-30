(require "../theories/so/90.sl")
(require "../../common/models/so/90.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_90_P1 0 (size (Pos so_90_P1)))))
(constraint (RegLan.Sem r (ReAsp so_90_P2 0 (size (Pos so_90_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_90_N1 0 (size (Pos so_90_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_90_N2 0 (size (Pos so_90_N2))))))

(check-synth)
