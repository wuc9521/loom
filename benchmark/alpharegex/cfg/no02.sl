(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no02.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p01 0 (size (Pos p01)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p_01 0 (size (Pos p_01)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__01 0 (size (Pos p__01)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n00 0 (size (Pos n00)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n10 0 (size (Pos n10)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n11 0 (size (Pos n11)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n_?00 0 (size (Pos n_?00)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n_?10 0 (size (Pos n_?10)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n_?11 0 (size (Pos n_?11)) reset.S))))

(check-synth)
