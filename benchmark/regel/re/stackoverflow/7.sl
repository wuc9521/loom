(require "../theories/so/7.sl")
(require "../../common/models/so/7.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_7_P1 0 (size (Pos so_7_P1)))))
(constraint (RegLan.Sem r (ReAsp so_7_P2 0 (size (Pos so_7_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_7_N1 0 (size (Pos so_7_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_7_N2 0 (size (Pos so_7_N2))))))

(check-synth)
