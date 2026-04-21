(set-logic ALL)
(require "../theories/so/12.sl")
(require "../../common/models/so/12.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_12_P1 0 (size (Pos so_12_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_12_P2 0 (size (Pos so_12_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_12_P3 0 (size (Pos so_12_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_12_P4 0 (size (Pos so_12_P4)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_12_P5 0 (size (Pos so_12_P5)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N1 0 (size (Pos so_12_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N2 0 (size (Pos so_12_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N3 0 (size (Pos so_12_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N4 0 (size (Pos so_12_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N5 0 (size (Pos so_12_N5)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_12_N6 0 (size (Pos so_12_N6)) reset.S))))

(check-synth)
