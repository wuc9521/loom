(set-logic ALL)
(require "../theories/so/36.sl")
(require "../../common/models/so/36.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_36_P1 0 (size (Pos so_36_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_36_P2 0 (size (Pos so_36_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_36_P3 0 (size (Pos so_36_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_36_N1 0 (size (Pos so_36_N1)) reset.S))))

(check-synth)
