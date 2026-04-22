(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no04.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p01 0 (size (Pos p01)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p0_1 0 (size (Pos p0_1)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p0__1 0 (size (Pos p0__1)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n1 0 (size (Pos n1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0 0 (size (Pos n0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n00 0 (size (Pos n00)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0?0 0 (size (Pos n0?0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1?1 0 (size (Pos n1?1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1?0 0 (size (Pos n1?0)) reset.S))))

(check-synth)
