(set-logic ALL)
(require "../theories/so/46.sl")
(require "../../common/models/so/46.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_46_P1 0 (size (Pos so_46_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_46_P2 0 (size (Pos so_46_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_46_P3 0 (size (Pos so_46_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_46_P4 0 (size (Pos so_46_P4)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_46_P5 0 (size (Pos so_46_P5)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_46_P6 0 (size (Pos so_46_P6)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_46_N1 0 (size (Pos so_46_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_46_N2 0 (size (Pos so_46_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_46_N3 0 (size (Pos so_46_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_46_N4 0 (size (Pos so_46_N4)) reset.S))))

(check-synth)
