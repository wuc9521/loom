(set-logic ALL)
(require "../theories/so/42.sl")
(require "../../common/models/so/42.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_42_P1 0 (size (Pos so_42_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_42_P2 0 (size (Pos so_42_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_42_N1 0 (size (Pos so_42_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_42_N2 0 (size (Pos so_42_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_42_N3 0 (size (Pos so_42_N3)) reset.S))))

(check-synth)
