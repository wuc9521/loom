(set-logic ALL)
(require "../theories/so/102.sl")
(require "../../common/models/so/102.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_102_P1 0 (size (Pos so_102_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_102_P2 0 (size (Pos so_102_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_102_P3 0 (size (Pos so_102_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_102_P4 0 (size (Pos so_102_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_102_N1 0 (size (Pos so_102_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_102_N2 0 (size (Pos so_102_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_102_N3 0 (size (Pos so_102_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_102_N4 0 (size (Pos so_102_N4)) reset.S))))

(check-synth)
