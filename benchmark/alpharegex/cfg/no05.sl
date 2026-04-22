(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no05.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p__0 0 (size (Pos p__0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__0_ 0 (size (Pos p__0_)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__0__ 0 (size (Pos p__0__)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n? 0 (size (Pos n?)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?? 0 (size (Pos n??)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??1 0 (size (Pos n??1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??1? 0 (size (Pos n??1?)) reset.S))))

(check-synth)
