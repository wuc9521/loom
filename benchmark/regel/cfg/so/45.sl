(set-logic ALL)
(require "../theories/so/45.sl")
(require "../../common/models/so/45.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_45_P1 0 (size (Pos so_45_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_45_P2 0 (size (Pos so_45_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_45_N1 0 (size (Pos so_45_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_45_N2 0 (size (Pos so_45_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_45_N3 0 (size (Pos so_45_N3)) reset.S))))

(check-synth)
