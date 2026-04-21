(set-logic ALL)
(require "../theories/so/59.sl")
(require "../../common/models/so/59.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_59_P1 0 (size (Pos so_59_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_59_P2 0 (size (Pos so_59_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_59_P3 0 (size (Pos so_59_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_59_P4 0 (size (Pos so_59_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_59_N1 0 (size (Pos so_59_N1)) reset.S))))

(check-synth)
