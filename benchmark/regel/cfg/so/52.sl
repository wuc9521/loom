(set-logic ALL)
(require "../theories/so/52.sl")
(require "../../common/models/so/52.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_52_P1 0 (size (Pos so_52_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_52_P2 0 (size (Pos so_52_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_52_P3 0 (size (Pos so_52_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_52_P4 0 (size (Pos so_52_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_52_N1 0 (size (Pos so_52_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_52_N2 0 (size (Pos so_52_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_52_N3 0 (size (Pos so_52_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_52_N4 0 (size (Pos so_52_N4)) reset.S))))

(check-synth)
