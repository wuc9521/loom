(set-logic ALL)
(require "../theories/so/24.sl")
(require "../../common/models/so/24.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_24_P1 0 (size (Pos so_24_P1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_24_N1 0 (size (Pos so_24_N1)) reset.S))))

(check-synth)
