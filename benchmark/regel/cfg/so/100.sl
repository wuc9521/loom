(set-logic ALL)
(require "../theories/so/100.sl")
(require "../../common/models/so/100.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_100_P1 0 (size (Pos so_100_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_100_P2 0 (size (Pos so_100_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_100_P3 0 (size (Pos so_100_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_100_P4 0 (size (Pos so_100_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_100_N1 0 (size (Pos so_100_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_100_N2 0 (size (Pos so_100_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_100_N3 0 (size (Pos so_100_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_100_N4 0 (size (Pos so_100_N4)) reset.S))))

(check-synth)
