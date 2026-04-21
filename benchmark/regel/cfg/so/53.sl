(set-logic ALL)
(require "../theories/so/53.sl")
(require "../../common/models/so/53.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_53_P1 0 (size (Pos so_53_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_53_P2 0 (size (Pos so_53_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_53_P3 0 (size (Pos so_53_P3)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_53_P4 0 (size (Pos so_53_P4)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_53_P5 0 (size (Pos so_53_P5)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_53_N1 0 (size (Pos so_53_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_53_N2 0 (size (Pos so_53_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_53_N3 0 (size (Pos so_53_N3)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_53_N4 0 (size (Pos so_53_N4)) reset.S))))

(check-synth)
