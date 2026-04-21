(set-logic ALL)
(require "../theories/so/101.sl")
(require "../../common/models/so/101.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_101_P1 0 (size (Pos so_101_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_101_P2 0 (size (Pos so_101_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_101_P3 0 (size (Pos so_101_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_101_P4 0 (size (Pos so_101_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_101_N1 0 (size (Pos so_101_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_101_N2 0 (size (Pos so_101_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_101_N3 0 (size (Pos so_101_N3)) reset.S))))

(check-synth)
