(set-logic ALL)
(require "../theories/so/20.sl")
(require "../../common/models/so/20.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_20_P1 0 (size (Pos so_20_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_20_P2 0 (size (Pos so_20_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_20_P3 0 (size (Pos so_20_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_20_P4 0 (size (Pos so_20_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_20_N1 0 (size (Pos so_20_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_20_N2 0 (size (Pos so_20_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_20_N3 0 (size (Pos so_20_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_20_N4 0 (size (Pos so_20_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_20_N5 0 (size (Pos so_20_N5)) reset.S))))

(check-synth)
