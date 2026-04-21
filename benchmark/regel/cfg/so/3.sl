(set-logic ALL)
(require "../theories/so/3.sl")
(require "../../common/models/so/3.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_3_P1 0 (size (Pos so_3_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_3_P2 0 (size (Pos so_3_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_3_N1 0 (size (Pos so_3_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_3_N2 0 (size (Pos so_3_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_3_N3 0 (size (Pos so_3_N3)) reset.S))))

(check-synth)
