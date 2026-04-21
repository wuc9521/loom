(set-logic ALL)
(require "../theories/so/25.sl")
(require "../../common/models/so/25.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_25_P1 0 (size (Pos so_25_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_25_P2 0 (size (Pos so_25_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_25_P3 0 (size (Pos so_25_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_25_P4 0 (size (Pos so_25_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_25_N1 0 (size (Pos so_25_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_25_N2 0 (size (Pos so_25_N2)) reset.S))))

(check-synth)
