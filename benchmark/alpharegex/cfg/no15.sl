(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no15.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p__ 0 (size (Pos p__)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p___ 0 (size (Pos p___)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p____ 0 (size (Pos p____)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n0 0 (size (Pos n0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1 0 (size (Pos n1)) reset.S))))

(check-synth)
