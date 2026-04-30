(require "../theories/so/31.sl")
(require "../../common/models/so/31.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_31_P1 0 (size (Pos so_31_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_31_N1 0 (size (Pos so_31_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_31_N2 0 (size (Pos so_31_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_31_N3 0 (size (Pos so_31_N3))))))

(check-synth)
