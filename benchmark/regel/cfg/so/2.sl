(set-logic ALL)
(require "../theories/so/2.sl")
(require "../../common/models/so/2.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_2_P1 0 (size (Pos so_2_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_2_P2 0 (size (Pos so_2_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_2_N1 0 (size (Pos so_2_N1)) reset.S))))

(check-synth)
