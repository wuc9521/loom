(require "../theories/so/59.sl")
(require "../../common/models/so/59.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_59_P1 0 (size (Pos so_59_P1)))))
(constraint (RegLan.Sem r (ReAsp so_59_P2 0 (size (Pos so_59_P2)))))
(constraint (RegLan.Sem r (ReAsp so_59_P3 0 (size (Pos so_59_P3)))))
(constraint (RegLan.Sem r (ReAsp so_59_P4 0 (size (Pos so_59_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_59_N1 0 (size (Pos so_59_N1))))))

(check-synth)
