(set-logic ALL)
(require "../theories/so/74.sl")
(require "../../common/models/so/74.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_74_P1 0 (size (Pos so_74_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_74_P2 0 (size (Pos so_74_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_74_N1 0 (size (Pos so_74_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_74_N2 0 (size (Pos so_74_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_74_N3 0 (size (Pos so_74_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_74_N4 0 (size (Pos so_74_N4)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_74_N5 0 (size (Pos so_74_N5)) reset.S))))

(check-synth)
