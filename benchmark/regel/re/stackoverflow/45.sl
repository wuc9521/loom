(require "../theories/so/45.sl")
(require "../../common/models/so/45.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_45_P1 0 (size (Pos so_45_P1)))))
(constraint (RegLan.Sem r (ReAsp so_45_P2 0 (size (Pos so_45_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_45_N1 0 (size (Pos so_45_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_45_N2 0 (size (Pos so_45_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_45_N3 0 (size (Pos so_45_N3))))))

(check-synth)
