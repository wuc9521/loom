(require "../theories/so/77.sl")
(require "../../common/models/so/77.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_77_P1 0 (size (Pos so_77_P1)))))
(constraint (RegLan.Sem r (ReAsp so_77_P2 0 (size (Pos so_77_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_77_N1 0 (size (Pos so_77_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_77_N2 0 (size (Pos so_77_N2))))))

(check-synth)
