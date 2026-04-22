(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no25.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p0 0 (size (Pos p0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p11 0 (size (Pos p11)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p101010 0 (size (Pos p101010)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p00011000 0 (size (Pos p00011000)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p00100110001 0 (size (Pos p00100110001)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n111 0 (size (Pos n111)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n110011 0 (size (Pos n110011)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n0110110 0 (size (Pos n0110110)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n00011001100 0 (size (Pos n00011001100)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n011100011110 0 (size (Pos n011100011110)) reset.S))))

(check-synth)
