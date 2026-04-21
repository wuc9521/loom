(set-logic ALL)
(require "../theories/so/89.sl")
(require "../../common/models/so/89.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_89_P1 0 (size (Pos so_89_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_89_P2 0 (size (Pos so_89_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_89_N1 0 (size (Pos so_89_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_89_N2 0 (size (Pos so_89_N2)) reset.S))))

(check-synth)
