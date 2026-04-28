(require "../theories/so/96.sl")
(require "../../common/models/so/96.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_96_P1 0 (size (Pos so_96_P1)))))
(constraint (RegLan.Sem r (ReAsp so_96_P2 0 (size (Pos so_96_P2)))))
(constraint (RegLan.Sem r (ReAsp so_96_P3 0 (size (Pos so_96_P3)))))
(constraint (RegLan.Sem r (ReAsp so_96_P4 0 (size (Pos so_96_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_96_N1 0 (size (Pos so_96_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_96_N2 0 (size (Pos so_96_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_96_N3 0 (size (Pos so_96_N3))))))

(check-synth)
