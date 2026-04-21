(set-logic ALL)
(require "../theories/so/96.sl")
(require "../../common/models/so/96.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_96_P1 0 (size (Pos so_96_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_96_P2 0 (size (Pos so_96_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_96_P3 0 (size (Pos so_96_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_96_P4 0 (size (Pos so_96_P4)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_96_N1 0 (size (Pos so_96_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_96_N2 0 (size (Pos so_96_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_96_N3 0 (size (Pos so_96_N3)) reset.S))))

(check-synth)
