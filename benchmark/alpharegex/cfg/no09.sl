(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no09.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p0____ 0 (size (Pos p0____)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p_0____ 0 (size (Pos p_0____)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__0____ 0 (size (Pos p__0____)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n1???? 0 (size (Pos n1????)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?1???? 0 (size (Pos n?1????)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??1???? 0 (size (Pos n??1????)) reset.S))))

(check-synth)
