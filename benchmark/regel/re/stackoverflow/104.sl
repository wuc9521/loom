(require "../theories/so/104.sl")
(require "../../common/models/so/104.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_104_P1 0 (size (Pos so_104_P1)))))
(constraint (RegLan.Sem r (ReAsp so_104_P2 0 (size (Pos so_104_P2)))))
(constraint (RegLan.Sem r (ReAsp so_104_P3 0 (size (Pos so_104_P3)))))
(constraint (RegLan.Sem r (ReAsp so_104_P4 0 (size (Pos so_104_P4)))))
(constraint (RegLan.Sem r (ReAsp so_104_P5 0 (size (Pos so_104_P5)))))
(constraint (RegLan.Sem r (ReAsp so_104_P6 0 (size (Pos so_104_P6)))))
(constraint (not (RegLan.Sem r (ReAsp so_104_N1 0 (size (Pos so_104_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_104_N2 0 (size (Pos so_104_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_104_N3 0 (size (Pos so_104_N3))))))

(check-synth)
