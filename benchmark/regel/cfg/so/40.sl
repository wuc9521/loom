(set-logic ALL)
(require "../theories/so/40.sl")
(require "../../common/models/so/40.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_40_P1 0 (size (Pos so_40_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_40_P2 0 (size (Pos so_40_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_40_P3 0 (size (Pos so_40_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_40_N1 0 (size (Pos so_40_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_40_N2 0 (size (Pos so_40_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_40_N3 0 (size (Pos so_40_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_40_N4 0 (size (Pos so_40_N4)) reset.S))))

(check-synth)
