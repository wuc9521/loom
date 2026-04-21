(require "../theories/so/56.sl")
(require "../../common/models/so/56.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_56_P1 0 (size (Pos so_56_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_56_N1 0 (size (Pos so_56_N1))))))

(check-synth)
