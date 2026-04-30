(require "../theories/so/52.sl")
(require "../../common/models/so/52.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_52_P1 0 (size (Pos so_52_P1)))))
(constraint (RegLan.Sem r (ReAsp so_52_P2 0 (size (Pos so_52_P2)))))
(constraint (RegLan.Sem r (ReAsp so_52_P3 0 (size (Pos so_52_P3)))))
(constraint (RegLan.Sem r (ReAsp so_52_P4 0 (size (Pos so_52_P4)))))
(constraint (not (RegLan.Sem r (ReAsp so_52_N1 0 (size (Pos so_52_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_52_N2 0 (size (Pos so_52_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_52_N3 0 (size (Pos so_52_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_52_N4 0 (size (Pos so_52_N4))))))

(check-synth)
