(set-logic ALL)
(require "../theories/so/9.sl")
(require "../../common/models/so/9.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_9_P1 0 (size (Pos so_9_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_9_P2 0 (size (Pos so_9_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_9_P3 0 (size (Pos so_9_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_9_P4 0 (size (Pos so_9_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_9_N1 0 (size (Pos so_9_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_9_N2 0 (size (Pos so_9_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_9_N3 0 (size (Pos so_9_N3)) reset.S))))

(check-synth)
