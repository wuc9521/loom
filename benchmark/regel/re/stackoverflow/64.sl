(require "../theories/so/64.sl")
(require "../../common/models/so/64.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_64_P1 0 (size (Pos so_64_P1)))))
(constraint (RegLan.Sem r (ReAsp so_64_P2 0 (size (Pos so_64_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_64_N1 0 (size (Pos so_64_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_64_N2 0 (size (Pos so_64_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_64_N3 0 (size (Pos so_64_N3))))))

(check-synth)
