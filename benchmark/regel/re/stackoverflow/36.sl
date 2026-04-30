(require "../theories/so/36.sl")
(require "../../common/models/so/36.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_36_P1 0 (size (Pos so_36_P1)))))
(constraint (RegLan.Sem r (ReAsp so_36_P2 0 (size (Pos so_36_P2)))))
(constraint (RegLan.Sem r (ReAsp so_36_P3 0 (size (Pos so_36_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_36_N1 0 (size (Pos so_36_N1))))))

(check-synth)
