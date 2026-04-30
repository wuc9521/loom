(require "../theories/so/89.sl")
(require "../../common/models/so/89.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp so_89_P1 0 (size (Pos so_89_P1)))))
(constraint (RegLan.Sem r (ReAsp so_89_P2 0 (size (Pos so_89_P2)))))
(constraint (not (RegLan.Sem r (ReAsp so_89_N1 0 (size (Pos so_89_N1))))))
(constraint (not (RegLan.Sem r (ReAsp so_89_N2 0 (size (Pos so_89_N2))))))

(check-synth)
