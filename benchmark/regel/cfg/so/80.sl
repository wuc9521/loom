(set-logic ALL)
(require "../theories/so/80.sl")
(require "../../common/models/so/80.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_80_P1 0 (size (Pos so_80_P1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_80_N1 0 (size (Pos so_80_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_80_N2 0 (size (Pos so_80_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_80_N3 0 (size (Pos so_80_N3)) reset.S))))

(check-synth)
