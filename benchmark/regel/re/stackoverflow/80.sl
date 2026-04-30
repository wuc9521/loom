(require "../theories/so/80.sl")
(require "../../common/models/so/80.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_80_P1 0 (size (Pos so_80_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_80_N1 0 (size (Pos so_80_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_80_N2 0 (size (Pos so_80_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_80_N3 0 (size (Pos so_80_N3))))))

(check-synth)
