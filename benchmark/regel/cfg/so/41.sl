(set-logic ALL)
(require "../theories/so/41.sl")
(require "../../common/models/so/41.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_41_P1 0 (size (Pos so_41_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_41_P2 0 (size (Pos so_41_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_41_N1 0 (size (Pos so_41_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_41_N2 0 (size (Pos so_41_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_41_N3 0 (size (Pos so_41_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_41_N4 0 (size (Pos so_41_N4)) reset.S))))

(check-synth)
