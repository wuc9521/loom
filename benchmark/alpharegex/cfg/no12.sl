(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no12.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p000 0 (size (Pos p000)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p00011 0 (size (Pos p00011)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p0001111 0 (size (Pos p0001111)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p000011 0 (size (Pos p000011)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p00001111 0 (size (Pos p00001111)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p00000111111 0 (size (Pos p00000111111)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n0 0 (size (Pos n0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n00 0 (size (Pos n00)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0001 0 (size (Pos n0001)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n00001 0 (size (Pos n00001)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0000111 0 (size (Pos n0000111)) reset.S))))

(check-synth)
