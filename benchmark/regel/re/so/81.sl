(require "../theories/so/81.sl")
(require "../../common/models/so/81.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_81_P1 0 (size (Pos so_81_P1)))))
(constraint (RegLan.Sem r (ReAsp so_81_P2 0 (size (Pos so_81_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_81_N1 0 (size (Pos so_81_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_81_N2 0 (size (Pos so_81_N2))))))

(check-synth)
