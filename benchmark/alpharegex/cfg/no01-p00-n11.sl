(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no01.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p00 0 (size (Pos p00)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n11 0 (size (Pos n11)) reset.S))))

(check-synth)
