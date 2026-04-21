(set-logic ALL)
(require "../theories/so/120.sl")
(require "../../common/models/so/120.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_120_P1 0 (size (Pos so_120_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_120_P2 0 (size (Pos so_120_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_120_P3 0 (size (Pos so_120_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_120_N1 0 (size (Pos so_120_N1)) reset.S))))

(check-synth)
