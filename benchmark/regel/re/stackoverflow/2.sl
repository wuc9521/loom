(require "../theories/so/2.sl")
(require "../../common/models/so/2.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_2_P1 0 (size (Pos so_2_P1)))))
(constraint (RegLan.Sem r (ReAsp so_2_P2 0 (size (Pos so_2_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_2_N1 0 (size (Pos so_2_N1))))))

(check-synth)
