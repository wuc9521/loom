(set-logic ALL)
(require "../theories/so/10.sl")
(require "../../common/models/so/10.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_10_P1 0 (size (Pos so_10_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P2 0 (size (Pos so_10_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P3 0 (size (Pos so_10_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P4 0 (size (Pos so_10_P4)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P5 0 (size (Pos so_10_P5)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P6 0 (size (Pos so_10_P6)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_10_P7 0 (size (Pos so_10_P7)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_10_N1 0 (size (Pos so_10_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_10_N2 0 (size (Pos so_10_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_10_N3 0 (size (Pos so_10_N3)) reset.S))))

(check-synth)
