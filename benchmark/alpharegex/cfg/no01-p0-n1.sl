(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no01.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p0 0 (size (Pos p0)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n1 0 (size (Pos n1)) reset.S))))

(check-synth)
