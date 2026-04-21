(set-logic ALL)
(require "../theories/so/79.sl")
(require "../../common/models/so/79.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_79_P1 0 (size (Pos so_79_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_79_P2 0 (size (Pos so_79_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_79_P3 0 (size (Pos so_79_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_79_N1 0 (size (Pos so_79_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_79_N2 0 (size (Pos so_79_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_79_N3 0 (size (Pos so_79_N3)) reset.S))))

(check-synth)
