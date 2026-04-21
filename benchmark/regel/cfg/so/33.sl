(set-logic ALL)
(require "../theories/so/33.sl")
(require "../../common/models/so/33.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_33_P1 0 (size (Pos so_33_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_33_P2 0 (size (Pos so_33_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_33_N1 0 (size (Pos so_33_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_33_N2 0 (size (Pos so_33_N2)) reset.S))))

(check-synth)
