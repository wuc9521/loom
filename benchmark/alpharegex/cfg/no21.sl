(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no21.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p00 0 (size (Pos p00)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p1001 0 (size (Pos p1001)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p010010 0 (size (Pos p010010)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p1011001110 0 (size (Pos p1011001110)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n01 0 (size (Pos n01)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n11 0 (size (Pos n11)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n000 0 (size (Pos n000)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n00100 0 (size (Pos n00100)) reset.S))))

(check-synth)
