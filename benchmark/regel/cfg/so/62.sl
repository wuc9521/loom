(set-logic ALL)
(require "../theories/so/62.sl")
(require "../../common/models/so/62.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp so_62_P1 0 (size (Pos so_62_P1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp so_62_P2 0 (size (Pos so_62_P2)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp so_62_N1 0 (size (Pos so_62_N1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp so_62_N2 0 (size (Pos so_62_N2)) reset.S))))

(check-synth)
