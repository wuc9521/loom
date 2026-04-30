(require "../theories/so/85.sl")
(require "../../common/models/so/85.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_85_P1 0 (size (Pos so_85_P1)))))
(constraint (RegLan.Sem r (ReAsp so_85_P2 0 (size (Pos so_85_P2)))))
(constraint (RegLan.Sem r (ReAsp so_85_P3 0 (size (Pos so_85_P3)))))
(constraint (RegLan.Sem r (ReAsp so_85_P4 0 (size (Pos so_85_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_85_N1 0 (size (Pos so_85_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_85_N2 0 (size (Pos so_85_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_85_N3 0 (size (Pos so_85_N3))))))

(check-synth)
