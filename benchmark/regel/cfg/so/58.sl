(set-logic ALL)
(require "../theories/so/58.sl")
(require "../../common/models/so/58.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_58_P1 0 (size (Pos so_58_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_58_P2 0 (size (Pos so_58_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_58_P3 0 (size (Pos so_58_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_58_N1 0 (size (Pos so_58_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_58_N2 0 (size (Pos so_58_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_58_N3 0 (size (Pos so_58_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_58_N4 0 (size (Pos so_58_N4)) reset.S))))

(check-synth)
