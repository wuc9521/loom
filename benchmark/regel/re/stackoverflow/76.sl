(require "../theories/so/76.sl")
(require "../../common/models/so/76.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_76_P1 0 (size (Pos so_76_P1)))))
(constraint (RegLan.Sem r (ReAsp so_76_P2 0 (size (Pos so_76_P2)))))
(constraint (RegLan.Sem r (ReAsp so_76_P3 0 (size (Pos so_76_P3)))))
(constraint (not (RegLan.Sem r (ReAsp so_76_N1 0 (size (Pos so_76_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_76_N2 0 (size (Pos so_76_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_76_N3 0 (size (Pos so_76_N3))))))

(check-synth)
