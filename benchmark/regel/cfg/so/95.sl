(set-logic ALL)
(require "../theories/so/95.sl")
(require "../../common/models/so/95.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_95_P1 0 (size (Pos so_95_P1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_95_N1 0 (size (Pos so_95_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_95_N2 0 (size (Pos so_95_N2)) reset.S))))

(check-synth)
