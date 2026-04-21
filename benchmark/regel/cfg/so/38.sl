(set-logic ALL)
(require "../theories/so/38.sl")
(require "../../common/models/so/38.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_38_P1 0 (size (Pos so_38_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_38_P2 0 (size (Pos so_38_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_38_N1 0 (size (Pos so_38_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_38_N2 0 (size (Pos so_38_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_38_N3 0 (size (Pos so_38_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_38_N4 0 (size (Pos so_38_N4)) reset.S))))

(check-synth)
