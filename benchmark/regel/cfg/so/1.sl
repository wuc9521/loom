(set-logic ALL)
(require "../theories/so/1.sl")
(require "../../common/models/so/1.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_1_P1 0 (size (Pos so_1_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_1_P2 0 (size (Pos so_1_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_1_P3 0 (size (Pos so_1_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_1_P4 0 (size (Pos so_1_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_1_N1 0 (size (Pos so_1_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_1_N2 0 (size (Pos so_1_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_1_N3 0 (size (Pos so_1_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_1_N4 0 (size (Pos so_1_N4)) reset.S))))

(check-synth)
