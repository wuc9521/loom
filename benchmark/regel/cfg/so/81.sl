(set-logic ALL)
(require "../theories/so/81.sl")
(require "../../common/models/so/81.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_81_P1 0 (size (Pos so_81_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_81_P2 0 (size (Pos so_81_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_81_N1 0 (size (Pos so_81_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_81_N2 0 (size (Pos so_81_N2)) reset.S))))

(check-synth)
