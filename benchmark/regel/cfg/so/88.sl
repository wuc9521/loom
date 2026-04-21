(set-logic ALL)
(require "../theories/so/88.sl")
(require "../../common/models/so/88.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_88_P1 0 (size (Pos so_88_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_88_P2 0 (size (Pos so_88_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_88_P3 0 (size (Pos so_88_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_88_P4 0 (size (Pos so_88_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_88_N1 0 (size (Pos so_88_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_88_N2 0 (size (Pos so_88_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_88_N3 0 (size (Pos so_88_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_88_N4 0 (size (Pos so_88_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_88_N5 0 (size (Pos so_88_N5)) reset.S))))

(check-synth)
