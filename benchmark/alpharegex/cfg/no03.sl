(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no03.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p0101 0 (size (Pos p0101)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p_0101_ 0 (size (Pos p_0101_)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p__0101__ 0 (size (Pos p__0101__)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n0 0 (size (Pos n0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1 0 (size (Pos n1)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?? 0 (size (Pos n??)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??? 0 (size (Pos n???)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n???0 0 (size (Pos n???0)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?0?? 0 (size (Pos n?0??)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n1??? 0 (size (Pos n1???)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n??1? 0 (size (Pos n??1?)) reset.S))))

(check-synth)
