(set-logic ALL)
(require "../theories/so/56.sl")
(require "../../common/models/so/56.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_56_P1 0 (size (Pos so_56_P1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_56_N1 0 (size (Pos so_56_N1)) reset.S))))

(check-synth)
