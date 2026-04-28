(require "../theories/so/53.sl")
(require "../../common/models/so/53.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp so_53_P1 0 (size (Pos so_53_P1)))))
(constraint (RegLan.Sem r (ReAsp so_53_P2 0 (size (Pos so_53_P2)))))
(constraint (RegLan.Sem r (ReAsp so_53_P3 0 (size (Pos so_53_P3)))))
(constraint (RegLan.Sem r (ReAsp so_53_P4 0 (size (Pos so_53_P4)))))
(constraint (RegLan.Sem r (ReAsp so_53_P5 0 (size (Pos so_53_P5)))))
(constraint (not (RegLan.Sem r (ReAsp so_53_N1 0 (size (Pos so_53_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_53_N2 0 (size (Pos so_53_N2))))))
(constraint (not (RegLan.Sem r (ReAsp so_53_N3 0 (size (Pos so_53_N3))))))
(constraint (not (RegLan.Sem r (ReAsp so_53_N4 0 (size (Pos so_53_N4))))))

(check-synth)
