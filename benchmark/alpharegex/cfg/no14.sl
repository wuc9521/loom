(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no14.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p0 0 (size (Pos p0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p0__ 0 (size (Pos p0__)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p0____ 0 (size (Pos p0____)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p1_ 0 (size (Pos p1_)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p1___ 0 (size (Pos p1___)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n0? 0 (size (Pos n0?)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0??? 0 (size (Pos n0???)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1 0 (size (Pos n1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1?? 0 (size (Pos n1??)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1???? 0 (size (Pos n1????)) reset.S))))

(check-synth)
