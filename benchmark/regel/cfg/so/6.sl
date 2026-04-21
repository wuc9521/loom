(set-logic ALL)
(require "../theories/so/6.sl")
(require "../../common/models/so/6.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_6_P1 0 (size (Pos so_6_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_6_P2 0 (size (Pos so_6_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_6_P3 0 (size (Pos so_6_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_6_P4 0 (size (Pos so_6_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N1 0 (size (Pos so_6_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N2 0 (size (Pos so_6_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N3 0 (size (Pos so_6_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N4 0 (size (Pos so_6_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N5 0 (size (Pos so_6_N5)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_6_N6 0 (size (Pos so_6_N6)) reset.S))))

(check-synth)
