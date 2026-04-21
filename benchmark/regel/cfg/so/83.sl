(set-logic ALL)
(require "../theories/so/83.sl")
(require "../../common/models/so/83.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_83_P1 0 (size (Pos so_83_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_83_P2 0 (size (Pos so_83_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_83_P3 0 (size (Pos so_83_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_83_P4 0 (size (Pos so_83_P4)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_83_P5 0 (size (Pos so_83_P5)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_83_N1 0 (size (Pos so_83_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_83_N2 0 (size (Pos so_83_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_83_N3 0 (size (Pos so_83_N3)) reset.S))))

(check-synth)
