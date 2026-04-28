(require "../theories/so/51.sl")
(require "../../common/models/so/51.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_51_P1 0 (size (Pos so_51_P1)))))
(constraint (RegLan.Sem r (ReAsp so_51_P2 0 (size (Pos so_51_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N1 0 (size (Pos so_51_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N2 0 (size (Pos so_51_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N3 0 (size (Pos so_51_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N4 0 (size (Pos so_51_N4))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N5 0 (size (Pos so_51_N5))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N6 0 (size (Pos so_51_N6))))))
(constraint (not (RegLan.Sem r (ReAsp so_51_N7 0 (size (Pos so_51_N7))))))

(check-synth)
