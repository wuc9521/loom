(require "../theories/so/24.sl")
(require "../../common/models/so/24.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_24_P1 0 (size (Pos so_24_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_24_N1 0 (size (Pos so_24_N1))))))

(check-synth)
