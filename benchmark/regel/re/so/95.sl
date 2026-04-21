(require "../theories/so/95.sl")
(require "../../common/models/so/95.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_95_P1 0 (size (Pos so_95_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_95_N1 0 (size (Pos so_95_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_95_N2 0 (size (Pos so_95_N2))))))

(check-synth)
