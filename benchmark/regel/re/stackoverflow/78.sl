(require "../theories/so/78.sl")
(require "../../common/models/so/78.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_78_P1 0 (size (Pos so_78_P1)))))
(constraint (not (RegLan.Sem r (ReAsp so_78_N1 0 (size (Pos so_78_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_78_N2 0 (size (Pos so_78_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_78_N3 0 (size (Pos so_78_N3))))))

(check-synth)
