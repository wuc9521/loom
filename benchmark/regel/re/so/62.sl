(require "../theories/so/62.sl")
(require "../../common/models/so/62.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_62_P1 0 (size (Pos so_62_P1)))))
(constraint (RegLan.Sem r (ReAsp so_62_P2 0 (size (Pos so_62_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_62_N1 0 (size (Pos so_62_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_62_N2 0 (size (Pos so_62_N2))))))

(check-synth)
