(set-logic ALL)
(require "../theories/so/64.sl")
(require "../../common/models/so/64.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_64_P1 0 (size (Pos so_64_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_64_P2 0 (size (Pos so_64_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_64_N1 0 (size (Pos so_64_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_64_N2 0 (size (Pos so_64_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_64_N3 0 (size (Pos so_64_N3)) reset.S))))

(check-synth)
