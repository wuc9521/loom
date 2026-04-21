(set-logic ALL)
(require "../theories/so/76.sl")
(require "../../common/models/so/76.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_76_P1 0 (size (Pos so_76_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_76_P2 0 (size (Pos so_76_P2)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_76_P3 0 (size (Pos so_76_P3)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_76_N1 0 (size (Pos so_76_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_76_N2 0 (size (Pos so_76_N2)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_76_N3 0 (size (Pos so_76_N3)) reset.S))))

(check-synth)
