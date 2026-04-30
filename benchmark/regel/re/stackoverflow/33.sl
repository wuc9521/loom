(require "../theories/so/33.sl")
(require "../../common/models/so/33.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_33_P1 0 (size (Pos so_33_P1)))))
(constraint (RegLan.Sem r (ReAsp so_33_P2 0 (size (Pos so_33_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_33_N1 0 (size (Pos so_33_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_33_N2 0 (size (Pos so_33_N2))))))

(check-synth)
