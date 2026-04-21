(set-logic ALL)
(require "../theories/so/78.sl")
(require "../../common/models/so/78.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_78_P1 0 (size (Pos so_78_P1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_78_N1 0 (size (Pos so_78_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_78_N2 0 (size (Pos so_78_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_78_N3 0 (size (Pos so_78_N3)) reset.S))))

(check-synth)
