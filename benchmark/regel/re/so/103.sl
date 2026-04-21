(require "../theories/so/103.sl")
(require "../../common/models/so/103.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_103_P1 0 (size (Pos so_103_P1)))))
(constraint (RegLan.Sem r (ReAsp so_103_P2 0 (size (Pos so_103_P2)))))
(constraint (RegLan.Sem r (ReAsp so_103_P3 0 (size (Pos so_103_P3)))))
(constraint (RegLan.Sem r (ReAsp so_103_P4 0 (size (Pos so_103_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_103_N1 0 (size (Pos so_103_N1))))))

(check-synth)
