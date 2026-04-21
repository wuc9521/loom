(set-logic ALL)
(require "../theories/so/103.sl")
(require "../../common/models/so/103.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_103_P1 0 (size (Pos so_103_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_103_P2 0 (size (Pos so_103_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_103_P3 0 (size (Pos so_103_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_103_P4 0 (size (Pos so_103_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_103_N1 0 (size (Pos so_103_N1)) reset.S))))

(check-synth)
