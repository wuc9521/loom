(set-logic ALL)
(require "../theories/so/51.sl")
(require "../../common/models/so/51.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_51_P1 0 (size (Pos so_51_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_51_P2 0 (size (Pos so_51_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N1 0 (size (Pos so_51_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N2 0 (size (Pos so_51_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N3 0 (size (Pos so_51_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N4 0 (size (Pos so_51_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N5 0 (size (Pos so_51_N5)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N6 0 (size (Pos so_51_N6)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_51_N7 0 (size (Pos so_51_N7)) reset.S))))

(check-synth)
