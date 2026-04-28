(require "../theories/so/74.sl")
(require "../../common/models/so/74.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_74_P1 0 (size (Pos so_74_P1)))))
(constraint (RegLan.Sem r (ReAsp so_74_P2 0 (size (Pos so_74_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_74_N1 0 (size (Pos so_74_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_74_N2 0 (size (Pos so_74_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_74_N3 0 (size (Pos so_74_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_74_N4 0 (size (Pos so_74_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_74_N5 0 (size (Pos so_74_N5))))))

(check-synth)
