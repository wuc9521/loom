(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no16.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p_ 0 (size (Pos p_)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p_0 0 (size (Pos p_0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p11 0 (size (Pos p11)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__0 0 (size (Pos p__0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p_11 0 (size (Pos p_11)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p___0 0 (size (Pos p___0)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__11 0 (size (Pos p__11)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p___11 0 (size (Pos p___11)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n01 0 (size (Pos n01)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?01 0 (size (Pos n?01)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??01 0 (size (Pos n??01)) reset.S))))

(check-synth)
